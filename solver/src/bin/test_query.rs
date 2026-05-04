use postflop_solver::*;
use std::fs;

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
    println!("=== Complete Workflow Test: Solve → Save → Load → Query ===\n");

    let output_path = "../test_query_output.bin";

    // Step 1: Solve
    println!("Step 1: Solving...");
    let card_config = CardConfig {
        range: [
            "AA:1.0,KK:1.0,QQ:1.0,JJ:1.0".parse()?,  // OOP
            "AA:1.0,KK:1.0,QQ:1.0,JJ:1.0".parse()?,  // IP
        ],
        flop: flop_from_str("KhQs6h")?,
        turn: NOT_DEALT,
        river: NOT_DEALT,
    };

    let bet_sizes = BetSizeOptions::try_from(("50%", "2x"))?;
    let tree_config = TreeConfig {
        initial_state: BoardState::Flop,
        starting_pot: 550,
        effective_stack: 9750,
        rake_rate: 0.0,
        rake_cap: 0.0,
        flop_bet_sizes: [bet_sizes.clone(), bet_sizes.clone()],
        turn_bet_sizes: [bet_sizes.clone(), bet_sizes.clone()],
        river_bet_sizes: [bet_sizes.clone(), bet_sizes],
        turn_donk_sizes: None,
        river_donk_sizes: None,
        add_allin_threshold: 1.5,
        force_allin_threshold: 0.15,
        merging_threshold: 0.1,
    };

    let action_tree = ActionTree::new(tree_config)?;
    let mut game = PostFlopGame::with_config(card_config, action_tree)?;
    game.allocate_memory(false);
    solve(&mut game, 100, 0.1, false);
    println!("  ✓ Solved\n");

    // Step 2: Save
    println!("Step 2: Saving to {}...", output_path);
    save_data_to_file(&game, "test query workflow", output_path, None)?;
    let file_size = fs::metadata(output_path)?.len();
    println!("  ✓ Saved {:.2} MB\n", file_size as f64 / (1024.0 * 1024.0));

    drop(game);  // Ensure original is gone

    // Step 3: Load
    println!("Step 3: Loading from file...");
    let (mut loaded_game, _): (PostFlopGame, String) = load_data_from_file(output_path, None)?;
    println!("  ✓ Loaded\n");

    // Step 4: Query specific hand
    println!("Step 4: Query specific hand (AsAd - overpair)...");
    loaded_game.back_to_root();
    loaded_game.cache_normalized_weights();

    // Find AsAd for OOP player
    let oop_cards = loaded_game.private_cards(0);
    let hand_as_ad = oop_cards.iter()
        .position(|&(c1, c2)| {
            (card_to_string(c1) == "As" && card_to_string(c2) == "Ad") ||
            (card_to_string(c1) == "Ad" && card_to_string(c2) == "As")
        });

    if let Some(hand_idx) = hand_as_ad {
        let actions = loaded_game.available_actions();
        let strategy = loaded_game.strategy();
        let equity = loaded_game.equity(0);

        println!("\n  Hand: As Ad (OOP at root)");
        println!("  Equity: {:.1}%", equity[hand_idx] * 100.0);
        println!("\n  GTO Strategy:");

        for (action_idx, action) in actions.iter().enumerate() {
            let freq = strategy[hand_idx + action_idx * oop_cards.len()];
            if freq > 0.001 {
                println!("    {:?}: {:.1}%", action, freq * 100.0);
            }
        }
    } else {
        println!("  ✗ Hand AsAd not found in range");
    }

    // Cleanup
    fs::remove_file(output_path)?;

    println!("\n🎉 COMPLETE SUCCESS!");
    println!("✓ Solved full tree");
    println!("✓ Saved to disk");
    println!("✓ Loaded from disk");
    println!("✓ Queried specific hand strategy");
    println!("\n✅ Ready for EC2!");

    Ok(())
}
