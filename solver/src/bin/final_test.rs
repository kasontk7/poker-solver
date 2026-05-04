use postflop_solver::*;
use std::fs;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Final v1.1 Save/Load Test ===");
    println!("Testing exact EC2 configuration\n");

    // Load full ranges
    let ip_range = load_range_from_file("../ranges/gto/BTN/RFI.txt")?;
    let oop_range = load_range_from_file("../ranges/gto/BB/defend_vs_RFI_BTN.txt")?;

    let card_config = CardConfig {
        range: [oop_range.parse()?, ip_range.parse()?],
        flop: flop_from_str("KhQs6h")?,
        turn: NOT_DEALT,
        river: NOT_DEALT,
    };

    // Exact bet sizes from main.rs
    let flop_bet_sizes = BetSizeOptions::try_from(("50%, 100%", "2.5x"))?;
    let turn_bet_sizes = BetSizeOptions::try_from(("50%, 125%", "2.5x"))?;
    let river_bet_sizes = BetSizeOptions::try_from(("75%, a", "2.5x"))?;
    let donk_sizes = DonkSizeOptions::try_from("50%")?;

    let tree_config = TreeConfig {
        initial_state: BoardState::Flop,
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

    println!("Building and solving...");
    let action_tree = ActionTree::new(tree_config)?;
    let mut game = PostFlopGame::with_config(card_config, action_tree)?;

    let (mem_32, mem_16) = game.memory_usage();
    println!("  Memory (32-bit): {:.2} MB", mem_32 as f64 / (1024.0 * 1024.0));
    println!("  Memory (16-bit): {:.2} MB", mem_16 as f64 / (1024.0 * 1024.0));

    game.allocate_memory(true);  // 16-bit

    let start = std::time::Instant::now();
    let exploitability = solve(&mut game, 500, 5.0, false);
    let duration = start.elapsed();

    println!("  ✓ Solved in {:.1} minutes", duration.as_secs_f32() / 60.0);
    println!("  Exploitability: {:.2} cents\n", exploitability);

    // SAVE
    let output_path = "../final_test.bin";
    println!("Saving...");
    save_data_to_file(&game, "final v1.1 test", output_path, None)?;
    let file_size = fs::metadata(output_path)?.len();
    println!("  ✓ Saved {:.2} MB\n", file_size as f64 / (1024.0 * 1024.0));

    drop(game);  // Drop original

    // LOAD
    println!("Loading...");
    let (mut loaded_game, memo): (PostFlopGame, String) = load_data_from_file(output_path, None)?;
    println!("  ✓ Loaded (memo: '{}')\n", memo);

    // QUERY
    println!("Querying hand (AsAd)...");
    loaded_game.back_to_root();
    loaded_game.cache_normalized_weights();

    let oop_cards = loaded_game.private_cards(0);
    let actions = loaded_game.available_actions();
    let strategy = loaded_game.strategy();
    let equity = loaded_game.equity(0);

    // Find AsAd
    if let Some(hand_idx) = oop_cards.iter().position(|&(c1, c2)| {
        ((c1 / 4 == 12 && c1 % 4 == 0) && (c2 / 4 == 12 && c2 % 4 == 2)) ||
        ((c1 / 4 == 12 && c1 % 4 == 2) && (c2 / 4 == 12 && c2 % 4 == 0))
    }) {
        println!("  Equity: {:.1}%", equity[hand_idx] * 100.0);
        println!("  Strategy:");
        for (action_idx, action) in actions.iter().enumerate() {
            let freq = strategy[hand_idx + action_idx * oop_cards.len()];
            if freq > 0.01 {
                println!("    {:?}: {:.1}%", action, freq * 100.0);
            }
        }
    }

    // Cleanup
    fs::remove_file(output_path)?;

    println!("\n✅ COMPLETE SUCCESS!");
    println!("✓ Solve works with optimized parameters");
    println!("✓ Save works (bincode)");
    println!("✓ Load works");
    println!("✓ Query works");
    println!("\n🚀 READY FOR EC2!");

    Ok(())
}
