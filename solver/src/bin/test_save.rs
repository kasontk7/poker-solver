use postflop_solver::*;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing save functionality...\n");

    // Minimal test: single turn/river, tiny solve
    let card_config = CardConfig {
        range: [
            "AA:1.0,KK:1.0,QQ:1.0".parse()?,
            "AA:1.0,KK:1.0,QQ:1.0".parse()?,
        ],
        flop: flop_from_str("KhQs6h")?,
        turn: card_from_str("9d")?,
        river: card_from_str("3c")?,
    };

    let bet_sizes = BetSizeOptions::try_from(("50%", "2x"))?;

    let tree_config = TreeConfig {
        initial_state: BoardState::River,
        starting_pot: 550,
        effective_stack: 9750,
        rake_rate: 0.0,
        rake_cap: 0.0,
        flop_bet_sizes: [bet_sizes.clone(), bet_sizes.clone()],
        turn_bet_sizes: [bet_sizes.clone(), bet_sizes.clone()],
        river_bet_sizes: [bet_sizes.clone(), bet_sizes.clone()],
        turn_donk_sizes: None,
        river_donk_sizes: None,
        add_allin_threshold: 1.5,
        force_allin_threshold: 0.15,
        merging_threshold: 0.1,
    };

    let action_tree = ActionTree::new(tree_config)?;
    let mut game = PostFlopGame::with_config(card_config, action_tree)?;
    game.allocate_memory(false);

    println!("Solving (should take <1 second)...");
    solve(&mut game, 100, 0.1, false);
    println!("✓ Solved\n");

    // Test save
    let output_path = "../test_output.bin";
    println!("Attempting to save to {}...", output_path);

    save_data_to_file(&game, "test save", output_path, None)?;

    let file_size = fs::metadata(output_path)?.len();
    println!("✓ SAVE SUCCESSFUL!");
    println!("  File: {}", output_path);
    println!("  Size: {} bytes", file_size);

    // Cleanup
    fs::remove_file(output_path)?;
    println!("\n✓ Test cleanup complete");

    Ok(())
}
