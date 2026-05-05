/// Test: Navigate the solved tree directly and compare turn strategy
use postflop_solver::*;
use std::io::{self, Write};

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
    println!("=== Tree Navigation Test ===\n");

    let solution_path = "../solutions/v1.1_KhQs6h.bin";

    println!("Loading solution: {}", solution_path);
    let (mut game, _): (PostFlopGame, String) = load_data_from_file(solution_path, None)?;

    game.back_to_root();
    game.cache_normalized_weights();

    let hand_str = "6c6d";
    println!("Testing with hand: {}\n", hand_str);

    // Parse hand
    let hand_card1 = card_from_str(&hand_str[0..2])?;
    let hand_card2 = card_from_str(&hand_str[2..4])?;

    // Show OOP root strategy
    println!("=== FLOP ROOT (OOP) ===");
    let oop_cards = game.private_cards(0);
    let actions = game.available_actions();
    let strategy = game.strategy();
    let equity = game.equity(0);

    let hand_idx = oop_cards.iter().position(|&(c1, c2)|
        (c1 == hand_card1 && c2 == hand_card2) || (c1 == hand_card2 && c2 == hand_card1));

    if let Some(idx) = hand_idx {
        println!("OOP flop strategy for {}:", hand_str);
        for (i, action) in actions.iter().enumerate() {
            let freq = strategy[idx + i * oop_cards.len()];
            if freq > 0.001 {
                println!("  {:?}: {:.1}%", action, freq * 100.0);
            }
        }
        println!("  Equity: {:.1}%\n", equity[idx] * 100.0);
    }

    // Navigate flop: Check-Bet(275)-Raise(1375)-Call
    println!("Navigating: Check-Bet(275)-Raise(1375)-Call\n");

    // Action 0: OOP checks
    let check_idx = actions.iter().position(|a| matches!(a, Action::Check)).unwrap();
    game.play(check_idx);
    game.cache_normalized_weights();

    // Show IP strategy after OOP check
    println!("=== FLOP after OOP checks (IP) ===");
    let ip_cards = game.private_cards(1);
    let actions = game.available_actions();
    let strategy = game.strategy();
    let equity = game.equity(1);

    let hand_idx = ip_cards.iter().position(|&(c1, c2)|
        (c1 == hand_card1 && c2 == hand_card2) || (c1 == hand_card2 && c2 == hand_card1));

    if let Some(idx) = hand_idx {
        println!("IP flop strategy for {}:", hand_str);
        for (i, action) in actions.iter().enumerate() {
            let freq = strategy[idx + i * ip_cards.len()];
            if freq > 0.001 {
                println!("  {:?}: {:.1}%", action, freq * 100.0);
            }
        }
        println!("  Equity: {:.1}%\n", equity[idx] * 100.0);
    } else {
        println!("Hand {} not in IP range\n", hand_str);
    }

    // Action 1: IP bets 275
    let actions = game.available_actions();
    let bet275_idx = actions.iter().position(|a| matches!(a, Action::Bet(275))).unwrap();
    game.play(bet275_idx);
    game.cache_normalized_weights();

    // Action 2: OOP raises 1375
    let actions = game.available_actions();

    // Show OOP response after Check-Bet(275)
    println!("=== FLOP after Check-Bet(275) (OOP response) ===");
    let oop_cards = game.private_cards(0);
    let strategy = game.strategy();
    let equity = game.equity(0);

    let hand_idx = oop_cards.iter().position(|&(c1, c2)|
        (c1 == hand_card1 && c2 == hand_card2) || (c1 == hand_card2 && c2 == hand_card1));

    if let Some(idx) = hand_idx {
        println!("OOP response for {}:", hand_str);
        for (i, action) in actions.iter().enumerate() {
            let freq = strategy[idx + i * oop_cards.len()];
            if freq > 0.001 {
                println!("  {:?}: {:.1}%", action, freq * 100.0);
            }
        }
        println!("  Equity: {:.1}%\n", equity[idx] * 100.0);
    }

    let raise1375_idx = actions.iter().position(|a| matches!(a, Action::Raise(1375))).unwrap();
    game.play(raise1375_idx);
    game.cache_normalized_weights();

    // Action 3: IP calls
    let actions = game.available_actions();
    let call_idx = actions.iter().position(|a| matches!(a, Action::Call)).unwrap();
    game.play(call_idx);
    game.cache_normalized_weights();

    println!("After flop actions, at turn chance node\n");

    // Now we're at a chance node - navigate to turn 2c
    let actions = game.available_actions();
    let turn_2c = card_from_str("2c")?;
    let chance_idx = actions.iter().position(|a| {
        if let Action::Chance(card) = a {
            *card == turn_2c
        } else {
            false
        }
    }).unwrap();

    game.play(chance_idx);
    game.cache_normalized_weights();

    println!("Turn card: 2c\n");

    // Show the actual range at this node
    println!("=== TURN RANGES (from tree) ===");
    let oop_cards = game.private_cards(0);
    let ip_cards = game.private_cards(1);

    println!("OOP range has {} combos", oop_cards.len());
    println!("IP range has {} combos", ip_cards.len());

    // Show top 10 OOP hands by weight (checking frequency at this node)
    let oop_weights = game.normalized_weights(0);
    let mut oop_weighted: Vec<_> = oop_cards.iter().zip(oop_weights.iter())
        .map(|(&(c1, c2), &w)| (format!("{}{}", card_to_string(c1), card_to_string(c2)), w))
        .collect();
    oop_weighted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("\nTop 10 OOP hands by weight:");
    for (hand, weight) in oop_weighted.iter().take(10) {
        println!("  {}: {:.5}", hand, weight);
    }

    // Check specific hands
    println!("\nSpecific hand checks:");
    let check_hands = vec!["6c6d", "ThJh", "QdKc"];
    for check_hand in check_hands {
        let c1 = card_from_str(&check_hand[0..2]).unwrap();
        let c2 = card_from_str(&check_hand[2..4]).unwrap();
        if let Some(idx) = oop_cards.iter().position(|&(card1, card2)|
            (card1 == c1 && card2 == c2) || (card1 == c2 && card2 == c1)) {
            println!("  {} weight: {:.5}", check_hand, oop_weights[idx]);
        } else {
            println!("  {} not in range", check_hand);
        }
    }

    println!();

    // Now show OOP strategy on turn
    let actions = game.available_actions();
    let strategy = game.strategy();
    let equity = game.equity(0);

    // Find 6c6d
    let hand_card1 = card_from_str(&hand_str[0..2])?;
    let hand_card2 = card_from_str(&hand_str[2..4])?;
    let hand_idx = oop_cards.iter().position(|&(c1, c2)|
        (c1 == hand_card1 && c2 == hand_card2) || (c1 == hand_card2 && c2 == hand_card1));

    if let Some(idx) = hand_idx {
        println!("OOP turn strategy for {}:", hand_str);
        for (i, action) in actions.iter().enumerate() {
            let freq = strategy[idx + i * oop_cards.len()];
            if freq > 0.001 {
                println!("  {:?}: {:.1}%", action, freq * 100.0);
            }
        }
        println!("  Equity: {:.1}%", equity[idx] * 100.0);
    } else {
        println!("Hand not found in range!");
    }

    Ok(())
}
