# Fixed Suit Mapping Bug

**Date:** 2026-05-04

## The Bug

All our binary tools had incorrect suit mapping in `card_to_string()`:

**WRONG (old):**
```rust
let suit = match card % 4 {
    0 => 's', 1 => 'h', 2 => 'd', 3 => 'c',
```

**CORRECT (new):**
```rust
let suit = match card % 4 {
    0 => 'c', 1 => 'd', 2 => 'h', 3 => 's',
```

## Impact

This caused display confusion:
- What we CALLED "JhTh" was actually being shown for "JdTd"  
- Cards shown with wrong suits (clubs shown as spades, etc.)
- **Equity calculations were always correct** - just the display labels were wrong!

## Fixed Files

- ✅ `explore.rs`
- ✅ `diagnose_equity.rs`
- ✅ `find_jt.rs`
- ✅ `list_hands.rs`
- ✅ `test_explore_equity.rs`
- ✅ `test_ec2_solution.rs`
- ✅ `query_solution.rs`

## Verification

After fix:
- JhTh (OOP) = Hand #365 = 62.9% ✓
- JhTh (IP) = Hand #416 = 65.7% ✓
- Board display now correct: Kh Qs 6h ✓

## Test Commands

```bash
# Verify equity is correct
cargo run --release --bin diagnose_equity

# Find JhTh hand number
cargo run --release --bin find_jt

# Use explore tool
cargo run --release --bin explore
# Then select hand #365 for JhTh
```

## Hand Reference (OOP at Root)

Some key hands for testing:
- **365. Th Jh** (JhTh suited) - 62.9%
- **175. 7c 7h** (77 offsuit) - 43.9%
- **335-352. Various JT** (offsuit combos) - 43-47%

---

**Status:** ✅ All fixed and verified
