# Range Library

Hero always uses GTO ranges. Villain ranges vary by profile.

## Building Ranges

See `RANGE_MAPPING.md` for complete list and deduplication strategy.

### v2: Minimal (Start Here)
```
GTO/BTN/RFI.txt
GTO/BB/vs_RFI_BTN.txt
GTO/BTN/vs_3bet_by_BB.txt
GTO/BB/3bet_vs_BTN.txt
```

### v3: Add Profiles
Copy GTO structure, adjust for:
- NIT: Tighter (use symlinks to GTO tighter positions)
- FISH: Defends too wide
- MANIAC: Wide everything

## Format

Standard PIO format:
```
AA:1.0
KK:1.0
QQ:0.85
AKs:1.0
```

Hand notation: ranks (A-2), suits (s=suited, o=offsuit)
Frequency: 0.0-1.0 (0% to 100%)
