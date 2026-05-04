# Suit Normalization for Flop Isomorphism

## Problem Statement

We solve **184 canonical flops** but need to handle **22,100 real flops** at runtime.

**Challenge**: How do we map a real board like `Kh Qd 6h` to canonical `Ks Qh 6s` while preserving flush draws?

If we just randomly map suits, we'll break flush draw detection:
- Real: `Kh Qd 6h` + `JhTh` (flush draw!)
- Bad mapping: → `Ks Qh 6s` + `JdTd` (no longer a flush draw ❌)

## Solution: Consistent Suit Mapping

Map suits based on **frequency on the board**, ensuring hero hand uses the **same mapping**.

---

## Algorithm

### Step 1: Count Suit Frequencies

```python
def count_suits(board):
    """Count how many cards of each suit are on the board"""
    suits = {'s': 0, 'h': 0, 'd': 0, 'c': 0}
    for card in board:
        suit = card[1]  # e.g., 'Kh' → 'h'
        suits[suit] += 1
    return suits

# Example
board = ['Kh', 'Qd', '6h']
counts = count_suits(board)  # {'h': 2, 'd': 1, 's': 0, 'c': 0}
```

### Step 2: Order Suits by Frequency

```python
def order_suits(suit_counts, board):
    """Order suits: primary (most frequent) → quaternary (unused)"""
    
    # Sort by count (descending), then by highest card rank in that suit
    def suit_key(suit):
        count = suit_counts[suit]
        # Find highest card of this suit on board
        ranks = {'A': 14, 'K': 13, 'Q': 12, 'J': 11, 'T': 10,
                 '9': 9, '8': 8, '7': 7, '6': 6, '5': 5, '4': 4, '3': 3, '2': 2}
        highest_rank = 0
        for card in board:
            if card[1] == suit:
                highest_rank = max(highest_rank, ranks[card[0]])
        
        return (-count, -highest_rank)  # Negative for descending order
    
    suits = ['s', 'h', 'd', 'c']
    return sorted(suits, key=suit_key)

# Example
board = ['Kh', 'Qd', '6h']
counts = {'h': 2, 'd': 1, 's': 0, 'c': 0}
ordered = order_suits(counts, board)
# Result: ['h', 'd', 's', 'c']
#         (2 hearts with K-high, 1 diamond, 0 spades, 0 clubs)
```

### Step 3: Create Suit Mapping

```python
def create_suit_mapping(ordered_suits):
    """Map real suits → canonical suits"""
    canonical = ['s', 'h', 'd', 'c']
    mapping = {}
    for i, real_suit in enumerate(ordered_suits):
        mapping[real_suit] = canonical[i]
    return mapping

# Example
ordered = ['h', 'd', 's', 'c']
mapping = create_suit_mapping(ordered)
# Result: {'h': 's',  # Primary (2 cards) → spades
#          'd': 'h',  # Secondary (1 card) → hearts  
#          's': 'd',  # Tertiary (0 cards) → diamonds
#          'c': 'c'}  # Quaternary (0 cards) → clubs
```

### Step 4: Apply Mapping to Board and Hand

```python
def normalize_card(card, suit_map):
    """Map a single card to canonical representation"""
    rank = card[0]
    suit = card[1]
    canonical_suit = suit_map[suit]
    return rank + canonical_suit

def normalize_board_and_hand(board, hand, suit_map):
    """Apply the SAME mapping to both board and hand"""
    canonical_board = [normalize_card(card, suit_map) for card in board]
    canonical_hand = [normalize_card(card, suit_map) for card in hand]
    return canonical_board, canonical_hand

# Example
board = ['Kh', 'Qd', '6h']
hand = ['Jh', 'Th']  # Flush draw!
suit_map = {'h': 's', 'd': 'h', 's': 'd', 'c': 'c'}

canonical_board, canonical_hand = normalize_board_and_hand(board, hand, suit_map)
# canonical_board = ['Ks', 'Qh', '6s']
# canonical_hand = ['Js', 'Ts']  ← Still a flush draw! (2 spades)
```

---

## Complete Example

### Example 1: Flush Draw

**Real board**: `Ah Kh Qd` (2 hearts, 1 diamond)  
**Real hand**: `JhTh` (both hearts - flush draw!)

**Step 1**: Count suits
```
{'h': 2, 'd': 1, 's': 0, 'c': 0}
```

**Step 2**: Order suits
```
['h', 'd', 's', 'c']  (hearts most frequent)
```

**Step 3**: Create mapping
```
{'h': 's', 'd': 'h', 's': 'd', 'c': 'c'}
```

**Step 4**: Apply mapping
```
Board: Ah Kh Qd → As Ks Qh
Hand:  Jh Th   → Js Ts
```

**Result**: `Js Ts` on `As Ks Qh` is a **spade flush draw** ✅

---

### Example 2: Non-Flush Hand (Same Board)

**Real board**: `Ah Kh Qd` (same as above)  
**Real hand**: `JdTd` (both diamonds - NO flush draw)

**Mapping** (same as Example 1):
```
{'h': 's', 'd': 'h', 's': 'd', 'c': 'c'}
```

**Apply mapping**:
```
Board: Ah Kh Qd → As Ks Qh
Hand:  Jd Td   → Jh Th
```

**Result**: `Jh Th` on `As Ks Qh` is **not a flush draw** (1 spade, 1 heart) ✅

---

### Example 3: Offsuit Combo

**Real board**: `Kh Qd 6h`  
**Real hand**: `AsKd` (Ace of spades, King of diamonds - offsuit)

**Step 1-3**: Same mapping as before
```
{'h': 's', 'd': 'h', 's': 'd', 'c': 'c'}
```

**Step 4**: Apply mapping
```
Board: Kh Qd 6h → Ks Qh 6s
Hand:  As Kd   → Ad Kh
```

**Result**: `Ad Kh` on `Ks Qh 6s` - different suits preserved ✅

---

### Example 4: Backdoor Flush Draw

**Real board**: `Kh Qd 6h` (2 hearts, 1 diamond)  
**Real hand**: `JdTd` (both diamonds - backdoor flush draw!)

**Step 1**: Count suits
```
{'h': 2, 'd': 1, 's': 0, 'c': 0}
```

**Step 2**: Order suits
```
['h', 'd', 's', 'c']  (hearts most frequent with K-high)
```

**Step 3**: Create mapping
```
{'h': 's', 'd': 'h', 's': 'd', 'c': 'c'}
```

**Step 4**: Apply mapping
```
Board: Kh Qd 6h → Ks Qh 6s
Hand:  Jd Td   → Jh Th
```

**Result**: `Jh Th` on `Ks Qh 6s` is a **hearts backdoor flush draw** ✅  
(2 hearts: Qh on board + Jh Th in hand = backdoor)

---

### Example 5: Rainbow Board (Different Suits)

**Real board**: `As Kh 7d` (rainbow: spades, hearts, diamonds)  
**Real hand**: `QsJs` (both spades - suited connector)

**Step 1**: Count suits
```
{'s': 1, 'h': 1, 'd': 1, 'c': 0}
```

**Step 2**: Order suits (tiebreak by highest card rank)
```
As (14) > Kh (13) > 7d (7)
['s', 'h', 'd', 'c']  (spades has Ace, highest)
```

**Step 3**: Create mapping
```
{'s': 's', 'h': 'h', 'd': 'd', 'c': 'c'}  ← Identity mapping!
```

**Step 4**: Apply mapping
```
Board: As Kh 7d → As Kh 7d  (no change)
Hand:  Qs Js   → Qs Js      (no change)
```

**Result**: Already canonical, direct lookup ✅

---

### Example 6: Rainbow Board (Suits Out of Order)

**Real board**: `Kd Ah 7s` (same ranks as Ex 5, different suit order)  
**Real hand**: `QdJd` (both diamonds - suited connector)

**Step 1**: Count suits
```
{'d': 1, 'h': 1, 's': 1, 'c': 0}
```

**Step 2**: Order suits (tiebreak by highest card rank)
```
Ah (14) > Kd (13) > 7s (7)
['h', 'd', 's', 'c']  (hearts has Ace, highest)
```

**Step 3**: Create mapping
```
{'h': 's', 'd': 'h', 's': 'd', 'c': 'c'}
```

**Step 4**: Apply mapping
```
Board: Kd Ah 7s → Kh As 7d → As Kh 7d  (after sorting by rank)
Hand:  Qd Jd   → Qh Jh
```

**Result**: Maps to SAME canonical as Example 5! ✅  
Both `As Kh 7d` boards with suited connectors map identically.

---

### Example 7: Monotone Board

**Real board**: `Qd Jd 8d` (all diamonds)  
**Real hand**: `Td9d` (both diamonds - made flush!)

**Step 1**: Count suits
```
{'d': 3, 'h': 0, 's': 0, 'c': 0}
```

**Step 2**: Order suits
```
['d', 'h', 's', 'c']  (diamonds only suit on board)
```

**Step 3**: Create mapping
```
{'d': 's', 'h': 'h', 's': 'd', 'c': 'c'}
```

**Step 4**: Apply mapping
```
Board: Qd Jd 8d → Qs Js 8s
Hand:  Td 9d   → Ts 9s
```

**Result**: `Ts 9s` on `Qs Js 8s` - made spade flush ✅

---

### Example 8: Monotone Board with Offsuit Hand

**Real board**: `Qd Jd 8d` (all diamonds)  
**Real hand**: `Ah Kh` (both hearts - no flush!)

**Mapping** (same as Example 7):
```
{'d': 's', 'h': 'h', 's': 'd', 'c': 'c'}
```

**Apply mapping**:
```
Board: Qd Jd 8d → Qs Js 8s
Hand:  Ah Kh   → Ah Kh
```

**Result**: `Ah Kh` on `Qs Js 8s` - no flush (all spades on board, both hearts in hand) ✅

---

### Example 9: Paired Board with Flush Draw

**Real board**: `Tc Tc 6c` (pair of Tens, all clubs)  
**Real hand**: `AcKc` (both clubs - flush draw with pair on board)

**Step 1**: Count suits
```
{'c': 3, 'h': 0, 'd': 0, 's': 0}
```

**Step 2**: Order suits
```
['c', 'h', 'd', 's']
```

**Step 3**: Create mapping
```
{'c': 's', 'h': 'h', 'd': 'd', 's': 'c'}
```

**Step 4**: Apply mapping
```
Board: Tc Tc 6c → Ts Ts 6s
Hand:  Ac Kc   → As Ks
```

**Result**: `As Ks` on `Ts Ts 6s` - pair + flush draw preserved ✅

---

## Properties Preserved

✅ **Flush draws**: 2+ cards of primary suit  
✅ **Backdoor flush draws**: 2 cards same suit (not primary)  
✅ **Offsuit combos**: Cards map to different suit buckets  
✅ **Blocker effects**: Specific card removal preserved (e.g., Kh blocks villain's Kh)  
✅ **Pair texture**: Paired boards stay paired  
✅ **Connectedness**: Straight draws preserved

---

## Edge Cases

### Paired Boards

**Real**: `Kh Kd 6h` (pair of Kings, 2 hearts, 1 diamond)

**Counts**: `{'h': 2, 'd': 1, 's': 0, 'c': 0}`

**Mapping**: `{'h': 's', 'd': 'h', 's': 'd', 'c': 'c'}`

**Canonical**: `Ks Kh 6s`

✅ Pair preserved, suit distribution correct

### Monotone Boards

**Real**: `Kh Qh Jh` (all hearts)

**Counts**: `{'h': 3, 'd': 0, 's': 0, 'c': 0}`

**Mapping**: `{'h': 's', 'd': 'h', 's': 'd', 'c': 'c'}`

**Canonical**: `Ks Qs Js`

✅ Monotone preserved (all spades)

### Rainbow Boards (Can Be Identity or Require Mapping)

**Case 1: Already in canonical order**

**Real**: `Ks Qh 6d` 

**Counts**: `{'s': 1, 'h': 1, 'd': 1, 'c': 0}`

**Ordered**: `['s', 'h', 'd', 'c']` (tiebreak by rank: K > Q > 6)

**Mapping**: `{'s': 's', 'h': 'h', 'd': 'd', 'c': 'c'}` (identity!)

**Canonical**: `Ks Qh 6d` (no change)

✅ Already canonical

---

**Case 2: Suits need reordering**

**Real**: `Kd Qh 6s` (different suit order, same ranks)

**Counts**: `{'d': 1, 'h': 1, 's': 1, 'c': 0}`

**Ordered**: `['d', 'h', 's', 'c']` (tiebreak by rank: Kd=13 > Qh=12 > 6s=6)

**Mapping**: `{'d': 's', 'h': 'h', 's': 'd', 'c': 'c'}`

**Apply mapping**:
```
Board: Kd Qh 6s → Ks Qh 6d
```

**Canonical**: `Ks Qh 6d` (normalized to canonical suit order)

✅ Maps to same canonical as Case 1

---

## Implementation Notes

### Performance
- Suit counting: O(3) = constant
- Suit ordering: O(4 log 4) = constant  
- Mapping creation: O(4) = constant
- Card normalization: O(5) for board + hand
- **Total: <1ms per lookup**

### Caching
```python
class SuitNormalizer:
    def __init__(self):
        self.cache = {}  # board_tuple → (canonical_board, suit_map)
    
    def normalize(self, board, hand):
        board_tuple = tuple(sorted(board))
        
        if board_tuple not in self.cache:
            # Compute mapping once per unique board
            suit_map = self._create_mapping(board)
            canonical_board = self._apply_mapping(board, suit_map)
            self.cache[board_tuple] = (canonical_board, suit_map)
        
        canonical_board, suit_map = self.cache[board_tuple]
        canonical_hand = self._apply_mapping(hand, suit_map)
        
        return canonical_board, canonical_hand
```

### Testing Strategy

Test cases to validate:
1. ✅ Flush draws preserved
2. ✅ Non-flush hands on same board different from flush draws
3. ✅ Backdoor draws preserved  
4. ✅ Offsuit combos map correctly
5. ✅ Monotone boards all map to all-spades
6. ✅ Two-tone boards map to spades+hearts
7. ✅ Rainbow boards map to spades+hearts+diamonds
8. ✅ Paired boards preserve pairs
9. ✅ Multiple boards with same structure map to same canonical

---

## Summary

**The key insight**: Use the **same suit mapping** for both board and hand.

This ensures that relationships between hand and board (flush draws, blockers, etc.) are preserved in the canonical representation.

**Why it works**: Poker strategy depends on **relative suit relationships**, not absolute suit identity. As long as we map consistently, the strategy remains correct.
