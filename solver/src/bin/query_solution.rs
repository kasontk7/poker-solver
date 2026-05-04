use postflop_solver::*;
use std::io::{self, Write};

fn card_to_string(card: u8) -> String {
    let rank = match card / 4 {
        12 => 'A',
        11 => 'K',
        10 => 'Q',
        9 => 'J',
        8 => 'T',
        r => (b'2' + r) as char,
    };
    let suit = match card % 4 {
        0 => 's',
        1 => 'h',
        2 => 'd',
        3 => 'c',
        _ => '?',
    };
    format!("{}{}", rank, suit)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Poker Solver Query Tool ===");
    println!("Loading solution...\n");

    let bin_path = "../solutions/v1.0_KhQs6h.bin";
    let (mut game, memo): (PostFlopGame, String) = load_data_from_file(bin_path, None)?;

    println!("✓ Loaded: {}", memo);
    println!("Board: Kh Qs 6h (full tree)\n");

    game.back_to_root();
    game.cache_normalized_weights();

    let oop_cards = game.private_cards(0);
    let ip_cards = game.private_cards(1);

    println!("Available hands:");
    println!("  OOP (BB): {} combos", oop_cards.len());
    println!("  IP (BTN): {} combos", ip_cards.len());
    println!();

    loop {
        print!("Enter hand (e.g., 'AsAd') or 'q' to quit: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "q" || input == "quit" {
            println!("Goodbye!");
            break;
        }

        // Parse hand
        if input.len() != 4 {
            println!("Invalid format. Use 4 characters like 'AsAd'\n");
            continue;
        }

        let card1_str = &input[0..2];
        let card2_str = &input[2..4];

        let card1 = match card_from_str(card1_str) {
            Ok(c) => c,
            Err(_) => {
                println!("Invalid card: {}\n", card1_str);
                continue;
            }
        };

        let card2 = match card_from_str(card2_str) {
            Ok(c) => c,
            Err(_) => {
                println!("Invalid card: {}\n", card2_str);
                continue;
            }
        };

        // Check board conflicts
        let board = [
            card_from_str("Kh")?,
            card_from_str("Qs")?,
            card_from_str("6h")?,
        ];

        if board.contains(&card1) || board.contains(&card2) {
            println!("⚠️  Card is on the board (Kh Qs 6h)\n");
            continue;
        }

        // Find hand in OOP range
        if let Some(hand_idx) = oop_cards.iter().position(|&(c1, c2)| {
            (c1 == card1 && c2 == card2) || (c1 == card2 && c2 == card1)
        }) {
            let actions = game.available_actions();
            let strategy = game.strategy();
            let equity = game.equity(0);

            println!("\n  Hand: {} {} (OOP/BB at root)", card_to_string(card1), card_to_string(card2));
            println!("  Equity: {:.1}%", equity[hand_idx] * 100.0);
            println!("\n  GTO Strategy:");

            for (action_idx, action) in actions.iter().enumerate() {
                let freq = strategy[hand_idx + action_idx * oop_cards.len()];
                if freq > 0.01 {
                    println!("    {:?}: {:.1}%", action, freq * 100.0);
                }
            }
            println!();
        } else {
            println!("⚠️  Hand not in OOP range (try IP range next)\n");

            // Try IP range
            if let Some(hand_idx) = ip_cards.iter().position(|&(c1, c2)| {
                (c1 == card1 && c2 == card2) || (c1 == card2 && c2 == card1)
            }) {
                println!("  (Hand found in IP range but OOP acts first at root)");
                println!("  Navigate tree to see IP strategy\n");
            } else {
                println!("  Hand not in either range\n");
            }
        }
    }

    Ok(())
}
