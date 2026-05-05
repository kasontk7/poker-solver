/// Test river-only solving speed
///
/// Tests two scenarios:
/// 1. River solve after flop+turn action (no post-turn action yet)
/// 2. River solve after flop+turn+turn-action

use postflop_solver::*;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== River-Only Solve Speed Test ===\n");

    // Scenario: AsKsQs-7h-2d
    // After flop betting and turn action, ranges are filtered

    println!("Test 1: River solve with filtered ranges (post-turn, pre-river-action)");
    println!("--------------------------------------------------------");
    test_river_solve_1()?;

    println!("\n\nTest 2: River solve after turn bet (even more filtered)");
    println!("--------------------------------------------------------");
    test_river_solve_2()?;

    Ok(())
}

fn test_river_solve_1() -> Result<(), Box<dyn std::error::Error>> {
    // Simulating ranges after: Flop bet-call, turn check-check
    // These are "value heavy" ranges that would realistically call flop
    let oop_range = "AA,KK,QQ,AKs,AQs,KQs"; // ~35 combos
    let ip_range = "AA,KK,QQ,JJ,TT,AKs,AQs,AJs,KQs"; // ~55 combos

    println!("Board: AsKsQs-7h-2d");
    println!("Filtered ranges after flop+turn action:");
    println!("  OOP: {} (value hands that bet-call flop)", oop_range);
    println!("  IP: {} (hands that called flop bet)", ip_range);
    println!("\nStarting pot: 1200¢, Stack: 8550¢");
    println!();

    let card_config = CardConfig {
        range: [
            oop_range.parse()?,
            ip_range.parse()?,
        ],
        flop: flop_from_str("AsKsQs")?,
        turn: card_from_str("7h")?,
        river: card_from_str("2d")?,
    };

    // Simple river bet sizing
    let river_bet_sizes = BetSizeOptions::try_from(("75%, a", "2x"))?;

    let tree_config = TreeConfig {
        initial_state: BoardState::River, // Start at river!
        starting_pot: 1200,
        effective_stack: 8550,
        rake_rate: 0.0,
        rake_cap: 0.0,
        flop_bet_sizes: [BetSizeOptions::try_from(("50%", "2x"))?, BetSizeOptions::try_from(("50%", "2x"))?],
        turn_bet_sizes: [BetSizeOptions::try_from(("50%", "2x"))?, BetSizeOptions::try_from(("50%", "2x"))?],
        river_bet_sizes: [river_bet_sizes.clone(), river_bet_sizes],
        turn_donk_sizes: None,
        river_donk_sizes: None,
        add_allin_threshold: 1.5,
        force_allin_threshold: 0.15,
        merging_threshold: 0.1,
    };

    println!("Building river-only tree...");
    let build_start = Instant::now();
    let action_tree = ActionTree::new(tree_config)?;
    let mut game = PostFlopGame::with_config(card_config, action_tree)?;
    let build_time = build_start.elapsed();

    let (_, mem_usage) = game.memory_usage();
    println!("  Build time: {:.2}s", build_time.as_secs_f64());
    println!("  Memory: {:.2} MB", mem_usage as f64 / (1024.0 * 1024.0));

    game.allocate_memory(true);

    println!("\nSolving river...");
    let solve_start = Instant::now();
    let exploitability = solve(&mut game, 200, 5.0, false); // No verbose output
    let solve_time = solve_start.elapsed();

    println!("✓ River solve complete!");
    println!("  Solve time: {:.2}s", solve_time.as_secs_f64());
    println!("  Exploitability: {:.2}¢", exploitability);
    println!("  Total time: {:.2}s", (build_time + solve_time).as_secs_f64());

    // Show a sample strategy
    game.back_to_root();
    game.cache_normalized_weights();
    let actions = game.available_actions();
    println!("\n  Available actions: {:?}", actions);

    Ok(())
}

fn test_river_solve_2() -> Result<(), Box<dyn std::error::Error>> {
    // Even more filtered: After turn bet-call
    // Only strong hands continue here
    let oop_range = "AA,KK,AKs,AQs"; // ~20 combos (value betting turn)
    let ip_range = "AA,KK,QQ,AKs,AQs"; // ~28 combos (calling turn bet)

    println!("Board: AsKsQs-7h-2d");
    println!("Filtered ranges after flop bet-call, turn bet-call:");
    println!("  OOP: {} (strong value)", oop_range);
    println!("  IP: {} (strong calls)", ip_range);
    println!("\nStarting pot: 1750¢, Stack: 8000¢");
    println!();

    let card_config = CardConfig {
        range: [
            oop_range.parse()?,
            ip_range.parse()?,
        ],
        flop: flop_from_str("AsKsQs")?,
        turn: card_from_str("7h")?,
        river: card_from_str("2d")?,
    };

    let river_bet_sizes = BetSizeOptions::try_from(("75%, a", "2x"))?;

    let tree_config = TreeConfig {
        initial_state: BoardState::River,
        starting_pot: 1750,
        effective_stack: 8000,
        rake_rate: 0.0,
        rake_cap: 0.0,
        flop_bet_sizes: [BetSizeOptions::try_from(("50%", "2x"))?, BetSizeOptions::try_from(("50%", "2x"))?],
        turn_bet_sizes: [BetSizeOptions::try_from(("50%", "2x"))?, BetSizeOptions::try_from(("50%", "2x"))?],
        river_bet_sizes: [river_bet_sizes.clone(), river_bet_sizes],
        turn_donk_sizes: None,
        river_donk_sizes: None,
        add_allin_threshold: 1.5,
        force_allin_threshold: 0.15,
        merging_threshold: 0.1,
    };

    println!("Building river-only tree...");
    let build_start = Instant::now();
    let action_tree = ActionTree::new(tree_config)?;
    let mut game = PostFlopGame::with_config(card_config, action_tree)?;
    let build_time = build_start.elapsed();

    let (_, mem_usage) = game.memory_usage();
    println!("  Build time: {:.2}s", build_time.as_secs_f64());
    println!("  Memory: {:.2} MB", mem_usage as f64 / (1024.0 * 1024.0));

    game.allocate_memory(true);

    println!("\nSolving river...");
    let solve_start = Instant::now();
    let exploitability = solve(&mut game, 200, 5.0, false);
    let solve_time = solve_start.elapsed();

    println!("✓ River solve complete!");
    println!("  Solve time: {:.2}s", solve_time.as_secs_f64());
    println!("  Exploitability: {:.2}¢", exploitability);
    println!("  Total time: {:.2}s", (build_time + solve_time).as_secs_f64());

    game.back_to_root();
    game.cache_normalized_weights();
    let actions = game.available_actions();
    println!("\n  Available actions: {:?}", actions);

    Ok(())
}
