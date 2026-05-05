#!/usr/bin/env python3
"""Reorganize ranges from raw/ to new folder structure"""

import os
import shutil

BASE = "/Users/kasonkang/personal/poker-solver/ranges"
RAW = f"{BASE}/raw"
GTO_WIZ = f"{BASE}/gto_wizard"

# RFI scenarios (8)
rfi_scenarios = [
    ("UTG_RFI_vs_BB_defend", "UTG", "BB", "RFI", "defend_vs_RFI_UTG"),
    ("MP_RFI_vs_BB_defend", "MP", "BB", "RFI", "defend_vs_RFI_MP"),
    ("CO_RFI_vs_BB_defend", "CO", "BB", "RFI", "defend_vs_RFI_CO"),
    ("BTN_RFI_vs_BB_defend", "BTN", "BB", "RFI", "defend_vs_RFI_BTN"),
    ("SB_RFI_vs_BB_defend", "SB", "BB", "RFI", "defend_vs_RFI_SB"),
    ("UTG_RFI_vs_BTN_cold_call", "UTG", "BTN", "RFI", "cold_call_vs_UTG"),
    ("MP_RFI_vs_BTN_cold_call", "MP", "BTN", "RFI", "cold_call_vs_MP"),
    ("CO_RFI_vs_BTN_cold_call", "CO", "BTN", "RFI", "cold_call_vs_CO"),
]

# 3bet scenarios (15) - {3bettor}_3bet_vs_{opener}_call
threebet_scenarios = [
    ("MP_3bet_vs_UTG_call", "MP", "UTG", "3bet_vs_UTG", "call_vs_3bet_by_MP"),
    ("CO_3bet_vs_UTG_call", "CO", "UTG", "3bet_vs_UTG", "call_vs_3bet_by_CO"),
    ("BTN_3bet_vs_UTG_call", "BTN", "UTG", "3bet_vs_UTG", "call_vs_3bet_by_BTN"),
    ("SB_3bet_vs_UTG_call", "SB", "UTG", "3bet_vs_UTG", "call_vs_3bet_by_SB"),
    ("BB_3bet_vs_UTG_call", "BB", "UTG", "3bet_vs_UTG", "call_vs_3bet_by_BB"),
    ("CO_3bet_vs_MP_call", "CO", "MP", "3bet_vs_MP", "call_vs_3bet_by_CO"),
    ("BTN_3bet_vs_MP_call", "BTN", "MP", "3bet_vs_MP", "call_vs_3bet_by_BTN"),
    ("SB_3bet_vs_MP_call", "SB", "MP", "3bet_vs_MP", "call_vs_3bet_by_SB"),
    ("BB_3bet_vs_MP_call", "BB", "MP", "3bet_vs_MP", "call_vs_3bet_by_BB"),
    ("BTN_3bet_vs_CO_call", "BTN", "CO", "3bet_vs_CO", "call_vs_3bet_by_BTN"),
    ("SB_3bet_vs_CO_call", "SB", "CO", "3bet_vs_CO", "call_vs_3bet_by_SB"),
    ("BB_3bet_vs_CO_call", "BB", "CO", "3bet_vs_CO", "call_vs_3bet_by_BB"),
    ("SB_3bet_vs_BTN_call", "SB", "BTN", "3bet_vs_BTN", "call_vs_3bet_by_SB"),
    ("BB_3bet_vs_BTN_call", "BB", "BTN", "3bet_vs_BTN", "call_vs_3bet_by_BB"),
    ("BB_3bet_vs_SB_call", "BB", "SB", "3bet_vs_SB", "call_vs_3bet_by_BB"),
]

# 4bet scenarios (15) - {4bettor}_4bet_vs_{3bettor}_call
fourbet_scenarios = [
    ("UTG_4bet_vs_MP_call", "UTG", "MP", "4bet_vs_MP_3bet", "call_vs_4bet_by_UTG"),
    ("UTG_4bet_vs_CO_call", "UTG", "CO", "4bet_vs_CO_3bet", "call_vs_4bet_by_UTG"),
    ("UTG_4bet_vs_BTN_call", "UTG", "BTN", "4bet_vs_BTN_3bet", "call_vs_4bet_by_UTG"),
    ("UTG_4bet_vs_SB_call", "UTG", "SB", "4bet_vs_SB_3bet", "call_vs_4bet_by_UTG"),
    ("UTG_4bet_vs_BB_call", "UTG", "BB", "4bet_vs_BB_3bet", "call_vs_4bet_by_UTG"),
    ("MP_4bet_vs_CO_call", "MP", "CO", "4bet_vs_CO_3bet", "call_vs_4bet_by_MP"),
    ("MP_4bet_vs_BTN_call", "MP", "BTN", "4bet_vs_BTN_3bet", "call_vs_4bet_by_MP"),
    ("MP_4bet_vs_SB_call", "MP", "SB", "4bet_vs_SB_3bet", "call_vs_4bet_by_MP"),
    ("MP_4bet_vs_BB_call", "MP", "BB", "4bet_vs_BB_3bet", "call_vs_4bet_by_MP"),
    ("CO_4bet_vs_BTN_call", "CO", "BTN", "4bet_vs_BTN_3bet", "call_vs_4bet_by_CO"),
    ("CO_4bet_vs_SB_call", "CO", "SB", "4bet_vs_SB_3bet", "call_vs_4bet_by_CO"),
    ("CO_4bet_vs_BB_call", "CO", "BB", "4bet_vs_BB_3bet", "call_vs_4bet_by_CO"),
    ("BTN_4bet_vs_SB_call", "BTN", "SB", "4bet_vs_SB_3bet", "call_vs_4bet_by_BTN"),
    ("BTN_4bet_vs_BB_call", "BTN", "BB", "4bet_vs_BB_3bet", "call_vs_4bet_by_BTN"),
    ("SB_4bet_vs_BB_call", "SB", "BB", "4bet_vs_BB_3bet", "call_vs_4bet_by_SB"),
]

def copy_scenario(folder, scenario, pos1, pos2, p1_action, p2_action):
    dest = f"{GTO_WIZ}/{folder}/{scenario}"
    os.makedirs(dest, exist_ok=True)
    src1 = f"{RAW}/gto_wizard_{pos1}_{p1_action}.txt"
    src2 = f"{RAW}/gto_wizard_{pos2}_{p2_action}.txt"
    if os.path.exists(src1):
        shutil.copy(src1, f"{dest}/{pos1.lower()}.txt")
        print(f"✓ {scenario}/{pos1.lower()}.txt")
    if os.path.exists(src2):
        shutil.copy(src2, f"{dest}/{pos2.lower()}.txt")
        print(f"✓ {scenario}/{pos2.lower()}.txt")

print("=== Reorganizing RFI scenarios ===")
for args in rfi_scenarios:
    copy_scenario("rfi", *args)

print("\n=== Reorganizing 3bet scenarios ===")
for args in threebet_scenarios:
    copy_scenario("3bet", *args)

print("\n=== Reorganizing 4bet scenarios ===")
for args in fourbet_scenarios:
    copy_scenario("4bet", *args)

print(f"\n✓ Complete. Check {GTO_WIZ}/")
