/// Test turn-only and turn+river solving speed
///
/// Tests:
/// 1. Turn-only solve after flop action (filtered ranges)
/// 2. River-only solve after turn action (more filtered ranges)

use postflop_solver::*;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Turn + River Live Solve Speed Test ===\n");

    println!("Scenario: Root in DB, solve turn and river live\n");

    println!("Test 1: Turn-only solve after flop bet-call");
    println!("--------------------------------------------------------");
    test_turn_solve()?;

    println!("\n\nTest 2: River-only solve after turn check-check");
    println!("--------------------------------------------------------");
    test_river_solve()?;

    Ok(())
}

fn test_turn_solve() -> Result<(), Box<dyn std::error::Error>> {
    // After flop bet-call, ranges are somewhat filtered
    // OOP bet flop, IP called - both have value + draws
    let oop_range = "AA,KK,QQ,JJ,TT,AKs,AQs,AJs,ATs,KQs,QJs,JTs"; // ~110 combos
    let ip_range = "AA,KK,QQ,JJ,TT,99,88,77,AKs,AQs,AJs,ATs,KQs,KJs,QJs,JTs"; // ~150 combos

    println!("Board: AsKsQs-7h (specific turn card)");
    println!("Filtered ranges after flop bet-call:");
    println!("  OOP: {} (~110 combos)", oop_range);
    println!("  IP: {} (~150 combos)", ip_range);
    println!();
    println!("Solving: Turn + all 43 possible rivers");
    println!("Starting pot: 1100¢, Stack: 8650¢");
    println!();

    let card_config = CardConfig {
        range: [
            oop_range.parse()?,
            ip_range.parse()?,
        ],
        flop: flop_from_str("AsKsQs")?,
        turn: card_from_str("7h")?, // Must specify turn
        river: NOT_DEALT, // River will vary
    };

    // Turn + river bet sizes
    let turn_bet_sizes = BetSizeOptions::try_from(("50%, 100%", "2x"))?;
    let river_bet_sizes = BetSizeOptions::try_from(("75%, a", "2x"))?;

    let tree_config = TreeConfig {
        initial_state: BoardState::Turn, // Start at turn!
        starting_pot: 1100,
        effective_stack: 8650,
        rake_rate: 0.0,
        rake_cap: 0.0,
        flop_bet_sizes: [BetSizeOptions::try_from(("50%", "2x"))?, BetSizeOptions::try_from(("50%", "2x"))?],
        turn_bet_sizes: [turn_bet_sizes.clone(), turn_bet_sizes],
        river_bet_sizes: [river_bet_sizes.clone(), river_bet_sizes],
        turn_donk_sizes: None,
        river_donk_sizes: None,
        add_allin_threshold: 1.5,
        force_allin_threshold: 0.15,
        merging_threshold: 0.1,
    };

    println!("Building turn+river tree...");
    let build_start = Instant::now();
    let action_tree = ActionTree::new(tree_config)?;
    let mut game = PostFlopGame::with_config(card_config, action_tree)?;
    let build_time = build_start.elapsed();

    let (_, mem_usage) = game.memory_usage();
    println!("  Build time: {:.2}s", build_time.as_secs_f64());
    println!("  Memory: {:.2} MB", mem_usage as f64 / (1024.0 * 1024.0));

    game.allocate_memory(true);

    println!("\nSolving turn+river tree (all 44 turns)...");
    let solve_start = Instant::now();
    let exploitability = solve(&mut game, 200, 5.0, false);
    let solve_time = solve_start.elapsed();

    println!("✓ Turn+river solve complete!");
    println!("  Solve time: {:.2}s", solve_time.as_secs_f64());
    println!("  Exploitability: {:.2}¢", exploitability);
    println!("  Total time: {:.2}s", (build_time + solve_time).as_secs_f64());

    game.back_to_root();
    game.cache_normalized_weights();
    let actions = game.available_actions();
    println!("\n  Available turn actions: {:?}", actions);

    Ok(())
}

fn test_river_solve() -> Result<(), Box<dyn std::error::Error>> {
    // After flop bet-call, turn check-check
    // Ranges are moderately filtered
    let oop_range = "AA,KK,QQ,AKs,AQs,KQs"; // ~35 combos
    let ip_range = "AA,KK,QQ,JJ,TT,AKs,AQs,AJs,KQs"; // ~55 combos

    println!("Board: AsKsQs-7h-2d (specific river)");
    println!("Filtered ranges after flop bet-call, turn check-check:");
    println!("  OOP: {} (~35 combos)", oop_range);
    println!("  IP: {} (~55 combos)", ip_range);
    println!();
    println!("Solving: Single river card");
    println!("Starting pot: 1100¢, Stack: 8650¢");
    println!();

    let card_config = CardConfig {
        range: [
            oop_range.parse()?,
            ip_range.parse()?,
        ],
        flop: flop_from_str("AsKsQs")?,
        turn: card_from_str("7h")?,
        river: card_from_str("2d")?, // Must specify river
    };

    let river_bet_sizes = BetSizeOptions::try_from(("75%, a", "2x"))?;

    let tree_config = TreeConfig {
        initial_state: BoardState::River,
        starting_pot: 1100,
        effective_stack: 8650,
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

    println!("Building river tree (all 43 rivers)...");
    let build_start = Instant::now();
    let action_tree = ActionTree::new(tree_config)?;
    let mut game = PostFlopGame::with_config(card_config, action_tree)?;
    let build_time = build_start.elapsed();

    let (_, mem_usage) = game.memory_usage();
    println!("  Build time: {:.2}s", build_time.as_secs_f64());
    println!("  Memory: {:.2} MB", mem_usage as f64 / (1024.0 * 1024.0));

    game.allocate_memory(true);

    println!("\nSolving all river runouts...");
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
    println!("\n  Available river actions: {:?}", actions);

    Ok(())
}
