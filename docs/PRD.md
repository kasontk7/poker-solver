# PRD: Poker Overlay Bot for ACR (6-max $0.50/$1.00 NLHE)

**Target User**: Personal use only (single user)  
**Platform**: macOS  
**Game Type**: 6-max No Limit Hold'em Cash Games (100bb effective stacks)  
**Pot Types**: Heads-up only (no 3-way pots)

---

## Overview

A real-time poker solver overlay that displays GTO and exploitative strategies for ACR cash game tables. The system uses pre-solved solutions from postflop-solver, live screen reading via OCR, and player profiling from hand history analysis.

---

## Core Architecture

### 1. Screen Reading (Hybrid Approach)

**Pixel Monitoring** (triggers):
- Detect flop/turn/river appearance via color changes in board card regions
- Detect when action is on hero (button highlighting)

**OCR** (data extraction):
- **Preflop**:
  - Player positions (BTN, SB, BB, CO, MP, UTG)
  - Who folded, called, raised (with amounts)
  - Track complete preflop action sequence
- **Postflop**:
  - Board cards (3-5 cards depending on street)
  - Pot size
  - Bet/raise sizes
  - Action sequence (check, bet, call, raise, fold)
  - Whose turn to act

**Technology**: 
- pytesseract or EasyOCR for card/text recognition
- Pillow for screen capture
- Continuous monitoring thread at 60fps (16ms per frame)

**Latency Target**: 300-500ms from flop appearance to overlay display

---

### 2. GTO Solutions Database

**Pre-solved Trees**: Generated using postflop-solver (Rust, open source)

#### Flop Bucketing: 184 Canonical Flops

**Total flops after suit isomorphism**: **184** (industry standard)

Breakdown:
- **Monotone** (all 3 cards same suit): ~9 flops
- **Two-tone** (2 cards one suit, 1 another): ~55 flops
- **Rainbow** (3 different suits): ~120 flops

Each categorized by:
1. **Paired vs unpaired**
2. **High card**: A-high, K-high, Q-high, J-high, T-high, 9-2-high
3. **Connectedness**: 3-straight, 2-straight draw, gapped, very dry, broadway

**Examples**:
- A♠K♠Q♠ → `monotone_A-high_broadway`
- 7♥6♥5♣ → `two-tone_7-high_3-straight`
- K♦8♣2♠ → `rainbow_K-high_very-dry`

**Canonical representations** (what we solve):
- Monotone: All spades (e.g., `Ks Qs Js`)
- Two-tone: Spades primary, hearts secondary (e.g., `Ks Qh 6s`)
- Rainbow: Spades, hearts, diamonds (e.g., `Ks Qh 6d`)

#### Suit Normalization Algorithm

**Problem**: We solve 184 canonical flops, but need to handle 22,100 real flops at runtime. How do we map a real board like `Kh Qd 6h` to canonical `Ks Qh 6s` while preserving flush draws?

**Solution**: Consistent suit mapping that preserves hand-board relationships.

**Algorithm**:

1. **Count suit frequencies on board**
   ```python
   board = [Kh, Qd, 6h]
   suit_counts = {hearts: 2, diamonds: 1, spades: 0, clubs: 0}
   ```

2. **Order suits by frequency (primary → quaternary)**
   - Primary = most frequent suit on board (2 cards)
   - Secondary = second most frequent (1 card)
   - Tertiary/Quaternary = unused suits (0 cards each)
   - Tiebreaker: card rank order (A > K > Q > ... > 2)
   
   ```python
   suit_order = [hearts, diamonds, spades, clubs]
   ```

3. **Create mapping: real → canonical**
   ```python
   suit_map = {
       hearts: 's',     # Primary → spades (canonical)
       diamonds: 'h',   # Secondary → hearts
       spades: 'd',     # Tertiary → diamonds
       clubs: 'c'       # Quaternary → clubs
   }
   ```

4. **Map both board AND hero hand with SAME mapping**
   ```python
   # Real board: Kh Qd 6h
   canonical_board = [Ks, Qh, 6s]  # Using suit_map
   
   # Real hand: JhTh (flush draw!)
   canonical_hand = [Js, Ts]       # Using SAME suit_map
   ```

5. **Look up strategy in database**
   ```python
   strategy = db.query(
       board=canonical_board,
       hand=canonical_hand,
       scenario="BTN_vs_BB"
   )
   # Strategy includes flush draw equity because JsTs on Ks_Qh_6s 
   # is a flush draw (2 spades)
   ```

**Key properties**:
- ✅ Flush draws preserved (2+ cards of primary suit)
- ✅ Backdoor draws preserved (2 cards of same suit)
- ✅ Offsuit combos preserved (cards in different suit buckets)
- ✅ Blocker effects preserved (specific card removal)

**Example 1: Flush draw**
```
Real:     Board Ah Kh Qd | Hand JhTh
Mapping:  hearts→s, diamonds→h, spades→d, clubs→c
Canonical: Board As Ks Qh | Hand JsTs ← Flush draw (2 spades)
Result:   Correct! Strategy shows flush draw equity
```

**Example 2: Non-flush hand on same board**
```
Real:     Board Ah Kh Qd | Hand JdTd
Mapping:  hearts→s, diamonds→h, spades→d, clubs→c (SAME mapping!)
Canonical: Board As Ks Qh | Hand JhTh ← No flush draw (1 spade, 1 heart)
Result:   Correct! Strategy shows non-flush-draw equity
```

**Example 3: Rainbow board**
```
Real:     Board Ks Qh 6d | Hand AsKd
Mapping:  spades→s, hearts→h, diamonds→d, clubs→c
Canonical: Board Ks Qh 6d | Hand AsKd ← Exact match! No mapping needed
Result:   Direct lookup
```

**Implementation notes**:
- Suit mapping is computed once per board
- All hand lookups for that board use the same mapping
- Mapping is deterministic (same board always maps the same way)
- No reverse mapping needed (just use the frequencies directly)

#### Scenario Structure

Each postflop scenario is defined by:
- **Hero position & action**: BTN/RFI, BB/vs_RFI_BTN, CO/vs_3bet_by_UTG, etc.
- **Villain position & action**: Corresponding villain range
- **Pot type**: SRP, 3-bet pot, 4-bet pot
- **OOP/IP orientation**: Who acts first

**Total unique scenarios**: ~50-60 (e.g., BTN_RFI_vs_BB_call, CO_vs_3bet_by_UTG, etc.)

#### Range-Pair Deduplication Strategy

Ranges are **position-agnostic** (not hero/villain specific). Some range pairs will be identical:

**Hash-Based Deduplication (ELI5)**:

Think of a hash as a "fingerprint" for a file. Two files with identical content = identical fingerprint.

1. **Pre-solving**: 
   - Take BTN/RFI.txt + BB/vs_RFI_BTN.txt
   - Calculate fingerprint: `abc123`
   - Run solver once
   - Save tree as: `abc123_rainbow-K-high-gapped.json`

2. **If another scenario uses same ranges**:
   - CO/RFI.txt + BB/vs_RFI_CO.txt
   - If files are identical to BTN ranges, fingerprint = `abc123` (same!)
   - Don't solve again, just note: "CO scenario also uses `abc123` tree"

3. **At runtime**:
   - User asks: "BTN opened, BB called, flop is Ks Qd 7c, hero is BTN with As Kd"
   - Lookup: BTN opened = BTN/RFI.txt, BB called = BB/vs_RFI_BTN.txt
   - Calculate fingerprint from those 2 files: `abc123`
   - **Suit normalize**: Board `Ks Qd 7c` → Canonical `Ks Qh 7d` (rainbow K-high)
   - **Suit normalize**: Hand `As Kd` → Canonical `As Kh` (using SAME mapping)
   - Flop bucket: `rainbow-K-high-gapped`
   - Load tree: `abc123_rainbow-K-high-gapped.json`
   - Navigate tree for canonical hand `AsKh` → get frequencies

**Your understanding is 100% correct!**

**Technical Details**:
1. For each scenario, identify IP range and OOP range
2. Hash both range files: `md5(ip_range) + md5(oop_range)`
3. If two scenarios have identical (IP, OOP) range pairs, solve once and reuse

**Example**:
- Scenario A: `BTN_RFI_vs_BB_call` → IP: BTN/RFI.txt, OOP: BB/vs_RFI_BTN.txt
- Scenario B: `CO_RFI_vs_BB_call` → IP: CO/RFI.txt, OOP: BB/vs_RFI_CO.txt
- If BTN/RFI.txt = CO/RFI.txt (identical files), same hash → solve once

**Database Schema**:
```sql
-- Game trees (one per unique range pair × flop bucket)
game_trees: range_pair_hash, bucket_id → game_tree_json (BLOB)

-- Mapping: scenario → which tree to use
scenario_mapping: scenario, bucket_id → range_pair_hash, hero_position
```

**Preflop Scenarios** (must end with a call to reach flop):
- RFI vs defend (call): 5 scenarios (UTG/MP/CO/BTN/SB vs BB)
- RFI vs cold call: 3 scenarios (UTG/MP/CO vs BTN cold call)
- Open → 3bet → call 3bet: 15 scenarios
- 3bet → 4bet → call 4bet: 15 scenarios
- **Total: 38 unique preflop scenarios**

**Deduplication Results**:
- v1.2: 184 flops × 38 scenarios = 6,992 combinations
- After deduplication: **~4,500-5,500 unique solves** (estimated 20-30% reduction)

See `FINAL_ARCHITECTURE.md` for complete pipeline.

#### Solver Configuration

**Preflop Pot Sizes**:
- **RFI (raise first in)**: 2.5bb
- **SB completion**: 0.5bb posted + 0.5bb call = 1bb total
- **BB post**: 1bb (already in pot)
- **3-bet sizing**: 
  - OOP (BB/SB): 3.5-4x the raise (e.g., 2.5bb RFI → 9-10bb 3-bet)
  - IP (BTN/CO): 3x the raise (e.g., 2.5bb RFI → 7.5bb 3-bet)
- **4-bet sizing**: 2.2-2.5x the 3-bet (e.g., 10bb 3-bet → 22-25bb 4-bet)

**Example: BTN RFI vs BB call**:
- SB: 0.5bb (dead money)
- BB: 1bb posted + 1.5bb call = 2.5bb total
- BTN: 2.5bb raise
- **Pot at flop: 5.5bb**
- **Effective stack: 97.5bb**

**Bet Sizing Configuration** (applies to v1.0, v1.1, v1.2):
```json
{
  "flop_bet_sizes": ["25%", "50%", "100%"],
  "turn_bet_sizes": ["25%", "50%", "100%", "all-in"],
  "river_bet_sizes": ["50%", "100%", "150%", "all-in"],
  "raise_sizes": "2.5x",
  "donk_bet_sizes": ["33%", "50%"]
}
```

**Rationale**:
- **Flop**: Small to large bets, no all-in (too much play left)
- **Turn**: Added all-in option (SPR decreases)
- **River**: Larger sizes allowed (50%, pot, 1.5x pot, all-in)
- **Raises**: 2.5x multiplier (industry standard, balanced)
- **Donk bets**: Small/medium only (large donks are exploitable with capped OOP range)

**Bet sizing strategy**: Nearest node (v1). Linear interpolation for future versions.

#### Runtime Bucketing

**Flop Classification**:
- Suit normalization algorithm maps any board → 1 of 184 canonical flops
- Precomputed lookup table: all 22,100 flops → canonical bucket (~2MB JSON)
- Example: `Ks Qd 7c` → Canonical `Ks Qh 7d` → `"rainbow_K-high_gapped"`
- Runtime: <5ms per lookup (suit counting + mapping)

**Turn/River Handling**:
- For v1.0-v1.1: Use locked runouts (specific turn/river cards) to save memory
- For v1.2+: Solve full trees (all possible turns/rivers) using AWS
- Turn buckets: ~1,472 (184 flop × 8 turn effects)
- River buckets: ~11,776 (1,472 turn × 8 river effects)

**Phased Approach**:
- v1.1: 184 flops × locked runouts (~184 solves per scenario)
- v1.2: Full game trees with all turns/rivers (~5,000-6,000 unique solves)
- v2: OCR + Overlay integration
- v3: Preflop integration
- v4: Add player profiles (4 tightness levels)

---

### 3. Stat-Based Range Selection (v4)

Instead of fixed profiles, use **4 tightness levels** based on actual stats:

#### Tightness Levels
- **tight**: ~20% tighter than GTO (nits)
- **gto**: Baseline optimal
- **loose**: ~20% wider than GTO (loose players)
- **extra_loose**: ~40% wider than GTO (fish/maniacs)

#### Key Stats for Range Selection

**Priority 1: Position-specific RFI%**
- BTN_RFI, CO_RFI, MP_RFI, UTG_RFI, SB_RFI
- Directly determines opening range width

**Priority 2: Position-specific defense%**
- BB_vs_BTN, BB_vs_CO, BB_vs_MP, etc.
- Directly determines defense range width

**Priority 3: Position-specific 3bet%**
- BB_3bet_vs_BTN, BTN_3bet_vs_CO, etc.
- Determines 3betting range width

#### Exploitative Warnings

**Calling Station Detection**:
- WTSD% >35% AND Fold to Cbet <40%
- Warning: "⚠️ Calling station - Don't bluff"

**Over-Aggressive Detection**:
- Aggression Factor >4 AND Cbet% >75%
- Warning: "⚠️ Over-aggressive - Bluff catch more"

**Sample Size Thresholds**:
- <100 hands: Use GTO ranges only
- 100-300 hands: Show warning with ⚠️ low-confidence
- 300+ hands: Full confidence

#### Stats Tracking

**Data Source**: Hand history files (parsed post-session)

**Essential Stats** (for range selection):
- Position-specific RFI% (UTG, MP, CO, BTN, SB)
- Position-specific defense% (BB vs each position, SB vs BTN/CO)
- Position-specific 3bet%
- Hands played (sample size)

**Exploitative Stats** (for warnings):
- WTSD% (Went To Showdown)
- Fold to Cbet%
- Aggression Factor
- Cbet%

See `ALTERNATIVE_PROFILING_SYSTEM.md` for complete implementation details.

---

### 4. Range Library

**Structure**: Organized by tightness level, then position (Option B)

```
ranges/
├── raw/              # Original GTO Wizard downloads
│   └── gto_wizard_*.txt
│
├── tight/            # ~20% tighter than GTO
│   ├── UTG/ (11 ranges)
│   ├── MP/ (12 ranges)
│   ├── CO/ (13 ranges)
│   ├── BTN/ (14 ranges)
│   ├── SB/ (15 ranges)
│   └── BB/ (15 ranges)
│
├── gto/              # Baseline optimal (v1.1)
│   ├── UTG/ (11 ranges)
│   ├── MP/ (12 ranges)
│   ├── CO/ (13 ranges)
│   ├── BTN/ (14 ranges)
│   ├── SB/ (15 ranges)
│   └── BB/ (15 ranges)
│
├── loose/            # ~20% wider than GTO
│   └── [same structure]
│
└── extra_loose/      # ~40% wider than GTO
    └── [same structure]

Total: 80 ranges × 4 levels = 320 ranges
Focus on top ~40 scenarios = 160 ranges
```

**Runtime selection**: `ranges/{level}/BTN/RFI.txt`
- `level` chosen based on villain's actual stats
- Example: villain_BTN_RFI = 58% → use `loose` level (54-62% range)

See `RANGE_FILE_ORGANIZATION.md` and `ALTERNATIVE_PROFILING_SYSTEM.md` for details.

**Range Format**: Standard PIO format
```
AA:1.0
KK:1.0
QQ:0.85
AKs:1.0
AKo:0.75
```

---

### 5. Storage Architecture

```
stealthproject/
├── data/
│   ├── solutions.db          # 1-3GB, read-heavy
│   │   ├── solutions         # Unique solves (range_key → strategy)
│   │   ├── scenario_mapping  # (scenario, profile, bucket) → range_key
│   │   └── range_index       # Hash → range file metadata
│   │
│   └── player_stats.db       # 10-100MB, read+write
│       ├── players           # player_id, username, stats
│       └── hands             # per-hand history
│
└── ranges/                   # Range library (~250 unique files)
    ├── GTO/ (80 ranges)
    ├── NIT/ (30 unique + 50 symlinks)
    ├── LAG/ (40 unique + 40 symlinks)
    ├── FISH/ (45 unique + 35 symlinks)
    └── MANIAC/ (55 unique + 25 symlinks)
```

**solutions.db**: 
- SQLite on disk with memory-mapped file
- Indexed on `(range_key, bucket_id, street)` for <10ms lookups
- Stores only unique solves, uses scenario_mapping for deduplication

**player_stats.db**: 
- SQLite on disk, persisted across sessions
- Tracks per-player VPIP, PFR, 3bet%, etc.

---

## UI Design

### Overlay Window Specification

**Position**: Semi-transparent overlay anchored above hero's hole cards  
**Technology**: PyQt5, always-on-top window  
**Updates**: Real-time on flop/turn/river detection and villain action

### Layout

```
┌──────────────────────────────────────────────────────────┐
│ Player: fish_Player123                                   │
│ ⚠️ Calling station (WTSD 38%) - Don't bluff              │
│ Using: Loose defense range (52% vs 42% GTO)              │
│ Last action: Raised to $8.40 (84% pot)                   │
│ Hero hand: Ace-high flush                                │
│                                                           │
│ [Fold: 12%] [Call: 58%] [Raise 50%: 22%] [Raise 100%: 8%]│
└──────────────────────────────────────────────────────────┘
```

### Specifications

1. **Player Line**: 
   - Display villain username
   - Sample size indicator if <300 hands

2. **Warning Line** (if applicable):
   - Calling station: WTSD >35% AND Fold to Cbet <40%
   - Over-aggressive: AF >4 AND Cbet >75%
   - Only show if >100 hands sample

3. **Range Selection Line**:
   - Which tightness level being used (tight/gto/loose/extra_loose)
   - Which stat triggered it (e.g., "High RFI at BTN: 58%")
   - Show "GTO vs GTO" if <100 hands

4. **Last Action Line**: 
   - Villain's most recent action
   - Bet size as $ amount and % of pot

5. **Hero Hand Line**: 
   - Current made hand strength
   - Examples: "Ace-high", "Top pair", "Flush draw", "Two pair", "Nut flush"

6. **Action Row**:
   - Always show: **Fold**, **Call/Check**, and **top 2 raise sizes** by frequency
   - Color coding:
     - **Blue square**: Fold
     - **Green square**: Call/Check
     - **Red square**: Raise
   - **Bold** the highest frequency action
   - Format: `[Action BetSize%: Frequency%]`

---

## Development Roadmap

### v1.0: Interactive GTO Solver (Single Flop)
**Goal**: Validate solver pipeline with minimal scope

**Scope**:
- Single scenario: BTN RFI vs BB call (SRP - Single Raised Pot)
- Single flop: `KhQs6h` (two-tone K-high, no straight)
- Pot at flop: **5.5bb** (SB 0.5bb + BB 2.5bb + BTN 2.5bb)
- Effective stack: **97.5bb** (100bb - 2.5bb invested)

**Ranges** (✅ converted):
- BTN (IP): `ranges/gto/BTN/RFI.txt` (93 hands)
- BB (OOP): `ranges/gto/BB/defend_vs_RFI_BTN.txt` (84 hands)

**Solver Config**:
- Bet sizes: Flop [25%, 50%, 100%] | Turn [25%, 50%, 100%, all-in] | River [50%, 100%, 150%, all-in]
- Raise sizes: 2.5x
- Donk sizes: [33%, 50%]
- Rake: 5% ($0.01 per $0.20 in pot) capped at $3.00 (3bb) - ACR 6-max $0.50/$1.00

**Features**:
- ✅ Convert GTO Wizard format to postflop-solver format:
  - Input: `2d2c: 0.3094, 2h2c: 0.3094, ...`
  - Output: `22:0.3094` (generalized hands, line-separated)
- Solve ONE flop: `KhQs6h`
  - BTN_RFI_vs_BB_call scenario only
- Build interactive CLI program:
  - Select hero position (BTN or BB)
  - Input hero's 2 cards (e.g., "As Kd")
  - Display frequency array for hero's hand
  - Input actions (hero or villain, depending on position)
  - Progress through turn/river with action input
- **Examine actual solver output structure**:
  - How does it handle suited vs offsuit?
  - How does it handle flush draws vs made flushes?
  - What's the exact JSON format?

**Deliverables**:
- ✅ Range conversion script (`scripts/convert_range.py`)
- ✅ Rust installed and postflop-solver compiled (fixed compatibility issues with Rust 1.95)
- ✅ 1 solved game tree (Board: KhQs6h 9d 3c, BTN vs BB) - 0.45 MB, exploitability 2.09 cents
- ✅ Interactive CLI (`solver/src/bin/interactive.rs`) - query any hand, see equity/EV/strategy
- ✅ Game tree navigation (play through streets with actions, see strategy at each node)
- ✅ Board card validation (prevents querying hands with cards on the board)
- ✅ Demo documentation (`DEMO_v1.0.md`)

**Success Criteria**:
- ✅ Successfully converts GTO Wizard ranges
- ✅ Solve completes without errors (2.09 cents exploitability, <0.5% target)
- ✅ Can navigate full game tree (river board with all streets playable)
- ✅ Frequencies sum to 1.0 for each decision point
- ✅ Shows equity, EV, and strategy at each node

**Status**: ✅ **COMPLETE**

**Actual time**: ~1 day
- ✅ Range conversion: 2 hours
- ✅ Rust setup + postflop-solver compilation fixes: 3 hours
- ✅ Single solve: 5 seconds runtime
- ✅ Interactive program: 3 hours
- ✅ Testing/validation: 1 hour

---

### v1.1: AWS Pipeline Validation (Single Flop Full Tree)
**Goal**: Validate full tree solving on AWS before scaling to 184 flops

**Scope**:
- **Single flop**: KhQs6h (canonical two-tone K-high)
- **Full tree**: turn NOT_DEALT, river NOT_DEALT (all possible runouts)
- Same scenario: BTN RFI vs BB call
- Same solver config as v1.0 (bet sizes, raise sizes, rake)
- **Compile on EC2** (binary compiled locally won't work - Mac ARM vs Linux x86_64)

**Why this step**:
- Validates full tree solving works (not just locked runouts)
- Tests AWS pipeline (EC2, S3, IAM roles)
- Measures actual memory usage (~30 GB compressed)
- Validates solve quality and convergence
- Tests output format for later parsing
- Provides cost estimate for scaling to 184 flops

**AWS Setup** (one-time):
- IAM role for S3 access
- Security group for SSH
- EC2 key pair
- S3 bucket (poker-solver-kason)

**Workflow**:
1. **Prepare locally**: Push code to GitHub, upload ranges to S3
2. **Launch EC2**: r6a.2xlarge (64 GB RAM, $0.504/hour or $0.15/hour spot)
3. **Inside EC2**: 
   - Install Rust (~3 min)
   - Clone repo (~10 sec)
   - Download ranges from S3 (~10 sec)
   - Compile solver (~5 min)
   - Run solver (~30 min)
   - Upload results to S3 (~10 sec)
4. **Terminate instance**
5. **Download results** from S3

**Deliverables**:
- ✅ Modified solver for full tree (NOT_DEALT)
- ✅ Code pushed to GitHub
- ✅ Ranges uploaded to S3
- ✅ AWS setup documentation (`aws/README.md`)
- ✅ Setup script (`aws/v1.1-setup.sh`)
- ⬜ 1 solved full tree (~30 GB output)
- ⬜ Memory usage validation
- ⬜ Cost validation

**Success Criteria**:
- Solve completes without OOM (out of memory)
- Exploitability < 0.5% of pot
- Output format is parseable
- Total cost ~$0.10-0.35
- Can query strategies for any turn/river combo

**Estimated time**: 2-3 days
- AWS setup: 2-4 hours (one-time)
- First solve attempt: 41 minutes runtime
- Analysis/validation: 2-4 hours
- Documentation: 2 hours

**Estimated cost**: 
- Single run: $0.35 (on-demand) or $0.10 (spot)
- With retries/testing: ~$0.50-1.00 total

---

### v1.2: Complete GTO Database (All Scenarios)
**Goal**: Generate full solution database with AWS parallel solving

**Scope**:
- Build all 72 GTO ranges covering all positions and scenarios
- ~50-60 unique scenarios (SRP, 3-bet, 4-bet pots)
- 150 flop buckets
- **Total**: ~4,000-5,000 unique solves after deduplication
- AWS EC2 fleet for parallel solving (500 cores, 5 hours, ~$75)
- Same solver config as v1.0/v1.1 (bet sizes, raise sizes, donk sizes, rake)

**Pot sizes for different scenarios**:
- SRP (RFI vs call): 5.5bb pot, 97.5bb effective
- 3-bet pots: Varies by position (e.g., BB 3-bets BTN to 10bb → pot ~21bb)
- 4-bet pots: Varies (e.g., BTN 4-bets to 25bb → pot ~51bb)

**Deliverables**:
- 72 GTO range files (converted from GTO Wizard)
- AWS solving scripts (queue generation, worker, monitoring)
- Local SQLite database (~50-150 GB)
- Enhanced interactive program supporting all scenarios

**Success Criteria**:
- All ranges successfully uploaded to S3
- AWS solving completes with <1% failures
- Database correctly stores all game trees
- Can query any scenario/flop combination

**Estimated time**: 2-3 weeks
- Range building: 40-60 hours
- AWS setup: 8-16 hours
- Solving: 5 hours wall time
- Database processing: 8-16 hours

---

### v2.0: OCR Board Cards
**Goal**: Validate screen reading of flop cards

**Features**:
- Implement OCR for board card recognition (rank + suit)
- Output results to test file with color-coded suits
- No overlay yet, console/file output only

**Success Criteria**:
- 95%+ card recognition accuracy
- Correctly identifies rank (A-2) and suit (♠♥♦♣)
- Color formatting for suits in output

**Estimated time**: 1 week

---

### v2.01: OCR Hero Cards
**Goal**: Extend OCR to read hero's hole cards

**Features**:
- Detect and OCR hero's 2 cards
- Output to test file alongside board cards
- Validate card positioning detection

**Success Criteria**:
- 95%+ accuracy on hero cards
- Correctly handles card overlap/positioning

**Estimated time**: 1 week

---

### v2.1: OCR Action (Preflop + Postflop)
**Goal**: Read all actions from preflop through river

**Features**:
- **Preflop tracking** (critical for range selection):
  - Detect player positions (BTN, SB, BB, CO, MP, UTG)
  - Detect who opened (position + amount)
  - Detect 3bets, 4bets, calls
  - Identify preflop scenario: "BTN_RFI_vs_BB_call", "CO_RFI_vs_BB_3bet_CO_call", etc.
  - Track hero position vs villain position
- **Postflop tracking**:
  - OCR pot size
  - OCR bet/raise amounts
  - Track action sequence (check, bet, call, raise, fold)
  - Detect whose turn it is
- Output full hand sequence to test file (preflop → river)

**Testing**:
- Run on 30+ hands from ACR (mix of SRP, 3bet, 4bet pots)
- Compare OCR output with actual hand history files line-by-line
- Validate preflop scenario identification accuracy
- Measure action tracking accuracy

**Success Criteria**:
- 95%+ accuracy on preflop scenario identification
- 90%+ accuracy on pot/bet size (within $0.10)
- Correctly sequences all actions (preflop through river)
- <5% missed actions
- Output matches hand history format

**Estimated time**: 2-3 weeks

---

### v2.2: Anchored Overlay with Hardcoded Text
**Goal**: Build overlay window with dummy data

**Features**:
- PyQt5 transparent overlay window
- Always-on-top, anchored above hero cards
- Displays hardcoded frequencies (not real solver data)
- Tracks table window movement

**Success Criteria**:
- Overlay anchors correctly on 2+ tables
- Maintains position when table window moves
- No performance impact on ACR client

**Estimated time**: 1 week

---

### v2.21: Overlay Dynamic Data (Connect to Solver)
**Goal**: Display real solver frequencies in overlay

**Features**:
- Integrate OCR output with game state manager
- Game state manager queries solution database
- Show villain username in overlay
- Display last action with bet size
- Show hero's current hand strength (using hand evaluator)
- Display real GTO frequencies from solver
- Update overlay in real-time as action progresses

**Success Criteria**:
- Overlay updates <100ms after OCR completes
- Hand strength evaluation correct (pair, flush, etc.)
- Last action display matches actual game state
- Frequencies match solver output for given game state

**Estimated time**: 1-2 weeks

---

### v2.22: Multiple Table Support
**Goal**: Support 2-4 simultaneous tables

**Features**:
- Enumerate all open ACR windows (macOS Accessibility API)
- Independent OCR thread per table
- Independent overlay window per table
- Track window positions continuously (background thread)
- Match tables to hand history files via table name

**Success Criteria**:
- Correctly handles 2-4 tables simultaneously
- Overlays anchor correctly when windows move
- No performance degradation (<500ms latency maintained)
- CPU usage <30% with 4 tables

**Estimated time**: 1 week

---

### v3: Preflop Display Integration
**Goal**: Add preflop decision support

**Features**:
- Download/generate preflop ranges (GTO Wizard or custom)
- Preflop actions: Open, 3bet, 4bet, defend ranges
- Detect pot type from preflop action (SRP vs 3-bet vs 4-bet)
- Display preflop frequencies before flop appears
- Auto-load correct postflop tree based on preflop action

**Deliverables**:
- Preflop range database
- Preflop overlay display
- Pot type detection logic
- Seamless preflop → postflop transition

**Estimated time**:
- Preflop range acquisition: 4-8 hours
- Integration: 8-16 hours

---

### v4: Stat-Based Exploitative Ranges
**Goal**: Add exploitative strategies based on villain's actual stats

**Features**:
- Build 4 tightness levels: tight, gto, loose, extra_loose
- Focus on top ~40 scenarios = 160 ranges (40 × 4 levels)
- Generate solutions for all tightness levels × all flop buckets
- **Total**: 150 flop buckets × ~50 range pairs × 4 levels = **30,000 solves**
- With hash-based deduplication: **~12,000-15,000 unique solves**
- Build player_stats.db with hand history parser
- Implement stat-based range selection algorithm
- Add exploitative warnings (calling station, over-aggressive)

**Stats Tracked**:
- Position-specific RFI% (UTG, MP, CO, BTN, SB)
- Position-specific defense% (BB vs each position)
- Position-specific 3bet%
- WTSD%, Fold to Cbet%, AF, Cbet% (for warnings)

**Range Selection Logic**:
- villain_BTN_RFI = 58% → use `loose/BTN/RFI.txt` (54-62% range)
- villain_BB_defense = 32% → use `tight/BB/vs_RFI_BTN.txt` (28-36% range)
- <100 hands → always use `gto/` ranges

**Deliverables**:
- 160 range files (tight/gto/loose/extra_loose × ~40 scenarios)
- solutions.db with all exploitative solutions
- Hand history parser → player stats database
- Stat-based range selector
- Overlay shows: warnings + which range being used + why

**Estimated time**:
- Range building: 20-30 hours (scripting + validation)
- Solving: 12,000 solves × 20 min = 4,000 hours compute
  - Parallelized (500 cores): **8 hours wall time**
  - AWS cost: 500 cores × 8 hours × $0.03 = **~$120**

See `ALTERNATIVE_PROFILING_SYSTEM.md` for complete implementation.

---

## Technical Considerations

### Preflop Action Tracking (Critical)

**Required for range selection**: Must track complete preflop action to identify scenario.

**Examples**:
- BTN opens, BB calls → "BTN_RFI_vs_BB_call" (SRP)
- CO opens, BTN 3bets, CO calls → "CO_RFI_vs_BTN_3bet_CO_call" (3bet pot)
- MP opens, BB 3bets, MP 4bets, BB calls → "MP_RFI_vs_BB_3bet_MP_4bet_BB_call" (4bet pot)

**Implementation**: OCR tracks all preflop actions and maps to scenario string, which determines:
1. Which range pair to load
2. Whether hero is IP or OOP
3. Pot type (SRP, 3bet, 4bet)

### OOP vs IP Position Handling

**Automatically handled by action tracking**: 
- v1.1 OCR tracks whose turn it is
- If hero's turn → display frequencies
- If villain's turn → display "waiting..."
- Action tracking (like hand history) inherently handles OOP/IP

See `OOP_IP_HANDLING.md` for complete implementation details.

### Multi-Table Support
- Enumerate ACR windows via macOS Accessibility API
- Match each window to table via window title
- Track window positions in background thread
- Independent overlay per table

### Hand Strength Evaluation
- Use poker hand evaluator library (e.g., `pokerkit`, `treys`)
- Evaluate hero's 2 cards + board cards
- Display: "High card", "Pair", "Two pair", "Trips", "Straight", "Flush", "Full house", "Quads", "Straight flush"
- Add specificity: "Ace-high flush", "Top pair", "Bottom two pair"

### Error Handling
- OCR failure: Display "Waiting for clear image..." on overlay
- Table window closed: Destroy corresponding overlay
- Database connection failure: Log error, fallback to cached solutions

### Performance Optimization
- Preload top 50 flop solutions when preflop action detected
- Index solutions.db on `(bucket_id, profile, street, position)`
- Memory-mapped SQLite for faster reads
- Cache last 10 lookups in memory

---

## Open Questions & Future Enhancements

1. **Turn/River solving**: Full game tree or simplified?
2. **ICM considerations**: Tournament mode (future)
3. **Range visualization**: Show villain's estimated range distribution
4. **EV calculations**: Display $ EV difference between actions
5. **Multi-way pots**: 3-player scenarios (currently excluded)
6. **Live adjustments**: Manual profile override for specific opponents
7. **Session analytics**: Track hero's actual vs recommended actions

---

## Legal & Compliance
_To be addressed in implementation phase. This tool is for personal use only._

---

**End of PRD**
