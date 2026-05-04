use postflop_solver::*;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Full Tree Solve (Minimal) ===");
    println!("Turn: NOT_DEALT, River: NOT_DEALT");
    println!("Ranges: Just AA, KK (ultra minimal)");
    println!();

    // Minimal ranges - just 2 hands each
    let card_config = CardConfig {
        range: [
            "AA:1.0,KK:1.0".parse()?,  // OOP: just AA and KK
            "AA:1.0,KK:1.0".parse()?,  // IP: just AA and KK
        ],
        flop: flop_from_str("KhQs6h")?,
        turn: NOT_DEALT,   // Full tree
        river: NOT_DEALT,  // Full tree
    };

    // Minimal bet sizes - just 1 option per street
    let flop_bet_sizes = BetSizeOptions::try_from(("50%", "2x"))?;
    let turn_bet_sizes = BetSizeOptions::try_from(("50%", "2x"))?;
    let river_bet_sizes = BetSizeOptions::try_from(("50%", "2x"))?;

    let tree_config = TreeConfig {
        initial_state: BoardState::Flop,
        starting_pot: 550,
        effective_stack: 9750,
        rake_rate: 0.0,
        rake_cap: 0.0,
        flop_bet_sizes: [flop_bet_sizes.clone(), flop_bet_sizes.clone()],
        turn_bet_sizes: [turn_bet_sizes.clone(), turn_bet_sizes.clone()],
        river_bet_sizes: [river_bet_sizes.clone(), river_bet_sizes],
        turn_donk_sizes: None,
        river_donk_sizes: None,
        add_allin_threshold: 1.5,
        force_allin_threshold: 0.15,
        merging_threshold: 0.1,
    };

    println!("Building game tree...");
    let action_tree = ActionTree::new(tree_config)?;
    let mut game = PostFlopGame::with_config(card_config, action_tree)?;

    let (mem_usage, mem_usage_compressed) = game.memory_usage();
    println!("  Memory (32-bit): {:.2} MB", mem_usage as f64 / (1024.0 * 1024.0));
    println!("  Memory (16-bit): {:.2} MB", mem_usage_compressed as f64 / (1024.0 * 1024.0));
    println!();

    println!("Allocating memory...");
    game.allocate_memory(false);
    println!();

    println!("Solving (should take 10-30 seconds)...");
    let start = std::time::Instant::now();
    let exploitability = solve(&mut game, 100, 0.1, true);
    let duration = start.elapsed();

    println!();
    println!("✓ Solve complete!");
    println!("  Time: {:.1}s", duration.as_secs_f32());
    println!("  Final exploitability: {:.2} cents", exploitability);
    println!();

    // Test save using save_data_to_file
    let output_path = "../test_full_tree_output.bin";
    println!("Attempting to save to {}...", output_path);

    // Check if save_data_to_file is available
    match save_data_to_file(&game, "minimal full tree test", output_path, None) {
        Ok(_) => {
            let file_size = fs::metadata(output_path)?.len();
            println!("✓ SAVE SUCCESSFUL!");
            println!("  File: {}", output_path);
            println!("  Size: {:.2} MB", file_size as f64 / (1024.0 * 1024.0));

            // Cleanup
            fs::remove_file(output_path)?;
            println!("\n✓ Cleanup complete");
        }
        Err(e) => {
            println!("✗ SAVE FAILED: {}", e);
            println!("\nThis confirms save_data_to_file is not available.");
            println!("We need to implement custom JSON serialization.");
        }
    }

    Ok(())
}
