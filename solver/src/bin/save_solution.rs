use postflop_solver::*;
use std::fs;
use std::io::Write;

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
        0 => 's',
        1 => 'h',
        2 => 'd',
        3 => 'c',
        _ => '?',
    };
    format!("{}{}", rank, suit)
}

fn save_solution_as_json(
    game: &mut PostFlopGame,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut output = fs::File::create(output_path)?;

    writeln!(output, "{{")?;
    writeln!(output, r#"  "board": "Kh Qs 6h 9d 3c","#)?;
    writeln!(output, r#"  "scenario": "BTN_RFI_vs_BB_call","#)?;
    writeln!(output, r#"  "pot_bb": 5.5,"#)?;
    writeln!(output, r#"  "stack_bb": 97.5,"#)?;
    writeln!(output, r#"  "nodes": ["#)?;

    // We'll export the root node strategy
    game.back_to_root();
    game.cache_normalized_weights();

    // Get OOP (BB) hands
    let oop_cards = game.private_cards(0);
    let oop_hands: Vec<String> = oop_cards
        .iter()
        .map(|&h| format!("{}{}", card_to_string(h.0), card_to_string(h.1)))
        .collect();

    // Get IP (BTN) hands
    let ip_cards = game.private_cards(1);
    let ip_hands: Vec<String> = ip_cards
        .iter()
        .map(|&h| format!("{}{}", card_to_string(h.0), card_to_string(h.1)))
        .collect();

    // Root node - OOP acts first
    let actions = game.available_actions();
    let strategy = game.strategy();
    let equity_oop = game.equity(0);
    let ev_oop = game.expected_values(0);
    let equity_ip = game.equity(1);
    let ev_ip = game.expected_values(1);

    writeln!(output, "    {{")?;
    writeln!(output, r#"      "node_id": "root","#)?;
    writeln!(output, r#"      "player_to_act": "OOP","#)?;
    writeln!(output, r#"      "actions": ["#)?;
    for (i, action) in actions.iter().enumerate() {
        let comma = if i < actions.len() - 1 { "," } else { "" };
        writeln!(output, r#"        "{}"{}"#, format!("{:?}", action), comma)?;
    }
    writeln!(output, "      ],")?;

    // OOP strategy
    writeln!(output, r#"      "oop_strategy": {{"#)?;
    for (hand_idx, hand) in oop_hands.iter().enumerate() {
        let mut freqs = Vec::new();
        for action_idx in 0..actions.len() {
            freqs.push(strategy[hand_idx + action_idx * oop_cards.len()]);
        }
        let comma = if hand_idx < oop_hands.len() - 1 { "," } else { "" };
        writeln!(
            output,
            r#"        "{}": [{:.4}, {:.4}, {:.4}, {:.4}, {:.4}]{}"#,
            hand,
            freqs.get(0).unwrap_or(&0.0),
            freqs.get(1).unwrap_or(&0.0),
            freqs.get(2).unwrap_or(&0.0),
            freqs.get(3).unwrap_or(&0.0),
            freqs.get(4).unwrap_or(&0.0),
            comma
        )?;
    }
    writeln!(output, "      }},")?;

    // OOP equity and EV
    writeln!(output, r#"      "oop_equity": {{"#)?;
    for (hand_idx, hand) in oop_hands.iter().enumerate() {
        let comma = if hand_idx < oop_hands.len() - 1 { "," } else { "" };
        writeln!(
            output,
            r#"        "{}": {:.4}{}"#,
            hand, equity_oop[hand_idx], comma
        )?;
    }
    writeln!(output, "      }},")?;

    writeln!(output, r#"      "oop_ev": {{"#)?;
    for (hand_idx, hand) in oop_hands.iter().enumerate() {
        let comma = if hand_idx < oop_hands.len() - 1 { "," } else { "" };
        writeln!(
            output,
            r#"        "{}": {:.2}{}"#,
            hand, ev_oop[hand_idx], comma
        )?;
    }
    writeln!(output, "      }},")?;

    // IP equity and EV
    writeln!(output, r#"      "ip_equity": {{"#)?;
    for (hand_idx, hand) in ip_hands.iter().enumerate() {
        let comma = if hand_idx < ip_hands.len() - 1 { "," } else { "" };
        writeln!(
            output,
            r#"        "{}": {:.4}{}"#,
            hand, equity_ip[hand_idx], comma
        )?;
    }
    writeln!(output, "      }},")?;

    writeln!(output, r#"      "ip_ev": {{"#)?;
    for (hand_idx, hand) in ip_hands.iter().enumerate() {
        let comma = if hand_idx < ip_hands.len() - 1 { "," } else { "" };
        writeln!(
            output,
            r#"        "{}": {:.2}{}"#,
            hand, ev_ip[hand_idx], comma
        )?;
    }
    writeln!(output, "      }}")?;

    writeln!(output, "    }}")?;
    writeln!(output, "  ]")?;
    writeln!(output, "}}")?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Saving Solution to JSON ===\n");

    // Load and solve
    let ip_range = load_range_from_file("../ranges/gto/BTN/RFI.txt")?;
    let oop_range = load_range_from_file("../ranges/gto/BB/defend_vs_RFI_BTN.txt")?;

    let card_config = CardConfig {
        range: [oop_range.parse()?, ip_range.parse()?],
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

    println!("Solving...");
    let action_tree = ActionTree::new(tree_config)?;
    let mut game = PostFlopGame::with_config(card_config, action_tree)?;
    game.allocate_memory(false);

    let target_exploitability = game.tree_config().starting_pot as f32 * 0.005;
    solve(&mut game, 1000, target_exploitability, false);

    println!("✓ Solution ready\n");

    // Save as JSON
    let output_path = "../solutions/v1.0_KhQs6h_9d_3c_root.json";
    fs::create_dir_all("../solutions")?;

    println!("Saving to {}...", output_path);
    save_solution_as_json(&mut game, output_path)?;

    let file_size = fs::metadata(output_path)?.len();
    println!("✓ Saved {} bytes", file_size);
    println!("\nYou can now examine this JSON file to see the solution format.");

    Ok(())
}
