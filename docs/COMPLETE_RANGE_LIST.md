# Complete Range List

## Organization Structure

Ranges are organized by **tightness level** first, then **position**:

```
ranges/
├── tight/          # ~20% tighter than GTO
├── gto/            # Baseline optimal
├── loose/          # ~20% wider than GTO
└── extra_loose/    # ~40% wider than GTO
```

Each tightness level contains the same 80 position-specific ranges.

---

## Range Naming Convention

**File path format**: `ranges/{level}/{position}/{action}.txt`

**Examples**:
- `ranges/gto/BTN/RFI.txt` - GTO BTN open raise
- `ranges/loose/BB/defend_vs_RFI_BTN.txt` - Loose BB defense vs BTN
- `ranges/tight/MP/3bet_vs_UTG.txt` - Tight MP 3bet vs UTG

**Action types**:
- `RFI.txt` - Open raise
- `defend_vs_RFI_{position}.txt` - Call against open (defense)
- `cold_call_vs_{position}.txt` - Call with caller(s) already in
- `3bet_vs_{position}.txt` - 3bet vs open
- `call_vs_3bet_by_{position}.txt` - Call vs 3bet after opening
- `4bet_vs_{position}_3bet.txt` - 4bet after opponent 3bets your open
- `call_vs_4bet_by_{position}.txt` - Call vs 4bet after you 3bet

---

## UTG (Under the Gun) - 11 ranges

1. `RFI.txt` - Open raise
2. `call_vs_3bet_by_MP.txt` - Call MP 3bet **[RARE]**
3. `call_vs_3bet_by_CO.txt` - Call CO 3bet
4. `call_vs_3bet_by_BTN.txt` - Call BTN 3bet
5. `call_vs_3bet_by_SB.txt` - Call SB 3bet **[RARE]**
6. `call_vs_3bet_by_BB.txt` - Call BB 3bet
7. `4bet_vs_MP_3bet.txt` - 4bet vs MP **[VERY RARE]**
8. `4bet_vs_CO_3bet.txt` - 4bet vs CO **[RARE]**
9. `4bet_vs_BTN_3bet.txt` - 4bet vs BTN **[RARE]**
10. `4bet_vs_SB_3bet.txt` - 4bet vs SB **[VERY RARE]**
11. `4bet_vs_BB_3bet.txt` - 4bet vs BB **[RARE]**

---

## MP (Middle Position) - 10 ranges

1. `RFI.txt` - Open raise
2. `3bet_vs_UTG.txt` - 3bet vs UTG
3. `call_vs_4bet_by_UTG.txt` - Call UTG 4bet after 3betting **[RARE]**
4. `call_vs_3bet_by_CO.txt` - Call CO 3bet
5. `call_vs_3bet_by_BTN.txt` - Call BTN 3bet
6. `call_vs_3bet_by_SB.txt` - Call SB 3bet **[RARE]**
7. `call_vs_3bet_by_BB.txt` - Call BB 3bet
8. `4bet_vs_CO_3bet.txt` - 4bet vs CO **[RARE]**
9. `4bet_vs_BTN_3bet.txt` - 4bet vs BTN **[RARE]**
10. `4bet_vs_SB_3bet.txt` - 4bet vs SB **[VERY RARE]**
11. `4bet_vs_BB_3bet.txt` - 4bet vs BB **[RARE]**

---

## CO (Cutoff) - 11 ranges

1. `RFI.txt` - Open raise
2. `3bet_vs_UTG.txt` - 3bet vs UTG
3. `call_vs_4bet_by_UTG.txt` - Call UTG 4bet **[RARE]**
4. `3bet_vs_MP.txt` - 3bet vs MP
5. `call_vs_4bet_by_MP.txt` - Call MP 4bet **[RARE]**
6. `call_vs_3bet_by_BTN.txt` - Call BTN 3bet
7. `call_vs_3bet_by_SB.txt` - Call SB 3bet **[RARE]**
8. `call_vs_3bet_by_BB.txt` - Call BB 3bet
9. `4bet_vs_BTN_3bet.txt` - 4bet vs BTN **[RARE]**
10. `4bet_vs_SB_3bet.txt` - 4bet vs SB **[VERY RARE]**
11. `4bet_vs_BB_3bet.txt` - 4bet vs BB **[RARE]**

---

## BTN (Button) - 14 ranges

1. `RFI.txt` - Open raise
2. `cold_call_vs_UTG.txt` - Cold call UTG open
3. `cold_call_vs_MP.txt` - Cold call MP open
4. `cold_call_vs_CO.txt` - Cold call CO open
5. `3bet_vs_UTG.txt` - 3bet vs UTG
6. `call_vs_4bet_by_UTG.txt` - Call UTG 4bet **[RARE]**
7. `3bet_vs_MP.txt` - 3bet vs MP
8. `call_vs_4bet_by_MP.txt` - Call MP 4bet **[RARE]**
9. `3bet_vs_CO.txt` - 3bet vs CO
10. `call_vs_4bet_by_CO.txt` - Call CO 4bet **[RARE]**
11. `call_vs_3bet_by_SB.txt` - Call SB 3bet
12. `call_vs_3bet_by_BB.txt` - Call BB 3bet
13. `4bet_vs_SB_3bet.txt` - 4bet vs SB **[RARE]**
14. `4bet_vs_BB_3bet.txt` - 4bet vs BB

---

## SB (Small Blind) - 11 ranges

1. `RFI.txt` - Open raise (vs BB)
2. `3bet_vs_UTG.txt` - 3bet vs UTG
3. `call_vs_4bet_by_UTG.txt` - Call UTG 4bet **[RARE]**
4. `3bet_vs_MP.txt` - 3bet vs MP
5. `call_vs_4bet_by_MP.txt` - Call MP 4bet **[RARE]**
6. `3bet_vs_CO.txt` - 3bet vs CO
7. `call_vs_4bet_by_CO.txt` - Call CO 4bet **[RARE]**
8. `3bet_vs_BTN.txt` - 3bet vs BTN
9. `call_vs_4bet_by_BTN.txt` - Call BTN 4bet
10. `call_vs_3bet_by_BB.txt` - Call BB 3bet **[RARE]**
11. `4bet_vs_BB_3bet.txt` - 4bet vs BB **[RARE]**

---

## BB (Big Blind) - 15 ranges

1. `defend_vs_RFI_UTG.txt` - Defend vs UTG open (call only)
2. `defend_vs_RFI_MP.txt` - Defend vs MP open
3. `defend_vs_RFI_CO.txt` - Defend vs CO open
4. `defend_vs_RFI_BTN.txt` - Defend vs BTN open
5. `defend_vs_RFI_SB.txt` - Defend vs SB open
6. `3bet_vs_UTG.txt` - 3bet vs UTG
7. `call_vs_4bet_by_UTG.txt` - Call UTG 4bet **[RARE]**
8. `3bet_vs_MP.txt` - 3bet vs MP
9. `call_vs_4bet_by_MP.txt` - Call MP 4bet **[RARE]**
10. `3bet_vs_CO.txt` - 3bet vs CO
11. `call_vs_4bet_by_CO.txt` - Call CO 4bet **[RARE]**
12. `3bet_vs_BTN.txt` - 3bet vs BTN
13. `call_vs_4bet_by_BTN.txt` - Call BTN 4bet
14. `3bet_vs_SB.txt` - 3bet vs SB
15. `call_vs_4bet_by_SB.txt` - Call SB 4bet **[RARE]**

---

## Summary

**Total per tightness level**: 72 ranges
- UTG: 11
- MP: 10 (removed cold_call - should 3bet or fold)
- CO: 11 (removed cold_calls - should 3bet or fold)
- BTN: 14 (kept cold calls - best position, can profitably call)
- SB: 11 (removed cold calls - OOP postflop, should 3bet or fold)
- BB: 15 (defends, no cold calls)

**Total unique ranges needed**: 72 ranges × 4 tightness levels = **288 ranges**

**Rarity markers**:
- **[RARE]**: <5% of hands - include in v1.1 but lower priority
- **[VERY RARE]**: <1% of hands - consider excluding or combining

**Most common scenarios for v1.0-v1.1** (no RARE markers):
- All RFI ranges (6)
- All defend_vs_RFI ranges (5)
- Common 3bets: BTN vs CO, BB vs BTN, BB vs CO, BB vs SB (most frequent)
- Common call_vs_3bet: positions calling 3bets from blinds
- SB cold calls (unique situation with dead money)

~35 most common scenarios × 4 levels = **140 ranges** for v1.1 focus

---

## Raw File Naming (GTO Wizard Downloads)

Raw files in `ranges/raw/` follow the convention:
`gto_wizard_{position}_{action}.txt`

**Examples**:
- `gto_wizard_BTN_RFI.txt`
- `gto_wizard_BB_defend_vs_RFI_BTN.txt`
- `gto_wizard_MP_call_vs_4bet_by_UTG.txt`

These get converted and distributed to tightness levels during range building.

---

**End of Range List**
