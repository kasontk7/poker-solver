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
    println!("=== Optimized Solve Test ===");
    println!("16-bit compression + minimal bet sizes + full ranges\n");

    // Load FULL realistic ranges
    println!("Loading full ranges...");
    let ip_range = load_range_from_file("../ranges/gto/BTN/RFI.txt")?;
    let oop_range = load_range_from_file("../ranges/gto/BB/defend_vs_RFI_BTN.txt")?;
    println!("  IP range: {} chars", ip_range.len());
    println!("  OOP range: {} chars", oop_range.len());
    println!();

    // Full tree
    let card_config = CardConfig {
        range: [
            oop_range.parse()?,
            ip_range.parse()?,
        ],
        flop: flop_from_str("KhQs6h")?,
        turn: NOT_DEALT,
        river: NOT_DEALT,
    };

    // OPTIMIZED: Minimal bet sizes
    // Flop: 50%, all-in
    // Turn: 50%, all-in
    // River: 75%, all-in
    println!("Bet sizing (optimized):");
    println!("  Flop: 50%, all-in");
    println!("  Turn: 50%, all-in");
    println!("  River: 75%, all-in");
    println!();

    let flop_bet_sizes = BetSizeOptions::try_from(("50%, a", "2.5x"))?;
    let turn_bet_sizes = BetSizeOptions::try_from(("50%, a", "2.5x"))?;
    let river_bet_sizes = BetSizeOptions::try_from(("75%, a", "2.5x"))?;

    // No donk bets (simplification)
    let tree_config = TreeConfig {
        initial_state: BoardState::Flop,
        starting_pot: 550,
        effective_stack: 9750,
        rake_rate: 0.05,
        rake_cap: 300.0,
        flop_bet_sizes: [flop_bet_sizes.clone(), flop_bet_sizes.clone()],
        turn_bet_sizes: [turn_bet_sizes.clone(), turn_bet_sizes.clone()],
        river_bet_sizes: [river_bet_sizes.clone(), river_bet_sizes],
        turn_donk_sizes: None,  // Disabled
        river_donk_sizes: None, // Disabled
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

    // OPTIMIZATION: Use 16-bit compression
    println!("Allocating memory with 16-bit compression...");
    game.allocate_memory(true);  // true = 16-bit compression
    println!();

    println!("Solving...");
    let start = std::time::Instant::now();
    let max_iterations = 500;
    let target_exploitability = 2.75;

    let exploitability = solve(&mut game, max_iterations, target_exploitability, true);
    let duration = start.elapsed();

    println!();
    println!("✓ Solve complete!");
    println!("  Time: {:.1} seconds ({:.1} minutes)", duration.as_secs_f32(), duration.as_secs_f32() / 60.0);
    println!("  Final exploitability: {:.2} cents (target: {:.2})", exploitability, target_exploitability);
    println!();

    // Save
    println!("Saving...");
    let output_path = "../test_optimized_output.bin";
    save_data_to_file(&game, "optimized test", output_path, None)?;

    let file_size = fs::metadata(output_path)?.len();
    println!("  ✓ Saved: {:.2} MB", file_size as f64 / (1024.0 * 1024.0));
    println!();

    // Test load
    println!("Testing load...");
    let (loaded_game, _): (PostFlopGame, String) = load_data_from_file(output_path, None)?;
    println!("  ✓ Loaded successfully");
    drop(loaded_game);

    // Cleanup
    fs::remove_file(output_path)?;

    println!();
    println!("=== Summary ===");
    println!("Solve time: {:.1} minutes", duration.as_secs_f32() / 60.0);
    println!("File size: {:.2} MB", file_size as f64 / (1024.0 * 1024.0));
    println!("Exploitability: {:.2} cents", exploitability);
    println!();

    // Extrapolate to full v1.2
    let solves_total = 6992;
    let total_time_hours = (duration.as_secs_f32() / 60.0) * solves_total as f32 / 60.0;
    let total_size_tb = (file_size as f64 / (1024.0 * 1024.0 * 1024.0)) * solves_total as f64;

    println!("=== v1.2 Estimates (6,992 solves) ===");
    println!("Total time: {:.0} hours ({:.1} days with 100 parallel instances)",
             total_time_hours, total_time_hours / 100.0 / 24.0);
    println!("Total storage: {:.1} TB", total_size_tb);
    println!("Monthly S3 cost: ${:.0} ($0.023/GB/month)", total_size_tb * 1024.0 * 0.023);

    Ok(())
}
