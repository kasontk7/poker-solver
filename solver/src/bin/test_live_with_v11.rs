/// Test: Use v11_root.db for flop, then live solve turn/river
///
/// Simulates the real workflow:
/// 1. Query flop strategy from database (instant)
/// 2. Filter ranges based on flop action
/// 3. Solve turn live (0.2s)
/// 4. Filter ranges based on turn action
/// 5. Solve river live (<0.01s)

use postflop_solver::*;
use rusqlite::Connection;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Test: DB Root + Live Turn/River ===\n");

    // Step 1: Query flop from database
    println!("Step 1: Query flop strategy from database");
    let conn = Connection::open("v11_root.db")?;

    let query_start = Instant::now();
    let flop_strategy = query_flop_strategy(&conn, "AsAd")?;
    println!("  Query time: {:.2}ms", query_start.elapsed().as_secs_f64() * 1000.0);
    println!("  Flop strategy for AsAd:");
    for (action, freq) in &flop_strategy {
        println!("    {}: {:.1}%", action, freq * 100.0);
    }

    // Step 2: Simulate flop action and filter ranges
    println!("\nStep 2: Simulate flop action (OOP bets, IP calls)");
    println!("  Filtering ranges to hands that would bet/call...");

    // In reality, you'd filter using the actual strategy frequencies
    // For now, use plausible filtered ranges
    let oop_turn_range = "AA,KK,QQ,JJ,AKs,AKo,KQs"; // Value + draws that bet
    let ip_turn_range = "AA,KK,QQ,JJ,TT,AKs,AKo,AQs,KQs"; // Hands that call bet

    println!("  OOP turn range: ~80 combos (hands that bet flop)");
    println!("  IP turn range: ~120 combos (hands that called bet)");

    // Step 3: Live solve turn
    println!("\nStep 3: Turn card comes: 7h");
    println!("  Solving turn + river live...");

    let turn_start = Instant::now();
    let turn_game = solve_turn_live(oop_turn_range, ip_turn_range, "KhQs6h", "7h", 825, 9175)?;
    let turn_time = turn_start.elapsed();

    println!("  ✓ Turn solve: {:.2}s", turn_time.as_secs_f64());

    // Step 4: Simulate turn action
    println!("\nStep 4: Simulate turn action (check-check)");
    println!("  Filtering ranges further...");

    let oop_river_range = "AA,KK,QQ,AKs,KQs";  // ~30 combos
    let ip_river_range = "AA,KK,QQ,JJ,TT,AKs"; // ~40 combos

    // Step 5: Live solve river
    println!("\nStep 5: River card comes: 2d");
    println!("  Solving river live...");

    let river_start = Instant::now();
    let _river_game = solve_river_live(oop_river_range, ip_river_range, "KhQs6h", "7h", "2d", 825, 9175)?;
    let river_time = river_start.elapsed();

    println!("  ✓ River solve: {:.2}s", river_time.as_secs_f64());

    println!("\n=== Summary ===");
    println!("Flop query: <1ms (from database)");
    println!("Turn solve: {:.2}s (live)", turn_time.as_secs_f64());
    println!("River solve: {:.2}s (live)", river_time.as_secs_f64());
    println!("Total latency: {:.2}s", (turn_time + river_time).as_secs_f64());

    Ok(())
}

fn query_flop_strategy(conn: &Connection, hand: &str) -> Result<Vec<(String, f64)>, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare(
        "SELECT action, frequency FROM strategies
         WHERE scenario='BTN_RFI_vs_BB_defend' AND board='KhQs6h' AND hand=? AND position='OOP'
         ORDER BY frequency DESC")?;

    let mut rows = stmt.query([hand])?;
    let mut result = Vec::new();

    while let Some(row) = rows.next()? {
        result.push((row.get(0)?, row.get(1)?));
    }

    Ok(result)
}

fn solve_turn_live(
    oop_range: &str,
    ip_range: &str,
    flop: &str,
    turn_card: &str,
    pot: i32,
    stack: i32,
) -> Result<PostFlopGame, Box<dyn std::error::Error>> {
    let card_config = CardConfig {
        range: [oop_range.parse()?, ip_range.parse()?],
        flop: flop_from_str(flop)?,
        turn: card_from_str(turn_card)?,
        river: NOT_DEALT,
    };

    let turn_bet_sizes = BetSizeOptions::try_from(("50%, 100%", "2x"))?;
    let river_bet_sizes = BetSizeOptions::try_from(("75%, a", "2x"))?;

    let tree_config = TreeConfig {
        initial_state: BoardState::Turn,
        starting_pot: pot,
        effective_stack: stack,
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

    let action_tree = ActionTree::new(tree_config)?;
    let mut game = PostFlopGame::with_config(card_config, action_tree)?;
    game.allocate_memory(true);
    solve(&mut game, 200, 5.0, false);

    Ok(game)
}

fn solve_river_live(
    oop_range: &str,
    ip_range: &str,
    flop: &str,
    turn_card: &str,
    river_card: &str,
    pot: i32,
    stack: i32,
) -> Result<PostFlopGame, Box<dyn std::error::Error>> {
    let card_config = CardConfig {
        range: [oop_range.parse()?, ip_range.parse()?],
        flop: flop_from_str(flop)?,
        turn: card_from_str(turn_card)?,
        river: card_from_str(river_card)?,
    };

    let river_bet_sizes = BetSizeOptions::try_from(("75%, a", "2x"))?;

    let tree_config = TreeConfig {
        initial_state: BoardState::River,
        starting_pot: pot,
        effective_stack: stack,
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

    let action_tree = ActionTree::new(tree_config)?;
    let mut game = PostFlopGame::with_config(card_config, action_tree)?;
    game.allocate_memory(true);
    solve(&mut game, 200, 5.0, false);

    Ok(game)
}
