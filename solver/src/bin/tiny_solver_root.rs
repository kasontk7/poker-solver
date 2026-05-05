/// Minimal test solver with ROOT ONLY extraction (flop first action)
///
/// Usage: tiny_solver_root
///
/// Tests root-only extraction with:
/// - Tiny ranges (just premium hands)
/// - Minimal bet sizes
/// - Low iterations
/// - Root strategies only (no turn/river navigation)

use postflop_solver::*;
use rusqlite::{Connection, params};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let total_start = Instant::now();

    println!("=== Tiny Test Solver → Database (ROOT ONLY) ===");
    println!("Testing root extraction with minimal inputs\n");

    // Tiny ranges - just premium hands
    let oop_range = "AA,KK,QQ,AKs"; // 10 combos
    let ip_range = "AA,KK,QQ,JJ,AKs,AKo"; // 22 combos

    let board_str = "AsKs2h";

    println!("Ranges:");
    println!("  OOP: {} (10 combos)", oop_range);
    println!("  IP: {} (22 combos)", ip_range);
    println!("  Board: {}", board_str);
    println!();

    // Card config
    let card_config = CardConfig {
        range: [
            oop_range.parse()?,
            ip_range.parse()?,
        ],
        flop: flop_from_str(board_str)?,
        turn: NOT_DEALT,
        river: NOT_DEALT,
    };

    // Minimal bet sizes
    let flop_bet_sizes = BetSizeOptions::try_from(("50%", "2x"))?;
    let turn_bet_sizes = BetSizeOptions::try_from(("50%", "2x"))?;
    let river_bet_sizes = BetSizeOptions::try_from(("75%", "2x"))?;

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

    // Build game tree
    println!("Building game tree...");
    let action_tree = ActionTree::new(tree_config)?;
    let mut game = PostFlopGame::with_config(card_config, action_tree)?;

    let (_, mem_usage_compressed) = game.memory_usage();
    println!("  Memory: {:.2} MB", mem_usage_compressed as f64 / (1024.0 * 1024.0));
    println!();

    println!("Allocating memory...");
    game.allocate_memory(true);

    // Solve with low iterations
    println!("Solving game...");
    let max_iterations = 100;
    let target_exploitability = 10.0;

    let solve_start = Instant::now();
    let exploitability = solve(&mut game, max_iterations, target_exploitability, true);
    let solve_duration = solve_start.elapsed();

    println!();
    println!("✓ Solve complete!");
    println!("  Exploitability: {:.2}¢", exploitability);
    println!("  Time: {:.2} seconds", solve_duration.as_secs_f64());
    println!();

    // Extract ROOT ONLY to database
    println!("Extracting ROOT strategies to database...");
    let extract_start = Instant::now();

    let conn = Connection::open("test_root.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS strategies (
            scenario TEXT NOT NULL,
            board TEXT NOT NULL,
            hand TEXT NOT NULL,
            position TEXT NOT NULL,
            action TEXT NOT NULL,
            frequency REAL NOT NULL,
            equity REAL NOT NULL,
            PRIMARY KEY (scenario, board, hand, position, action)
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_lookup ON strategies(scenario, board, hand)",
        [],
    )?;

    game.back_to_root();
    game.cache_normalized_weights();

    let mut inserted = 0;
    extract_root_only(&mut game, &conn, "TEST", board_str, &mut inserted)?;

    let extract_duration = extract_start.elapsed();

    println!("✓ Extraction complete!");
    println!("  Records inserted: {}", inserted);
    println!("  Extraction time: {:.2} seconds", extract_duration.as_secs_f64());
    println!();

    // Show database size
    let metadata = std::fs::metadata("test_root.db")?;
    println!("Database: test_root.db");
    println!("  Size: {:.2} KB", metadata.len() as f64 / 1024.0);
    println!();

    let total_duration = total_start.elapsed();
    println!("Total time: {:.2} seconds", total_duration.as_secs_f64());

    // Sample queries
    println!("\n=== Sample Queries ===");

    // Query AA
    println!("\nQuerying AsAd at root:");
    let mut stmt = conn.prepare(
        "SELECT position, action, frequency, equity
         FROM strategies
         WHERE scenario = 'TEST' AND board = ? AND hand = ?
         ORDER BY frequency DESC"
    )?;

    let mut rows = stmt.query(params![board_str, "AsAd"])?;
    while let Some(row) = rows.next()? {
        let position: String = row.get(0)?;
        let action: String = row.get(1)?;
        let frequency: f64 = row.get(2)?;
        let equity: f64 = row.get(3)?;

        if action != "WAIT" {
            println!("  {}: {}: {:.1}%", position, action, frequency * 100.0);
        } else {
            println!("  {} equity: {:.1}%", position, equity * 100.0);
        }
    }

    // Show all unique hands in database
    println!("\n\nAll hands in database:");
    let mut stmt = conn.prepare(
        "SELECT DISTINCT hand FROM strategies WHERE position = 'OOP' ORDER BY hand"
    )?;
    let mut rows = stmt.query([])?;
    let mut hands = Vec::new();
    while let Some(row) = rows.next()? {
        hands.push(row.get::<_, String>(0)?);
    }
    println!("  OOP: {}", hands.join(", "));

    Ok(())
}

fn extract_root_only(
    game: &mut PostFlopGame,
    conn: &Connection,
    scenario: &str,
    board: &str,
    inserted: &mut i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let oop_cards = game.private_cards(0);
    let ip_cards = game.private_cards(1);
    let actions = game.available_actions();
    let strategy = game.strategy();
    let oop_equity = game.equity(0);
    let ip_equity = game.equity(1);

    println!("  OOP acts first with {} actions", actions.len());
    println!("  OOP has {} hands, IP has {} hands", oop_cards.len(), ip_cards.len());

    // Extract OOP strategies (OOP acts first at root)
    for (hand_idx, &(c1, c2)) in oop_cards.iter().enumerate() {
        let hand_str = format!("{}{}", card_to_string(c1), card_to_string(c2));
        let equity = oop_equity[hand_idx];

        for (action_idx, action) in actions.iter().enumerate() {
            let freq = strategy[hand_idx + action_idx * oop_cards.len()];

            if freq > 0.001 {
                let action_str = format!("{:?}", action);

                conn.execute(
                    "INSERT OR REPLACE INTO strategies
                     (scenario, board, hand, position, action, frequency, equity)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![scenario, board, &hand_str, "OOP", &action_str, freq, equity],
                )?;
                *inserted += 1;
            }
        }
    }

    // Store IP equity (IP doesn't act at root but we want equity)
    for (hand_idx, &(c1, c2)) in ip_cards.iter().enumerate() {
        let hand_str = format!("{}{}", card_to_string(c1), card_to_string(c2));
        let equity = ip_equity[hand_idx];

        conn.execute(
            "INSERT OR REPLACE INTO strategies
             (scenario, board, hand, position, action, frequency, equity)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![scenario, board, &hand_str, "IP", "WAIT", 1.0, equity],
        )?;
        *inserted += 1;
    }

    Ok(())
}

fn card_to_string(card: u8) -> String {
    let rank = match card / 4 {
        12 => 'A', 11 => 'K', 10 => 'Q', 9 => 'J', 8 => 'T',
        r => (b'2' + r) as char,
    };
    let suit = match card % 4 {
        0 => 'c', 1 => 'd', 2 => 'h', 3 => 's',
        _ => '?',
    };
    format!("{}{}", rank, suit)
}
