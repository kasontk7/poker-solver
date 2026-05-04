use postflop_solver::*;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Save/Load Round-trip ===\n");

    // Minimal full tree
    let card_config = CardConfig {
        range: [
            "AA:1.0,KK:1.0".parse()?,
            "AA:1.0,KK:1.0".parse()?,
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

    println!("Step 1: Solve...");
    let action_tree = ActionTree::new(tree_config)?;
    let mut game = PostFlopGame::with_config(card_config, action_tree)?;
    game.allocate_memory(false);
    let exploitability = solve(&mut game, 50, 0.1, false);
    println!("  ✓ Exploitability: {:.2}", exploitability);

    // Get a sample strategy
    game.back_to_root();
    game.cache_normalized_weights();
    let original_strategy = game.strategy().to_vec();
    let original_equity = game.equity(0).to_vec();

    println!("\nStep 2: Save to file...");
    let output_path = "../test_roundtrip.bin";

    save_data_to_file(&game, "round-trip test", output_path, None)?;

    let file_size = fs::metadata(output_path)?.len();
    println!("  ✓ Saved {} bytes ({:.2} MB)", file_size, file_size as f64 / (1024.0 * 1024.0));

    // Drop the original game
    drop(game);

    println!("\nStep 3: Load from file...");
    let (loaded_game, memo): (PostFlopGame, String) = load_data_from_file(output_path, None)?;
    println!("  ✓ Loaded (memo: {})", memo);

    // Verify loaded game
    println!("\nStep 4: Verify loaded data...");
    let mut loaded_game = loaded_game;
    loaded_game.back_to_root();
    loaded_game.cache_normalized_weights();
    let loaded_strategy = loaded_game.strategy().to_vec();
    let loaded_equity = loaded_game.equity(0).to_vec();

    let strategy_match = original_strategy == loaded_strategy;
    let equity_match = original_equity == loaded_equity;

    println!("  Strategy matches: {}", if strategy_match { "✓ YES" } else { "✗ NO" });
    println!("  Equity matches: {}", if equity_match { "✓ YES" } else { "✗ NO" });

    // Cleanup
    fs::remove_file(output_path)?;

    if strategy_match && equity_match {
        println!("\n🎉 SUCCESS! Save/load round-trip works perfectly!");
        println!("✓ We can use this for EC2");
        Ok(())
    } else {
        Err("Round-trip verification failed".into())
    }
}
