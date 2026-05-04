# v1.0 Demo - Interactive GTO Solver

## Quick Start

```bash
cd solver
cargo run --release --bin interactive
```

## What It Does

- Solves a complete poker hand: **BTN RFI vs BB call on board Kh Qs 6h 9d 3c**
- Query any hand to see GTO strategy, equity, and EV
- Navigate game tree street by street

## Example Session

### 1. Query a Hand

**Input:** Position BB, Hand 8s7s

**Output:**
```
Position: BB (OOP)
Hand: 8c 7c
Board: Kh Qs 6h 9d 3c

💰 Equity: 1.61%
💵 EV: $-0.02

📊 GTO Strategy for BB (OOP) (YOUR TURN)
Check                 84.36%  ██████████████████████████████████
Bet(275)              15.64%  ███████
```

**Interpretation:** With 87s (missed everything), BB should mostly check (84%) and occasionally bluff (16%).

---

### 2. Query a Strong Hand

**Input:** Position BB, Hand KdTc (top pair on river)

**Output:**
```
Position: BB (OOP)
Hand: Kd Tc
Board: Kh Qs 6h 9d 3c

💰 Equity: 92.63%
💵 EV: $5.99

📊 GTO Strategy for BB (OOP) (YOUR TURN)
Check                 46.04%  ███████████████████████
Bet(275)              53.96%  ██████████████████████████
```

**Interpretation:** With top pair (Kings), BB should bet ~54% for value and check ~46% for balance.

---

### 3. Navigate Game Tree

Select option 2 "Navigate game tree" after querying a hand.

**Example: BB checks with 87s, then BTN bets**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
👤 BB (OOP) to act
💰 Your Equity: 1.61% | EV: $-0.02

📊 Your GTO Strategy:
   1. Check                 84.36%
   2. Bet(275)              15.64%

Choose action: 1  ← Check

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
👤 BTN (IP) to act
💰 Your Equity: 1.61% | EV: $0.00

⏳ Waiting for BTN (IP) to act

Choose action: 2  ← BTN bets $2.75 (50% pot)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
👤 BB (OOP) to act
💰 Your Equity: 1.61% | EV: $-0.45

📊 Your GTO Strategy:
   1. Fold                  92.15%  ← GTO says fold 92% with air
   2. Call                   7.85%
   3. Raise(688)             0.00%
```

---

## Technical Details

**Solver configuration:**
- Pot: 5.5bb ($5.50)
- Stack: 97.5bb ($97.50)
- Rake: 5% capped at $3
- Bet sizes: Flop [25%, 50%, 100%], Turn [25%, 50%, 100%, all-in], River [50%, 100%, 150%, all-in]
- Raise sizes: 2.5x
- Exploitability: 2.09 cents (0.4% of pot)

**Memory:** 0.45 MB (solved with specific turn/river runout)

**Solve time:** ~5 seconds (80 iterations)

---

## Limitations (v1.0)

- ✅ Single board runout (KhQs6h 9d 3c)
- ✅ Cannot save/load solutions (bincode disabled)
- ✅ Navigation is forward-only (can't go back one step, only restart)
- ✅ Shows strategy for current player only

---

## Next: v1.1

- Solve 150 flop buckets (all board textures)
- AWS batch solving
- Save/load solutions
- Query any board, not just this one runout
