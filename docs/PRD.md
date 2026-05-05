# PRD: GTO Poker Solver with Real-Time Overlay

**Goal**: Read ACR 6-max $0.50/$1.00 NLHE tables and display optimal GTO strategy using pre-solved game trees

**Stack**: 100bb effective stacks, 5% rake capped at $3.00

---

## System Architecture

Four independent components that integrate into one working system:

1. **Board Reader** - OCR to capture game state from screen
2. **Converter** - Map game state to solver inputs/outputs
3. **Solver** - Generate 6,992 pre-solved game trees (38 scenarios × 184 flops)
4. **Overlay** - Display recommended actions in real-time

Each component has versioned milestones for isolated testing.

---

## 1. Board Reader

**Purpose**: Monitor ACR client and extract game state via OCR

### Responsibilities
- Detect when hole cards are dealt (new hand start)
- Detect board updates (flop/turn/river appearance)
- Read board cards (rank + suit)
- Read hero's hole cards
- Track preflop action (positions, folds, calls, raises with amounts)
- Track postflop action (checks, bets, calls, raises, folds)
- Detect whose turn to act (button highlighting)
- Read pot size and stack sizes

### Technology
- **Screen capture**: Pillow (macOS screen regions)
- **OCR**: pytesseract or EasyOCR
- **Pixel monitoring**: 60fps for action triggers
- **Multi-table**: Enumerate windows via macOS Accessibility API

### Versions

**v1.0**: Single table, static flop detection
- Capture screenshot on demand
- OCR board cards (3 cards)
- OCR hero hole cards (2 cards)
- Output: raw strings like "KhQd6h" and "AhKd"

**v1.1**: Automatic triggers
- Pixel monitoring for flop appearance
- Auto-capture when board changes
- Detect whose turn to act

**v1.2**: Preflop tracking
- OCR player positions (UTG, MP, CO, BTN, SB, BB)
- Track action sequence (fold/call/raise amounts)
- Output action log for converter

**v1.3**: Full postflop tracking
- Track all postflop actions (check/bet/raise amounts)
- Track pot size updates
- Output complete action tree

**v2.0**: Multi-table support
- Monitor 2-4 tables simultaneously
- Independent tracking per table
- <30% CPU with 4 tables

---

## 2. Converter

**Purpose**: Bridge between Board Reader and Solver

### Responsibilities

**Preflop**:
- Parse action sequence from Board Reader
- Map to one of 38 preflop scenarios
- Load appropriate range file
- Output recommended preflop action to Overlay
- Determine final heads-up pair (OOP position, IP position)

**Postflop**:
- Map actual flop → canonical flop (two-step process)
- Map hero hand using same suit mapping
- Load correct .bin file: `{scenario}__{canonical_flop}.bin`
- Map villain's action to nearest solver node
- Query solver for hero's strategy
- Return action frequencies to Overlay

### Board Mapping Algorithm

Maps any of ~22,000 real flops to one of 184 canonical flops using two steps:

**Step 1: Suit Isomorphism** (`suit_mapper.rs`)
- Canonicalize suits only (ranks unchanged)
- **Monotone**: All spades (e.g., `AsKsQs`)
- **Two-tone**: Spades + hearts (e.g., `AsKsQh`)
- **Rainbow**: Spades, hearts, diamonds (e.g., `AsKhQd`)

**Step 2: Similarity Matching** (`flop_similarity.rs`)
- If exact match exists in 184 → use it
- Otherwise find closest match using weighted scoring:
  - **High card**: 0 ranks apart = +100, 1 apart = +80, 2 apart = +50
  - **Mid card**: 0 apart = +50, 1 apart = +30, 2 apart = +15
  - **Low card**: 0 apart = +25, 1 apart = +15, 2 apart = +8
  - **Trips/paired match**: Same = +80/+60
  - **Connectivity**: Same = +40, 1 off = +20
  - **Suit pattern**: Same = +20
- Scores all 184 candidates, returns highest
- Always returns best match (never fails)

**Example**:
- Actual: `KhQd6h` → Suit canonicalize: `KsQh6s`
- Check if `KsQh6s` exists in 184 → if yes, use it
- If not → find closest (e.g., `KsQh5s` or `KsJh6s`)
- Hero: `AhKd` → Canonical: `AsKh` (same suit mapping)
- Load: `BTN_RFI_vs_BB_defend___KsQh6s.bin`
- Query: `AsKh`

### Versions

**v1.0**: Static lookup
- Hardcode one scenario
- Input: flop string, hand string
- Output: canonical flop, canonical hand, filename

**v1.1**: Scenario mapping
- Parse 38 preflop scenarios from action logs
- Map to correct range files
- Output preflop recommendations

**v1.2**: Action tree navigation
- Track postflop action sequence
- Navigate solver tree to correct node
- Handle bet sizing approximations

**v2.0**: Full integration
- Real-time converter running in background
- Sub-500ms latency from OCR to solver query

---

## 3. Solver

**Purpose**: Generate 6,992 pre-solved game trees

### Database Specs

**Coverage**:
- **38 preflop scenarios** (from 72 ranges)
- **184 canonical flops**
- **Total**: 6,992 .bin files (~27 TB)

### 38 Preflop Scenarios

All scenarios end in a **call** (heads-up postflop). Derived from 72 ranges = 36 range pairs.

**RFI vs Defend (5 scenarios)**:
1. UTG_RFI_vs_BB_defend
2. MP_RFI_vs_BB_defend
3. CO_RFI_vs_BB_defend
4. BTN_RFI_vs_BB_defend
5. SB_RFI_vs_BB_defend

**RFI vs Cold Call (3 scenarios)**:
6. UTG_RFI_vs_BTN_cold_call
7. MP_RFI_vs_BTN_cold_call
8. CO_RFI_vs_BTN_cold_call

**3-Bet Pots (15 scenarios)** - `{3bettor}_3bet_vs_{opener}_call`:
9. MP_3bet_vs_UTG_call
10. CO_3bet_vs_UTG_call
11. BTN_3bet_vs_UTG_call
12. SB_3bet_vs_UTG_call
13. BB_3bet_vs_UTG_call
14. CO_3bet_vs_MP_call
15. BTN_3bet_vs_MP_call
16. SB_3bet_vs_MP_call
17. BB_3bet_vs_MP_call
18. BTN_3bet_vs_CO_call
19. SB_3bet_vs_CO_call
20. BB_3bet_vs_CO_call
21. SB_3bet_vs_BTN_call
22. BB_3bet_vs_BTN_call
23. BB_3bet_vs_SB_call

**4-Bet Pots (15 scenarios)** - `{4bettor}_4bet_vs_{3bettor}_call`:
24. UTG_4bet_vs_MP_call
25. UTG_4bet_vs_CO_call
26. UTG_4bet_vs_BTN_call
27. UTG_4bet_vs_SB_call
28. UTG_4bet_vs_BB_call
29. MP_4bet_vs_CO_call
30. MP_4bet_vs_BTN_call
31. MP_4bet_vs_SB_call
32. MP_4bet_vs_BB_call
33. CO_4bet_vs_BTN_call
34. CO_4bet_vs_SB_call
35. CO_4bet_vs_BB_call
36. BTN_4bet_vs_SB_call
37. BTN_4bet_vs_BB_call
38. SB_4bet_vs_BB_call

**Pot sizes**:
- SRP: 5.5bb pot, 97.5bb stacks
- 3-bet: 21bb pot, 79bb stacks
- 4-bet: 51bb pot, 49bb stacks

### 184 Canonical Flops

Generated by `generate_boards.py` with systematic coverage:

**Rank distributions**:
- High cards (A, K, Q, J, T)
- Mid cards (9, 8, 7)
- Low cards (6, 5, 4, 3, 2)

**Textures**:
- Connected (e.g., `JsTs9s`)
- One-gap (e.g., `Js9s8s`)
- Two-gap (e.g., `AsTs2s`)
- Paired (e.g., `KsKsQs`)
- Trips (e.g., `AsAsAs`)

**Suit patterns**:
- Monotone (all spades)
- Two-tone (spades + hearts)
- Rainbow (spades, hearts, diamonds)

### Solution File Format

Each `.bin` file contains a **complete game tree** with GTO strategies:

**Structure**:
- **Card configurations**: OOP range, IP range, board cards
- **Action tree**: All possible betting sequences (flop → turn → river)
- **Strategy tables**: For each node, stores mixed strategy for every hand
- **Equity data**: Showdown equity for every hand combination

**File size**: ~3.9 GB per solution (16-bit compression)

**What gets stored**:
- Every decision node in the game tree
- For each node: available actions (check, bet $X, raise to $Y, fold, call)
- For each hand in range: probability distribution over actions
- Example: `AhKd` at root → `{Check: 0.15, Bet(275¢): 0.65, Bet(550¢): 0.20}`

### Querying Solutions

**Navigation**:
1. Load `.bin` file → get `PostFlopGame` object
2. Start at root node (first action on flop)
3. Query strategy for specific hand → get action frequencies
4. Apply action → navigate to child node
5. Repeat for turn/river

**Example query flow**:
```rust
// Load solution
let game = load("BTN_RFI_vs_BB_defend___AsKsQs.bin");

// Root node - OOP first to act
let strategy = game.strategy();  // All hands, all actions
let hand_idx = find_hand("AhKd");
let actions = game.available_actions();  // [Check, Bet(275), Bet(550)]

// Get frequencies for AhKd
println!("Check: {}", strategy[hand_idx + 0 * num_hands]);
println!("Bet 50%: {}", strategy[hand_idx + 1 * num_hands]);
println!("Bet 100%: {}", strategy[hand_idx + 2 * num_hands]);

// Apply action: OOP checks
game.apply_action(0);  // Action index 0 = Check

// Now IP acts
let ip_strategy = game.strategy();
// ... query IP's strategy
```

**Key operations**:
- `game.available_actions()` → list of legal actions at current node
- `game.strategy()` → flattened array of frequencies
- `game.apply_action(idx)` → move to child node
- `game.equity(player)` → equity for all hands
- `game.back_to_root()` → reset to start

### Solver Configuration

**Bet sizes**:
- Flop: 50%, 100% | Raises: 3x, 5x
- Turn: 50%, 100%, 150% | Raises: 3x, 5x
- River: 75%, 150%, all-in | Raises: 3x, 5x
- Donk bets: 50%

**Settings**:
- Precision: 16-bit compressed
- Target exploitability: <5¢ (<1% pot)
- Max iterations: 500
- Rake: 5% capped at $3

### Architecture Pivot: Hybrid Database Approach

**Initial plan** was to store all 6,992 complete game tree solutions as `.bin` files:
- 6,992 scenarios × 3.9 GB per file = **27 TB storage**
- Local storage: 4× 8TB HDDs = $400
- Cost seemed manageable

**Problem 1: Loading time too slow**
- Each query requires loading 3.9 GB into RAM
- Even with fast HDDs, this takes several seconds per lookup
- Real-time overlay requires <500ms latency
- **Conclusion**: Pre-solved bins too large for fast runtime access

**Pivot 1: Extract everything to database**
- Attempted to extract complete game tree to SQLite
- Store every node's strategy (flop + turn + river)
- **Problem**: Still massive database size per scenario
- Multiplied by 6,992 scenarios = still storage-prohibitive

**Final solution: Hybrid approach**
- **Extract flop strategies only** to SQLite database
  - All flop decision nodes until call/fold (before turn card dealt)
  - ~7 MB per scenario (vs 3.9 GB bin file)
  - 6,992 × 7 MB ≈ **50 MB total** (vs 27 TB)
- **Solve turn/river live** when needed
  - Filter ranges based on flop action sequence using Bayesian inference:
    ```
    P(hand | action sequence) ∝ P(action sequence | hand) × P(hand)
    ```
  - Weighted ranges: multiply hand frequency by each action's strategy
  - Build new `PostFlopGame` with filtered ranges + observed turn card
  - Solve turn + all 44 rivers
  - **Performance**: 0.15s - 0.88s for complete turn/river solve
  - **Accuracy**: Flop perfect (from DB), turn/river within ~10% (acceptable tradeoff)

**Benefits**:
- **Storage**: 27 TB → 50 MB (99.99% reduction)
- **Latency**: Flop instant (<10ms DB lookup), turn/river <1s (live solve)
- **Total response**: <1s end-to-end (well under 500ms target for flop-only)
- **Accuracy**: Negligible difference for practical use

**Implementation**:
- `extract_v11_flop_bfs.rs` - Extracts flop strategies via BFS traversal
- `interactive_db.rs` - Queries DB + solves turn/river with filtered ranges
- Database schema: `(scenario, board, hand, position, action_history, action, frequency, equity)`

**Live solve bet sizes** (turn/river):
- **Turn**: 50%, 100%, 150%, all-in | Raises: 3x, 5x
- **River**: 33%, 75%, 125%, 200%, all-in | Raises: 3x, 5x
- Optimized sizing for balance between accuracy and solve speed

### Range Organization

```
ranges/
├── gto_wizard/          # Raw downloads from GTO Wizard
│   ├── rfi/             # RFI vs Defend/Cold Call (8 scenarios)
│   │   ├── UTG_RFI_vs_BB_defend/
│   │   │   ├── utg.txt       # UTG opening range
│   │   │   └── bb.txt        # BB defend range
│   │   ├── MP_RFI_vs_BB_defend/
│   │   ├── CO_RFI_vs_BB_defend/
│   │   ├── BTN_RFI_vs_BB_defend/
│   │   ├── SB_RFI_vs_BB_defend/
│   │   ├── UTG_RFI_vs_BTN_cold_call/
│   │   ├── MP_RFI_vs_BTN_cold_call/
│   │   └── CO_RFI_vs_BTN_cold_call/
│   ├── 3bet/            # 3-bet pots (15 scenarios)
│   │   ├── UTG_RFI_vs_MP_3bet_UTG_call/
│   │   │   ├── utg.txt       # UTG calling range
│   │   │   └── mp.txt        # MP 3-betting range
│   │   └── ...
│   └── 4bet/            # 4-bet pots (15 scenarios)
│       ├── UTG_vs_MP_3bet_UTG_4bet_MP_call/
│       │   ├── utg.txt       # UTG 4-betting range
│       │   └── mp.txt        # MP calling vs 4bet range
│       └── ...
└── gto/                 # Converted ranges for solver (same structure)
    ├── rfi/
    ├── 3bet/
    └── 4bet/
```

### File Naming

**Solution files**:
```
{scenario}___{canonical_flop}.bin
```

**Examples**:
- `BTN_RFI_vs_BB_defend___AsKsQs.bin`
- `BTN_3bet_vs_CO_call___9s8s7s.bin`
- `UTG_4bet_vs_MP_call___KsKsQs.bin`

### AWS Pipeline

**Phase 1**: Generate boards
```bash
python3 generate_boards.py > boards.txt
# Output: 184 canonical flops
```

**Phase 2**: Create job queue
```bash
# Generate 6,992 job specs: scenario × flop combinations
./create_job_queue.sh
```

**Phase 3**: Batch solve
```bash
./queue_manager.sh jobs.txt 50

# Launches 50 r6a.2xlarge spot instances in parallel
# Each solves one scenario+flop combination (~19 min)
# Auto-retries interrupted spots (adds back to queue)
# Uploads to S3 → downloads immediately → deletes from S3
```

**Phase 4**: Local storage
- Download all .bin files to local drives
- 4× 8TB HDDs = $400 (vs $621/month S3)

**Cost**: ~$358 total (6,992 × $0.048 + retries)  
**Time**: 3-4 days with 50 parallel instances

### Versions

**v1.0**: ✅ Complete
- Single scenario: `BTN_RFI_vs_BB_defend`
- Single flop: `KhQs6h` with predetermined turn and river
- CLI tool for testing

**v1.1**: ✅ Complete
- Same flop (`KhQs6h`) full game tree
- Navigated through game tree and compared results with GTO Wizard
- Generated on EC2 instance
- 3.9 GB per solution
- 18.9 min solve

**v1.2**: In Progress
- Generate all 184 canonical flops
- Define all 38 scenarios
- Build job queue (6,992 jobs)
- Solve on AWS spots

**v2.0**: Future
- All 6,992 solutions complete
- Stored locally (27 TB)
- Ready for runtime lookup

---

## 4. Overlay

**Purpose**: Display GTO strategy in real-time during play

### Window Specs

- **Technology**: PyQt5 semi-transparent window
- **Position**: Always-on-top, anchored above hero's hole cards
- **Tracking**: Follows table window movement
- **Updates**: Real-time as action progresses

### Display Layout

```
┌──────────────────────────────────────────────────────────┐
│ Board: Kh Qd 6h                                          │
│ Hand: Ah Kd (Top Pair, Top Kicker)                      │
│ Scenario: BTN RFI vs BB defend                          │
│ Pot: $5.50 | To act: You (OOP)                          │
│                                                           │
│ GTO Strategy:                                            │
│ [Check: 15%]  [Bet $2.75 (50%): 65%]  [Bet $5.50: 20%] │
└──────────────────────────────────────────────────────────┘
```

### Display Elements

1. **Board cards** - Current flop/turn/river
2. **Hero hand** - Your cards + hand strength label
3. **Scenario** - Which of 38 preflop scenarios
4. **Pot info** - Current pot, whose turn
5. **GTO frequencies** - All actions with percentages
6. **Highlighting** - Bold the highest frequency action

### Versions

**v1.0**: Static overlay
- Display hardcoded strategy
- Test window positioning and transparency
- Test always-on-top behavior

**v1.1**: Dynamic updates
- Receive strategy from Converter
- Update display in real-time
- Format bet sizes and percentages

**v1.2**: Preflop ranges
- Display preflop recommendations from range files
- Show recommended action with frequency

**v1.3**: Multi-table
- Independent overlay per table (2-4 tables)
- Track table windows independently
- <500ms latency per table

**v2.0**: Full integration
- Complete OCR → Converter → Solver → Overlay pipeline
- Sub-500ms end-to-end latency
- Stable multi-table support

---

## Integration Milestones

**M1**: Board Reader v1.0 + Converter v1.0 ❌
- Manual flop entry → canonical mapping
- Status: Not started

**M2**: Converter v1.0 + Solver v1.1 ❌ Not complete
- Load single .bin file and query hand
- ✅ `lookup_solution.rs` queries `v1.1_KhQs6h.bin`
- ❌ Suit isomorphism not implemented (code exists but unused, v1.1 bin is non-canonical)

**M3**: Converter v1.2 + Overlay v1.1 ❌
- Navigate game tree, display strategy
- Status: Not started

**M4**: Board Reader v1.3 + Full System ❌
- Automatic OCR → Overlay display
- Status: Not started

**M5**: All v2.0 components ❌
- Production-ready multi-table system
- Status: Not started

---

## Current Status

- **Solver**: v1.1 complete, v1.2 in progress
  - ✅ Single board validated (3.8GB, 18.9min solve)
  - ✅ 38 scenarios defined
  - ✅ 184 canonical flops generated
  - ✅ Range structure reorganized
  - ✅ `batch_solver.rs` created
  - ❌ Job queue not built
  - ❌ 6,992 solves not run

- **Converter**: v1.0 partial
  - ⚠️ Suit isomorphism code exists (`suit_mapper.rs`) but untested/unused
  - ✅ Solution loader (`lookup_solution.rs`)
  - ❌ Scenario mapping from preflop action

- **Board Reader**: Not started

- **Overlay**: Not started

**Next Steps**:
1. Build job queue for 6,992 solves (Solver v1.2)
2. Create AWS batch solving pipeline
3. Start Converter v1.1 (scenario mapping)
4. Start Board Reader v1.0 (static OCR)
5. Start Overlay v1.0 (static display)
