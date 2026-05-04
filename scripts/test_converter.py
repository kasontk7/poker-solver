#!/usr/bin/env python3
"""Test the range converter with sample data."""

from convert_range import parse_combo, convert_range, format_output


def test_parse_combo():
    """Test combo parsing."""
    assert parse_combo("AsAh") == "AA"
    assert parse_combo("2d2c") == "22"
    assert parse_combo("KsQs") == "KQs"
    assert parse_combo("KhQd") == "KQo"
    assert parse_combo("7c2s") == "72o"
    assert parse_combo("9s9h") == "99"
    print("✓ parse_combo tests passed")


def test_convert_range():
    """Test full conversion."""
    # Sample GTO Wizard format (all specific combos)
    input_text = """
    AsAh: 1.0,AsAd: 1.0,AsAc: 1.0,AhAd: 1.0,AhAc: 1.0,AdAc: 1.0,
    KsKh: 1.0,KsKd: 1.0,KsKc: 1.0,KhKd: 1.0,KhKc: 1.0,KdKc: 1.0,
    AsKs: 1.0,AhKh: 1.0,AdKd: 1.0,AcKc: 1.0,
    AsKh: 0.75,AsKd: 0.75,AsKc: 0.75,AhKs: 0.75,AhKd: 0.75,AhKc: 0.75,
    AdKs: 0.75,AdKh: 0.75,AdKc: 0.75,AcKs: 0.75,AcKh: 0.75,AcKd: 0.75
    """

    hands = convert_range(input_text)

    # Should have 3 hands: AA, KK, AKs, AKo
    assert "AA" in hands
    assert "KK" in hands
    assert "AKs" in hands
    assert "AKo" in hands

    # Check frequencies
    assert hands["AA"] == 1.0  # All 6 combos
    assert hands["KK"] == 1.0
    assert hands["AKs"] == 1.0  # All 4 suited combos
    assert hands["AKo"] == 0.75  # Only 3 out of 12 offsuit combos

    print("✓ convert_range tests passed")
    print(f"  Converted hands: {hands}")


def test_format_output():
    """Test output formatting."""
    hands = {
        "AA": 1.0,
        "KK": 0.95,
        "AKs": 1.0,
        "AKo": 0.75,
        "22": 0.5
    }

    output = format_output(hands)
    lines = output.split('\n')

    # Check format
    assert lines[0] == "AA:1.0000"  # High pairs first
    assert "22:0.5000" in output  # Low pair included
    assert "AKs:" in output  # Suited
    assert "AKo:" in output  # Offsuit
    # Order: pairs, then suited, then offsuit
    pair_idx = next(i for i, line in enumerate(lines) if line.startswith("22:"))
    aks_idx = next(i for i, line in enumerate(lines) if line.startswith("AKs:"))
    ako_idx = next(i for i, line in enumerate(lines) if line.startswith("AKo:"))
    assert pair_idx < aks_idx < ako_idx  # Verify ordering

    print("✓ format_output tests passed")
    print(f"  Sample output:\n{output}")


if __name__ == '__main__':
    test_parse_combo()
    test_convert_range()
    test_format_output()
    print("\n✅ All tests passed!")
