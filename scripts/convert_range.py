#!/usr/bin/env python3
"""
Convert GTO Wizard range format to postflop-solver format.

Input (GTO Wizard):
  2d2c: 0.3094,2h2c: 0.3094,2s2c: 0.3094,...

Output (postflop-solver):
  22:0.3094
  AKs:1.0
  AKo:0.75
"""

import sys
from collections import defaultdict
from pathlib import Path


def parse_combo(combo: str) -> str:
    """
    Convert specific combo to generalized hand notation.

    Examples:
      AsAh -> AA
      KsQs -> KQs
      KsQh -> KQo
    """
    if len(combo) != 4:
        raise ValueError(f"Invalid combo format: {combo}")

    rank1, suit1, rank2, suit2 = combo[0], combo[1], combo[2], combo[3]

    # Normalize to higher rank first
    if rank1 == rank2:
        return f"{rank1}{rank2}"  # Pair

    # Determine which rank is higher
    rank_order = "AKQJT98765432"
    if rank_order.index(rank1) < rank_order.index(rank2):
        high, low = rank1, rank2
        suited = suit1 == suit2
    else:
        high, low = rank2, rank1
        suited = suit1 == suit2

    return f"{high}{low}{'s' if suited else 'o'}"


def convert_range(input_text: str) -> dict[str, float]:
    """
    Convert GTO Wizard format to postflop-solver format.

    Returns dict of hand -> frequency (averaged across combos)
    """
    combos = defaultdict(list)

    # Parse input: "2d2c: 0.3094,2h2c: 0.3094,..."
    for entry in input_text.strip().split(','):
        entry = entry.strip()
        if not entry or ':' not in entry:
            continue

        combo_str, freq_str = entry.split(':', 1)
        combo_str = combo_str.strip()
        freq = float(freq_str.strip())

        hand = parse_combo(combo_str)
        combos[hand].append(freq)

    # Average frequencies for each hand
    result = {}
    for hand, freqs in combos.items():
        result[hand] = sum(freqs) / len(freqs)

    return result


def format_output(hands: dict[str, float]) -> str:
    """Format as postflop-solver range (line-separated, sorted by strength)."""

    # Sort by hand strength (approximate)
    rank_order = "AKQJT98765432"

    def hand_rank(hand: str) -> tuple:
        # Extract ranks
        if len(hand) == 2:  # Pair
            rank = hand[0]
            return (0, rank_order.index(rank), 0)
        elif hand.endswith('s'):  # Suited
            r1, r2 = hand[0], hand[1]
            return (1, rank_order.index(r1), rank_order.index(r2))
        else:  # Offsuit
            r1, r2 = hand[0], hand[1]
            return (2, rank_order.index(r1), rank_order.index(r2))

    sorted_hands = sorted(hands.items(), key=lambda x: hand_rank(x[0]))

    lines = []
    for hand, freq in sorted_hands:
        # Round to 4 decimal places
        lines.append(f"{hand}:{freq:.4f}")

    return '\n'.join(lines)


def main():
    if len(sys.argv) != 3:
        print("Usage: convert_range.py <input_file> <output_file>")
        print()
        print("Example:")
        print("  convert_range.py ranges/raw/gto_wizard_BTN_RFI.txt ranges/gto/BTN/RFI.txt")
        sys.exit(1)

    input_file = Path(sys.argv[1])
    output_file = Path(sys.argv[2])

    if not input_file.exists():
        print(f"Error: Input file not found: {input_file}")
        sys.exit(1)

    # Read and convert
    input_text = input_file.read_text()
    hands = convert_range(input_text)
    output_text = format_output(hands)

    # Write output
    output_file.parent.mkdir(parents=True, exist_ok=True)
    output_file.write_text(output_text + '\n')

    print(f"✓ Converted {len(hands)} hands")
    print(f"  Input:  {input_file}")
    print(f"  Output: {output_file}")


if __name__ == '__main__':
    main()
