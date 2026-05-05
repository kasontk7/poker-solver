use postflop_solver::*;
use std::fs;
use std::time::Instant;
use std::env;

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
    let total_start = Instant::now();

    // Get board from command line argument
    let args: Vec<String> = env::args().collect();
    let board_str = if args.len() > 1 {
        &args[1]
    } else {
        "KhQs6h" // Default
    };

    println!("=== Poker Solver v1.1 ===");
    println!("Scenario: BTN RFI vs BB call");
    println!("Board: {} (full tree)", board_str);
    println!();

    // Load ranges
    println!("Loading ranges...");
    let ip_range = load_range_from_file("ranges/gto/rfi/BTN_RFI_vs_BB_defend/btn.txt")?;
    let oop_range = load_range_from_file("ranges/gto/rfi/BTN_RFI_vs_BB_defend/bb.txt")?;

    println!("  IP (BTN): {} chars", ip_range.len());
    println!("  OOP (BB): {} chars", oop_range.len());
    println!();

    // Parse board
    println!("Configuring game...");
    let card_config = CardConfig {
        range: [
            oop_range.parse()?,
            ip_range.parse()?,
        ],
        flop: flop_from_str(board_str)?,
        turn: NOT_DEALT,
        river: NOT_DEALT,
    };

    // Bet sizes configuration
    let flop_bet_sizes = BetSizeOptions::try_from(("50%, 100%", "3x, 5x"))?;
    let turn_bet_sizes = BetSizeOptions::try_from(("50%, 100%, 150%", "3x, 5x"))?;
    let river_bet_sizes = BetSizeOptions::try_from(("75%, 150%, a", "3x, 5x"))?;

    let donk_sizes = DonkSizeOptions::try_from("50%")?;

    // Tree configuration
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

    println!("  Starting pot: ${:.2} (5.5bb)", tree_config.starting_pot as f32 / 100.0);
    println!("  Effective stack: ${:.2} (97.5bb)", tree_config.effective_stack as f32 / 100.0);
    println!("  Rake: {:.1}% capped at ${:.2}", tree_config.rake_rate * 100.0, tree_config.rake_cap / 100.0);
    println!();

    // Build game tree
    println!("Building game tree...");
    let action_tree = ActionTree::new(tree_config)?;
    let mut game = PostFlopGame::with_config(card_config, action_tree)?;

    // Check memory usage
    let (mem_usage, mem_usage_compressed) = game.memory_usage();
    println!("  Memory (32-bit float): {:.2} MB", mem_usage as f64 / (1024.0 * 1024.0));
    println!("  Memory (16-bit compressed): {:.2} MB", mem_usage_compressed as f64 / (1024.0 * 1024.0));
    println!();

    // Allocate memory with 16-bit compression
    println!("Allocating memory with 16-bit compression...");
    game.allocate_memory(true);
    println!();

    // Solve the game
    println!("Solving game...");
    let max_iterations = 500;
    let target_exploitability = 5.0;
    println!("  Max iterations: {}", max_iterations);
    println!("  Target exploitability: {:.2} cents (~1% of pot)", target_exploitability);
    println!();

    let solve_start = Instant::now();
    let exploitability = solve(&mut game, max_iterations, target_exploitability, true);
    let solve_duration = solve_start.elapsed();

    println!();
    println!("✓ Solve complete!");
    println!("  Final exploitability: {:.2} cents", exploitability);
    println!("  Solve time: {:.2} minutes ({:.1} hours)",
        solve_duration.as_secs_f64() / 60.0,
        solve_duration.as_secs_f64() / 3600.0);
    println!();

    // Save the solved game tree
    println!("Saving solution...");
    let output_path = format!("solutions/v1.1_{}.bin", board_str);
    fs::create_dir_all("solutions")?;

    let memo = format!("v1.1 BTN RFI vs BB defend, {} full tree", board_str);
    save_data_to_file(&game, &memo, &output_path, None)?;
    let file_size = fs::metadata(&output_path)?.len();
    println!("  Saved to: {}", output_path);
    println!("  File size: {:.2} MB", file_size as f64 / (1024.0 * 1024.0));

    let total_duration = total_start.elapsed();

    println!();
    println!("=== Done! ===");
    println!("Total time: {:.2} minutes ({:.1} hours)",
        total_duration.as_secs_f64() / 60.0,
        total_duration.as_secs_f64() / 3600.0);

    Ok(())
}
