use postflop_solver::*;
use std::io::{self, Write};
use std::collections::HashMap;

mod suit_mapper;
use suit_mapper::{map_to_canonical, SuitMapping};

fn card_to_string(card: u8) -> String {
    let rank = match card / 4 {
        12 => 'A', 11 => 'K', 10 => 'Q', 9 => 'J', 8 => 'T',
        r => (b'2' + r) as char,
    };
    let suit = match card % 4 {
        0 => 'c', 1 => 'd', 2 => 'h', 3 => 's',
        _ => '?',
    };
    format!("{}{}", rank, suit)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== GTO Solution Lookup (Any Flop) ===\n");

    // Get actual flop from user
    print!("Enter flop (e.g., KhQd6h): ");
    io::stdout().flush()?;

    let mut flop_input = String::new();
    io::stdin().read_line(&mut flop_input)?;
    let actual_flop = flop_input.trim();

    // Map to canonical
    let (canonical_flop, suit_mapping) = map_to_canonical(actual_flop)?;

    println!("\nActual flop:    {}", actual_flop);
    println!("Canonical flop: {}", canonical_flop);
    println!("Loading solution for: {}\n", canonical_flop);

    // Get scenario from user
    print!("Enter scenario (e.g., BTN_RFI_vs_BB_defend): ");
    io::stdout().flush()?;

    let mut scenario_input = String::new();
    io::stdin().read_line(&mut scenario_input)?;
    let scenario = scenario_input.trim();

    // Load solution file with new naming: {scenario}___{canonical_flop}.bin
    let solution_path = format!("../solutions/{}___{}.bin", scenario, canonical_flop);
    let (mut game, memo): (PostFlopGame, String) = load_data_from_file(&solution_path, None)?;

    println!("✓ Loaded: {}\n", memo);

    game.back_to_root();
    game.cache_normalized_weights();

    let oop_cards = game.private_cards(0);
    let ip_cards = game.private_cards(1);

    println!("Game tree info:");
    println!("  OOP (BB) hands: {}", oop_cards.len());
    println!("  IP (BTN) hands: {}", ip_cards.len());
    println!();

    // Get hand from user (in actual suits)
    print!("Enter your hand (e.g., AhKd): ");
    io::stdout().flush()?;

    let mut hand_input = String::new();
    io::stdin().read_line(&mut hand_input)?;
    let actual_hand = hand_input.trim();

    if actual_hand.len() != 4 {
        return Err("Invalid hand format. Use 4 characters like 'AhKd'".into());
    }

    // Map hand to canonical suits
    let canonical_hand = suit_mapping.map_hand_to_canonical(actual_hand);

    println!("\nYour actual hand:    {}", actual_hand);
    println!("Canonical hand:      {}", canonical_hand);

    // Parse canonical hand
    let card1_str = &canonical_hand[0..2];
    let card2_str = &canonical_hand[2..4];

    let card1 = card_from_str(card1_str)?;
    let card2 = card_from_str(card2_str)?;

    // Check board conflicts (using canonical)
    let board = flop_from_str(&canonical_flop)?;
    if board.contains(&card1) || board.contains(&card2) {
        println!("\n⚠️  Hand conflicts with board");
        return Ok(());
    }

    // Find in OOP range
    if let Some(hand_idx) = oop_cards.iter().position(|&(c1, c2)| {
        (c1 == card1 && c2 == card2) || (c1 == card2 && c2 == card1)
    }) {
        let actions = game.available_actions();
        let strategy = game.strategy();
        let equity = game.equity(0);

        println!("\n✅ Found in OOP (BB) range");
        println!("Position: OOP acts first at root");
        println!("Equity: {:.1}%", equity[hand_idx] * 100.0);
        println!("\nGTO Strategy:");

        for (action_idx, action) in actions.iter().enumerate() {
            let freq = strategy[hand_idx + action_idx * oop_cards.len()];
            if freq > 0.01 {
                println!("  {:?}: {:.1}%", action, freq * 100.0);
            }
        }
    } else if let Some(hand_idx) = ip_cards.iter().position(|&(c1, c2)| {
        (c1 == card1 && c2 == card2) || (c1 == card2 && c2 == card1)
    }) {
        println!("\n✅ Found in IP (BTN) range");
        println!("Position: IP acts second at root");
        println!("(Navigate tree to see IP's strategy)");

        let equity = game.equity(1);
        println!("Equity: {:.1}%", equity[hand_idx] * 100.0);
    } else {
        println!("\n⚠️  Hand not in either range");
    }

    Ok(())
}
