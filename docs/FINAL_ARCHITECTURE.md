# Final Architecture Summary

## Overview

Generate GTO poker solutions for all position/scenario combinations, store in local database, and provide real-time lookup during play.

---

## Phase 1: Range Preparation

### Build 80 Position-Agnostic Ranges

**Structure**:
```
ranges/
├── GTO/
│   ├── UTG/
│   │   ├── RFI.txt
│   │   ├── vs_3bet_by_MP.txt
│   │   ├── vs_3bet_by_CO.txt
│   │   └── ... (11 total)
│   ├── MP/ (12 ranges)
│   ├── CO/ (13 ranges)
│   ├── BTN/ (14 ranges)
│   ├── SB/ (15 ranges)
│   └── BB/ (15 ranges)
```

**Total**: 80 ranges (position-specific, not hero/villain specific)

**Key point**: These ranges are general. `BTN/RFI.txt` is the same whether hero or villain is on BTN.

---

## Phase 2: Generate Solve Pairs

### Pair Ranges as (IP, OOP)

For each preflop scenario, determine which two ranges to pair:

```python
# Example: "BTN opens, BB calls"
scenario = "BTN_RFI_vs_BB_call"
ip_range = "ranges/GTO/BTN/RFI.txt"
oop_range = "ranges/GTO/BB/vs_RFI_BTN.txt"
```

### Hash-Based Deduplication

```python
def get_range_pair_hash(ip_range_file, oop_range_file):
    """
    Generate unique hash for range pair
    If two scenarios have identical range pairs, they get same hash
    """
    ip_hash = md5(open(ip_range_file, 'rb').read()).hexdigest()
    oop_hash = md5(open(oop_range_file, 'rb').read()).hexdigest()
    
    # Order matters: IP first, OOP second
    return f"{ip_hash}_{oop_hash}"

# Build solve queue with deduplication
solve_map = {}
scenario_to_solve = {}

for scenario in scenarios:
    for flop_bucket in flop_buckets:
        ip_range, oop_range = get_ranges_for_scenario(scenario)
        pair_hash = get_range_pair_hash(ip_range, oop_range)
        
        # First time seeing this range pair?
        if pair_hash not in solve_map:
            solve_map[pair_hash] = {
                'ip_range': ip_range,
                'oop_range': oop_range,
                'flop': get_representative_flop(flop_bucket),
                'scenarios': []
            }
        
        # Track which scenarios use this solve
        solve_map[pair_hash]['scenarios'].append((scenario, flop_bucket))
        scenario_to_solve[(scenario, flop_bucket)] = pair_hash
```

**Preflop Scenarios** (must end with call to reach flop):
1. RFI vs defend (call): 5 scenarios
2. RFI vs cold call: 3 scenarios  
3. Open → 3bet → call 3bet: 15 scenarios
4. 3bet → 4bet → call 4bet: 15 scenarios
**Total: 38 scenarios**

**Output**: 
- 38 scenarios × 184 flops = **6,992 total combinations**
- After deduplication: **~4,500-5,500 unique solves** (20-30% reduction)
- Each unique solve reused for multiple scenarios

---

## Phase 3: AWS Parallel Solving

### Upload to S3

```bash
# Upload ranges
aws s3 sync ranges/ s3://poker-solver/ranges/

# Upload solve queue
python generate_queue.py  # Creates solve_queue.json
aws s3 cp solve_queue.json s3://poker-solver/solve_queue.json

# Upload worker script
aws s3 cp worker.py s3://poker-solver/worker.py
```

### Launch EC2 Fleet

```bash
# 500 spot instances
aws ec2 run-instances \
  --image-id ami-0c55b159cbfafe1f0 \
  --instance-type c6i.xlarge \
  --spot-price "0.05" \
  --count 500 \
  --user-data file://startup.sh
```

**Worker loop** (on each EC2 instance):
```python
while True:
    job = claim_job_from_queue()
    if not job:
        break
    
    # Download ranges
    download_from_s3(job['ip_range'], '/tmp/ip.txt')
    download_from_s3(job['oop_range'], '/tmp/oop.txt')
    
    # Run solver
    subprocess.run([
        'postflop-solver',
        '--ip-range', '/tmp/ip.txt',
        '--oop-range', '/tmp/oop.txt',
        '--board', job['flop'],
        '--bet-sizes', '0.25,0.5,1.0',
        '--output', f'/tmp/{job["id"]}.json'
    ])
    
    # Compress and upload
    compressed = gzip.compress(open(f'/tmp/{job["id"]}.json', 'rb').read())
    upload_to_s3(compressed, f's3://poker-solver/solutions/{job["id"]}.json.gz')
```

**Output**: 4,000-5,000 game trees in S3 (compressed)

---

## Phase 4: Download & Build Local Database

```python
# Download all solutions from S3
for solution_file in list_s3_objects('poker-solver', 'solutions/'):
    download_from_s3(solution_file, f'data/raw_solutions/{solution_file}')

# Build SQLite database
db = sqlite3.connect('data/solutions.db')

# Create tables
db.execute("""
    CREATE TABLE game_trees (
        range_pair_hash TEXT,
        bucket_id TEXT,
        game_tree_json BLOB,
        PRIMARY KEY (range_pair_hash, bucket_id)
    )
""")

db.execute("""
    CREATE TABLE scenario_mapping (
        scenario TEXT,
        bucket_id TEXT,
        range_pair_hash TEXT,
        hero_position TEXT,
        PRIMARY KEY (scenario, bucket_id)
    )
""")

# Populate game_trees
for solution_file in os.listdir('data/raw_solutions'):
    with gzip.open(solution_file, 'rb') as f:
        tree = f.read()
    
    pair_hash, bucket_id = parse_filename(solution_file)
    
    db.execute("""
        INSERT INTO game_trees VALUES (?, ?, ?)
    """, (pair_hash, bucket_id, tree))

# Populate scenario_mapping
for (scenario, bucket_id), pair_hash in scenario_to_solve.items():
    hero_position = extract_hero_position(scenario)
    
    db.execute("""
        INSERT INTO scenario_mapping VALUES (?, ?, ?, ?)
    """, (scenario, bucket_id, pair_hash, hero_position))

db.commit()
```

**Database size**: ~50-150 GB (compressed game trees)

---

## Phase 5: Runtime Lookup

### 4 Required Inputs

1. **Preflop situation**: e.g., "BTN_RFI_vs_BB_call"
2. **Hero position**: "BTN" or "BB"
3. **Flop**: `['Ks', 'Qd', '7c']`
4. **Hero's cards**: `['As', 'Kd']`

### Lookup Flow

```python
def get_strategy(preflop_situation, hero_position, flop, hero_cards):
    """
    Returns action frequencies for hero's specific hand
    """
    
    # 1. Bucket the flop
    bucket_id = bucket_flop(flop)  # "rainbow_K-high_gapped"
    
    # 2. Map to range_pair_hash
    pair_hash = db.execute("""
        SELECT range_pair_hash FROM scenario_mapping
        WHERE scenario=? AND bucket_id=?
    """, (preflop_situation, bucket_id)).fetchone()[0]
    
    # 3. Load game tree (with caching)
    if (pair_hash, bucket_id) not in cache:
        compressed_tree = db.execute("""
            SELECT game_tree_json FROM game_trees
            WHERE range_pair_hash=? AND bucket_id=?
        """, (pair_hash, bucket_id)).fetchone()[0]
        
        tree = json.loads(gzip.decompress(compressed_tree))
        cache[(pair_hash, bucket_id)] = tree
    
    tree = cache[(pair_hash, bucket_id)]
    
    # 4. Determine if hero is IP or OOP
    hero_is_ip = determine_if_ip(preflop_situation, hero_position)
    
    # 5. Navigate tree based on action history
    node = tree['root']
    for action in action_history:
        node = node['actions'][action]
        if 'ip_response' in node:
            node = node['ip_response']
        elif 'oop_response' in node:
            node = node['oop_response']
    
    # 6. If turn/river, navigate to specific card
    if len(flop) > 3:  # Turn
        node = node['turn']['turn_cards'][flop[3]]
    if len(flop) > 4:  # River
        node = node['river']['river_cards'][flop[4]]
    
    # 7. Normalize hero's hand
    hero_hand = normalize_hand(hero_cards)  # "AKo", "AKs", etc.
    
    # 8. Extract frequencies for hero's hand
    hero_strategy = {}
    for action_name, action_data in node['actions'].items():
        frequency = action_data['strategy'].get(hero_hand, 0.0)
        hero_strategy[action_name] = frequency
    
    return hero_strategy
```

### Example

```python
strategy = get_strategy(
    preflop_situation="BTN_RFI_vs_BB_call",
    hero_position="BTN",
    flop=['Ks', 'Qd', '7c'],
    hero_cards=['As', 'Kd']
)

# Returns:
# {
#   'check': 0.35,
#   'bet_0.5': 0.50,
#   'bet_1.0': 0.15
# }

# Display on overlay:
# [Check: 35%] [Bet 50%: 50%] [Bet 100%: 15%]
```

---

## Summary: Complete Pipeline

```
1. Build 80 ranges
   ↓
2. Generate ~7,500 scenario×flop combinations
   ↓
3. Deduplicate → ~4,000-5,000 unique solves needed
   ↓
4. AWS EC2 fleet solves in parallel (5 hours, $75)
   ↓
5. Download results from S3
   ↓
6. Build local SQLite database (~100 GB)
   ↓
7. Runtime: 4 inputs → navigate tree → extract frequencies
   ↓
8. Display on overlay
```

**Total cost**: ~$75 AWS  
**Total time**: ~1 week (range building + solving + database setup)  
**Runtime performance**: <5ms lookup (with caching)

---

**End of Final Architecture**
