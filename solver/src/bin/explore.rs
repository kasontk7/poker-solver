use postflop_solver::*;
use std::io::{self, Write};

fn card_to_string(card: u8) -> String {
    let rank = match card / 4 {
        12 => 'A', 11 => 'K', 10 => 'Q', 9 => 'J', 8 => 'T',
        r => (b'2' + r) as char,
    };
    let suit = match card % 4 {
        0 => 's', 1 => 'h', 2 => 'd', 3 => 'c',
        _ => '?',
    };
    format!("{}{}", rank, suit)
}

fn display_hand_matrix(hands: &[(u8, u8)], selected: Option<usize>) {
    println!("\nAvailable hands (type number to select):");
    for (idx, &(c1, c2)) in hands.iter().enumerate() {
        let marker = if Some(idx) == selected { "►" } else { " " };
        print!("{} {:3}. {} {}  ", marker, idx + 1, card_to_string(c1), card_to_string(c2));
        if (idx + 1) % 5 == 0 {
            println!();
        }
    }
    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════╗");
    println!("║   GTO Poker Solution Explorer v1.1    ║");
    println!("╚════════════════════════════════════════╝\n");

    // Load solution
    print!("Enter path to .bin file [../solutions/v1.1_KhQs6h.bin]: ");
    io::stdout().flush()?;

    let mut path_input = String::new();
    io::stdin().read_line(&mut path_input)?;
    let bin_path = path_input.trim();
    let bin_path = if bin_path.is_empty() {
        "../solutions/v1.1_KhQs6h.bin"
    } else {
        bin_path
    };

    println!("\nLoading {}...", bin_path);
    let (mut game, memo): (PostFlopGame, String) = load_data_from_file(bin_path, None)?;

    println!("✓ Loaded: {}\n", memo);

    // Get initial state and copy hand data (to avoid borrow checker issues)
    game.back_to_root();

    let oop_hands: Vec<(u8, u8)> = game.private_cards(0).to_vec();
    let ip_hands: Vec<(u8, u8)> = game.private_cards(1).to_vec();

    println!("Solution info:");
    println!("  OOP hands: {}", oop_hands.len());
    println!("  IP hands: {}", ip_hands.len());
    println!();

    // Choose position
    println!("Choose your position:");
    println!("  1. OOP - BB Defend (acts first)");
    println!("  2. IP - BTN RFI (acts second)");
    print!("\nPosition [1]: ");
    io::stdout().flush()?;

    let mut pos_input = String::new();
    io::stdin().read_line(&mut pos_input)?;
    let is_oop = pos_input.trim() != "2";
    let player = if is_oop { 0 } else { 1 };
    let hero_hands = if is_oop { &oop_hands } else { &ip_hands };

    let position_desc = if is_oop { "OOP (BB - Defend)" } else { "IP (BTN - RFI)" };
    println!("\n✓ Playing as {} ({} hands)\n", position_desc, hero_hands.len());

    // Choose hand
    display_hand_matrix(hero_hands, None);

    print!("Select your hand (number): ");
    io::stdout().flush()?;

    let mut hand_input = String::new();
    io::stdin().read_line(&mut hand_input)?;
    let hand_idx: usize = hand_input.trim().parse::<usize>()
        .map(|n| n.saturating_sub(1))
        .unwrap_or(0)
        .min(hero_hands.len() - 1);

    let (c1, c2) = hero_hands[hand_idx];
    println!("\n✓ Selected: {} {}\n", card_to_string(c1), card_to_string(c2));

    // Get board cards from config
    let flop = game.card_config().flop;

    // Navigation loop
    game.back_to_root();
    let mut action_history: Vec<String> = Vec::new();

    loop {
        // Handle chance nodes (turn/river dealing) automatically
        while game.is_chance_node() {
            // Deal first available card
            game.play(0);
            action_history.push(format!("[Card dealt]"));
        }

        game.cache_normalized_weights();
        let actions = game.available_actions();

        if actions.is_empty() {
            println!("\n═══ SHOWDOWN ═══");
            println!("Action history: {}", action_history.join(" → "));

            let equity = game.equity(player);
            let ev = game.expected_values(player);

            println!("\nYour hand: {} {}", card_to_string(c1), card_to_string(c2));
            println!("  Equity: {:.1}%", equity[hand_idx] * 100.0);
            println!("  EV: ${:.2}", ev[hand_idx] / 100.0);

            println!("\nEnd of tree. Type 'r' to restart or 'q' to quit.");
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            match input.trim() {
                "q" | "quit" => break,
                _ => {
                    game.back_to_root();
                    action_history.clear();
                    continue;
                }
            }
        }

        // Display current state
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // Show board - use postflop-solver's built-in formatting
        println!("Board: Kh Qs 6h (full tree)");

        println!("History: {}",
            if action_history.is_empty() { "ROOT".to_string() }
            else { action_history.join(" → ") }
        );
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let strategy = game.strategy();
        let equity = game.equity(player);
        let current_player = game.current_player();

        println!("\nYour hand: {} {}", card_to_string(c1), card_to_string(c2));
        println!("  Equity: {:.1}%", equity[hand_idx] * 100.0);
        println!("  To act: {}", if current_player == 0 { "OOP" } else { "IP" });

        if current_player == player {
            // Hero's turn - show GTO strategy
            println!("\n📊 GTO Strategy for your hand:");

            let hand_cards = if is_oop { &oop_hands } else { &ip_hands };
            for (action_idx, action) in actions.iter().enumerate() {
                let freq = strategy[hand_idx + action_idx * hand_cards.len()];
                if freq > 0.01 {
                    println!("    {:2}. {:?} - {:.1}%",
                        action_idx + 1, action, freq * 100.0);
                }
            }
        } else {
            // Villain's turn
            println!("\n🎲 Villain's turn - choose their action:");
            for (action_idx, action) in actions.iter().enumerate() {
                println!("    {:2}. {:?}", action_idx + 1, action);
            }
        }

        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Commands: [1-{}] action | 'r' restart | 'q' quit", actions.len());
        print!("Your choice: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input {
            "q" | "quit" => {
                println!("\nGoodbye!");
                break;
            }
            "r" | "restart" => {
                game.back_to_root();
                action_history.clear();
                println!("\n✓ Restarted to root");
                continue;
            }
            _ => {
                if let Ok(choice) = input.parse::<usize>() {
                    if choice > 0 && choice <= actions.len() {
                        let action_idx = choice - 1;
                        action_history.push(format!("{:?}", actions[action_idx]));
                        game.play(action_idx);
                    } else {
                        println!("Invalid choice. Try again.");
                    }
                } else {
                    println!("Invalid input. Enter a number or command.");
                }
            }
        }
    }

    Ok(())
}
