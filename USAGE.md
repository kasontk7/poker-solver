# Using the Poker Solver

## Interactive Tree Explorer

Explore solved game trees interactively with full navigation:

```bash
cd solver
cargo run --release --bin explore
```

**Features:**
- ✓ Choose your position (OOP/IP)
- ✓ Select hand from matrix display
- ✓ Navigate the full game tree
- ✓ Control villain actions
- ✓ View GTO strategies and equity at each node
- ✓ Restart or quit anytime

**Commands during exploration:**
- `1-N` - Choose action number
- `r` - Restart to root
- `q` - Quit

---

## Quick Hand Query

Query a specific hand without full navigation:

```bash
cd solver
cargo run --release --bin query_solution
```

Enter hands like `AsAd`, `KdKc`, `Jh9h` etc.

---

## Test EC2 Solution

Verify a downloaded solution works:

```bash
cd solver
cargo run --release --bin test_ec2_solution
```

---

## Running Solves

### Local solve (testing):
```bash
cd solver
cargo run --release --bin poker_solver
```

### EC2 solve (production):
See `aws/LAUNCH.md` for full instructions.

---

## File Locations

- **Solutions:** `~/personal/poker-solver/solutions/`
- **Source code:** `~/personal/poker-solver/solver/src/`
- **Ranges:** `~/personal/poker-solver/ranges/`

---

## Solution Files

Current v1.1 solution:
- **File:** `solutions/v1.1_KhQs6h.bin` (2.3 GB)
- **Board:** Kh Qs 6h (full tree)
- **Scenario:** BTN RFI vs BB defend
- **Size:** 442 OOP hands, 532 IP hands
- **Exploitability:** ~5¢ (<1% of pot)

---

## Tips

**Best hands to test:**
- Pocket pairs: 99, 88, 77
- Suited connectors: J9s, T9s, 87s
- Broadway: KQ, QJ, JT
- Ax suited: A5s, A4s

**Understanding GTO output:**
- Check 100% = Always check
- Bet 50% = Bet half the time, check half
- Multiple actions = Mixed strategy

**Navigation tips:**
- Always check equity to see how strong your hand is
- Compare your equity to your strategy (bluffs have low equity)
- High equity hands with bet frequency = value betting
- Low equity hands with bet frequency = bluffing
