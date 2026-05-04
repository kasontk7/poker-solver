use postflop_solver::*;
use std::fs;
use std::io::{self, Write};

fn load_range_from_file(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut hands = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || !line.contains(':') {
            continue;
        }

        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() != 2 {
            continue;
        }

        let hand = parts[0].trim();
        let freq: f32 = parts[1].trim().parse()?;

        if (freq - 1.0).abs() < 0.001 {
            hands.push(hand.to_string());
        } else {
            hands.push(format!("{}:{:.4}", hand, freq));
        }
    }

    Ok(hands.join(","))
}

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
        0 => 's', // spades
        1 => 'h', // hearts
        2 => 'd', // diamonds
        3 => 'c', // clubs
        _ => '?',
    };
    format!("{}{}", rank, suit)
}

fn print_separator() {
    println!("\n{}\n", "=".repeat(70));
}

fn navigate_game_tree(
    game: &mut PostFlopGame,
    player: usize,
    hand_index: usize,
    card1: u8,
    card2: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    game.back_to_root();
    let position_name = if player == 1 { "BTN (IP)" } else { "BB (OOP)" };

    println!("\n🎮 Game Tree Navigation");
    println!("{}", "=".repeat(70));
    println!("Hand: {} {}", card_to_string(card1), card_to_string(card2));
    println!("Position: {}", position_name);
    println!("\nNavigating from FLOP...\n");

    loop {
        // Cache weights at each node
        game.cache_normalized_weights();
        let actions = game.available_actions();

        if actions.is_empty() {
            println!("\n🏁 SHOWDOWN - No more actions available");

            let equity = game.equity(player);
            let ev = game.expected_values(player);
            println!("\n💰 Final Equity: {:.2}%", equity[hand_index] * 100.0);
            println!("💵 Final EV: ${:.2}", ev[hand_index] / 100.0);

            println!("\nPress Enter to return...");
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            break;
        }

        let current_player = game.current_player();
        let acting_player_name = if current_player == 0 { "BB (OOP)" } else { "BTN (IP)" };

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("👤 {} to act", acting_player_name);

        let equity = game.equity(player);
        let ev = game.expected_values(player);
        println!("💰 Your Equity: {:.2}% | EV: ${:.2}",
            equity[hand_index] * 100.0, ev[hand_index] / 100.0);

        // Show strategy for this hand
        if current_player == player {
            let strategy = game.strategy();
            let private_cards = game.private_cards(player);
            let num_hands = private_cards.len();

            println!("\n📊 Your GTO Strategy:");
            for (i, action) in actions.iter().enumerate() {
                let freq = strategy[hand_index + i * num_hands];
                let percentage = freq * 100.0;
                let bar_length = (percentage / 2.0) as usize;
                let bar = "█".repeat(bar_length);
                println!("  {:2}. {:20} {:6.2}%  {}", i + 1, format!("{:?}", action), percentage, bar);
            }
        } else {
            println!("\n⏳ Waiting for {} to act", acting_player_name);
        }

        println!("\n📍 Available Actions:");
        for (i, action) in actions.iter().enumerate() {
            println!("  {:2}. {:?}", i + 1, action);
        }
        println!("  r. Return to root (flop) and restart");
        println!("  q. Exit navigation");

        print!("\nChoose action: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input {
            "q" => {
                println!("\nExiting navigation...");
                break;
            }
            "r" => {
                game.back_to_root();
                println!("\n🔄 Returned to root (flop) - restarting navigation\n");
            }
            _ => {
                if let Ok(choice) = input.parse::<usize>() {
                    if choice > 0 && choice <= actions.len() {
                        game.play(choice - 1);
                        println!("\n✓ Played: {:?}\n", actions[choice - 1]);
                    } else {
                        println!("\n⚠️  Invalid action number\n");
                    }
                } else {
                    println!("\n⚠️  Invalid input\n");
                }
            }
        }
    }

    game.back_to_root();
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "═".repeat(70));
    println!("  POKER SOLVER v1.0 - Interactive Mode");
    println!("{}\n", "═".repeat(70));

    println!("Board: Kh Qs 6h 9d 3c");
    println!("Scenario: BTN RFI vs BB call\n");

    // Load and solve
    println!("⏳ Loading ranges and solving...");

    let ip_range = load_range_from_file("../ranges/gto/BTN/RFI.txt")?;
    let oop_range = load_range_from_file("../ranges/gto/BB/defend_vs_RFI_BTN.txt")?;

    let card_config = CardConfig {
        range: [
            oop_range.parse()?,
            ip_range.parse()?,
        ],
        flop: flop_from_str("KhQs6h")?,
        turn: card_from_str("9d")?,
        river: card_from_str("3c")?,
    };

    let flop_bet_sizes = BetSizeOptions::try_from(("25%, 50%, 100%", "2.5x"))?;
    let turn_bet_sizes = BetSizeOptions::try_from(("25%, 50%, 100%, a", "2.5x"))?;
    let river_bet_sizes = BetSizeOptions::try_from(("50%, 100%, 150%, a", "2.5x"))?;
    let donk_sizes = DonkSizeOptions::try_from("33%, 50%")?;

    let tree_config = TreeConfig {
        initial_state: BoardState::River,
        starting_pot: 550,
        effective_stack: 9750,
        rake_rate: 0.05,
        rake_cap: 300.0,
        flop_bet_sizes: [flop_bet_sizes.clone(), flop_bet_sizes.clone()],
        turn_bet_sizes: [turn_bet_sizes.clone(), turn_bet_sizes.clone()],
        river_bet_sizes: [river_bet_sizes.clone(), river_bet_sizes],
        turn_donk_sizes: Some(donk_sizes.clone()),
        river_donk_sizes: Some(donk_sizes),
        add_allin_threshold: 1.5,
        force_allin_threshold: 0.15,
        merging_threshold: 0.1,
    };

    let action_tree = ActionTree::new(tree_config)?;
    let mut game = PostFlopGame::with_config(card_config, action_tree)?;
    game.allocate_memory(false);

    let target_exploitability = game.tree_config().starting_pot as f32 * 0.005;
    solve(&mut game, 1000, target_exploitability, false);

    println!("✓ Solution ready!\n");

    print_separator();

    // Interactive mode
    loop {
        println!("Select position:");
        println!("  1) BTN (In Position - IP)");
        println!("  2) BB (Out of Position - OOP)");
        println!("  q) Quit");
        print!("\nChoice: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "q" || input == "quit" {
            println!("\nGoodbye!");
            break;
        }

        let player = match input {
            "1" => 1, // IP (BTN)
            "2" => 0, // OOP (BB)
            _ => {
                println!("Invalid choice. Try again.\n");
                continue;
            }
        };

        let position_name = if player == 1 { "BTN (IP)" } else { "BB (OOP)" };

        print_separator();

        // Get hero hand
        println!("Enter your hole cards (e.g., 'AhKs' or 'Ah Ks'):");
        print!("Cards: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().replace(" ", "");

        if input.len() != 4 {
            println!("Invalid format. Try again.\n");
            continue;
        }

        // Parse cards
        let card1_str = &input[0..2];
        let card2_str = &input[2..4];

        let card1 = match card_from_str(card1_str) {
            Ok(c) => c,
            Err(_) => {
                println!("Invalid card: {}. Try again.\n", card1_str);
                continue;
            }
        };

        let card2 = match card_from_str(card2_str) {
            Ok(c) => c,
            Err(_) => {
                println!("Invalid card: {}. Try again.\n", card2_str);
                continue;
            }
        };

        // Check if cards are on the board
        let board_cards = [
            card_from_str("Kh")?,
            card_from_str("Qs")?,
            card_from_str("6h")?,
            card_from_str("9d")?,
            card_from_str("3c")?,
        ];

        if board_cards.contains(&card1) {
            println!("\n⚠️  {} is on the board! Try again.\n", card_to_string(card1));
            continue;
        }
        if board_cards.contains(&card2) {
            println!("\n⚠️  {} is on the board! Try again.\n", card_to_string(card2));
            continue;
        }

        // Check if hand is in player's range
        game.back_to_root();
        game.cache_normalized_weights();

        let private_cards = game.private_cards(player);
        let hand_index = private_cards.iter().position(|&h| {
            (h.0 == card1 && h.1 == card2) || (h.0 == card2 && h.1 == card1)
        });

        let hand_index = match hand_index {
            Some(idx) => idx,
            None => {
                println!("\n⚠️  Hand {} {} is not in {}'s range!",
                    card_to_string(card1), card_to_string(card2), position_name);
                continue;
            }
        };

        print_separator();

        // Show initial equity and EV
        let equity = game.equity(player);
        let ev = game.expected_values(player);

        println!("Position: {}", position_name);
        println!("Hand: {} {}", card_to_string(card1), card_to_string(card2));
        println!("Board: Kh Qs 6h 9d 3c");
        println!("\n💰 Equity: {:.2}%", equity[hand_index] * 100.0);
        println!("💵 EV: ${:.2}", ev[hand_index] / 100.0);

        // Show available actions and strategy
        let actions = game.available_actions();

        if actions.is_empty() {
            println!("\n⚠️  No actions available (terminal node or showdown)");
        } else {
            let strategy = game.strategy();
            let num_hands = private_cards.len();

            // Check whose turn it is
            let current_player = game.current_player();
            let acting_player_name = if current_player == 0 { "BB (OOP)" } else { "BTN (IP)" };

            println!("\n📊 GTO Strategy for {} ({})", position_name,
                if current_player == player { "YOUR TURN" } else { "WAITING" });
            println!("{}", "-".repeat(50));

            if current_player == player {
                for (i, action) in actions.iter().enumerate() {
                    let freq = strategy[hand_index + i * num_hands];
                    let percentage = freq * 100.0;

                    let action_str = format!("{:?}", action);
                    let bar_length = (percentage / 2.0) as usize;
                    let bar = "█".repeat(bar_length);

                    println!("{:20} {:6.2}%  {}", action_str, percentage, bar);
                }
            } else {
                println!("Waiting for {} to act", acting_player_name);
            }
        }

        print_separator();

        println!("\nWould you like to:");
        println!("  1) Query another hand");
        println!("  2) Navigate game tree");
        println!("  q) Quit");
        print!("\nChoice: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "q" {
            println!("\nGoodbye!");
            break;
        }

        if input == "2" {
            // Navigate game tree
            navigate_game_tree(&mut game, player, hand_index, card1, card2)?;
        }

        print_separator();
    }

    Ok(())
}
