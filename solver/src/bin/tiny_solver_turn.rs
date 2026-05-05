/// Minimal test solver with ROOT + TURN extraction (no river)
///
/// Usage: tiny_solver_turn
///
/// Tests root + turn extraction with:
/// - Tiny ranges (just premium hands)
/// - Extracts flop root + all turn nodes
/// - Stops before river

use postflop_solver::*;
use rusqlite::{Connection, params};
use std::time::Instant;
use std::io::{self, Write as IoWrite};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let total_start = Instant::now();

    println!("=== Tiny Test Solver → Database (ROOT + TURN) ===");
    println!("Testing root + turn extraction\n");

    let oop_range = "AA,KK,QQ,AKs"; // 10 combos
    let ip_range = "AA,KK,QQ,JJ,AKs,AKo"; // 22 combos
    let board_str = "AsKs2h";

    println!("Ranges:");
    println!("  OOP: {} (10 combos)", oop_range);
    println!("  IP: {} (22 combos)", ip_range);
    println!("  Board: {}", board_str);
    println!();

    let card_config = CardConfig {
        range: [
            oop_range.parse()?,
            ip_range.parse()?,
        ],
        flop: flop_from_str(board_str)?,
        turn: NOT_DEALT,
        river: NOT_DEALT,
    };

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

    println!("Building game tree...");
    let action_tree = ActionTree::new(tree_config)?;
    let mut game = PostFlopGame::with_config(card_config, action_tree)?;

    let (_, mem_usage_compressed) = game.memory_usage();
    println!("  Memory: {:.2} MB", mem_usage_compressed as f64 / (1024.0 * 1024.0));
    println!();

    println!("Allocating memory...");
    game.allocate_memory(true);

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

    // Extract ROOT + TURN
    println!("Extracting ROOT + TURN strategies...");
    let extract_start = Instant::now();

    let conn = Connection::open("test_turn.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS strategies (
            scenario TEXT NOT NULL,
            board TEXT NOT NULL,
            action_path TEXT NOT NULL,
            hand TEXT NOT NULL,
            position TEXT NOT NULL,
            action TEXT NOT NULL,
            frequency REAL NOT NULL,
            equity REAL NOT NULL,
            PRIMARY KEY (scenario, board, action_path, hand, position, action)
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_lookup ON strategies(scenario, board, action_path, hand)",
        [],
    )?;

    game.back_to_root();
    game.cache_normalized_weights();

    let mut inserted = 0;
    let mut nodes_visited = 0;

    extract_root_and_turn(&mut game, &conn, "TEST", board_str, &mut inserted, &mut nodes_visited)?;

    let extract_duration = extract_start.elapsed();

    println!("\n✓ Extraction complete!");
    println!("  Nodes visited: {}", nodes_visited);
    println!("  Records inserted: {}", inserted);
    println!("  Extraction time: {:.2} seconds", extract_duration.as_secs_f64());
    println!();

    let metadata = std::fs::metadata("test_turn.db")?;
    println!("Database: test_turn.db");
    println!("  Size: {:.2} KB", metadata.len() as f64 / 1024.0);
    println!();

    let total_duration = total_start.elapsed();
    println!("Total time: {:.2} seconds", total_duration.as_secs_f64());

    // Sample queries
    println!("\n=== Sample Queries ===");

    // Count records by board
    println!("\nRecords per street:");
    let mut stmt = conn.prepare(
        "SELECT board, COUNT(*) as cnt FROM strategies GROUP BY board ORDER BY board"
    )?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let board: String = row.get(0)?;
        let count: i32 = row.get(1)?;
        println!("  {}: {} records", board, count);
    }

    // Sample turn query
    println!("\n\nQuerying AsAd on turn 3h after check-check:");
    let turn_board = format!("{}-3h", board_str);
    let mut stmt = conn.prepare(
        "SELECT position, action, frequency
         FROM strategies
         WHERE scenario = 'TEST' AND board = ? AND hand = ? AND action_path = ?
         ORDER BY frequency DESC
         LIMIT 5"
    )?;

    let mut rows = stmt.query(params![turn_board, "AsAd", "Check,Check"])?;
    let mut found = false;
    while let Some(row) = rows.next()? {
        found = true;
        let position: String = row.get(0)?;
        let action: String = row.get(1)?;
        let frequency: f64 = row.get(2)?;
        if action != "WAIT" {
            println!("  {}: {}: {:.1}%", position, action, frequency * 100.0);
        }
    }
    if !found {
        println!("  (No data - path may not exist)");
    }

    Ok(())
}

fn extract_root_and_turn(
    game: &mut PostFlopGame,
    conn: &Connection,
    scenario: &str,
    flop_board: &str,
    inserted: &mut i32,
    nodes_visited: &mut i32,
) -> Result<(), Box<dyn std::error::Error>> {
    // Extract root
    extract_node_strategies(game, conn, scenario, flop_board, "", inserted)?;
    *nodes_visited += 1;

    if game.is_terminal_node() {
        return Ok(());
    }

    // Navigate flop actions and extract turn
    let actions = game.available_actions();
    let history_root = game.history().to_vec();

    for (action_idx, action) in actions.iter().enumerate() {
        let action_str = format!("{:?}", action);

        game.play(action_idx);

        if !game.is_terminal_node() {
            game.cache_normalized_weights();

            // Check if we hit a chance node (turn)
            if game.is_chance_node() {
                extract_turn_nodes(game, conn, scenario, flop_board, &action_str, inserted, nodes_visited)?;
            } else {
                // Still on flop, recurse one more level
                let flop_actions = game.available_actions();
                let history_flop = game.history().to_vec();

                for (flop_action_idx, flop_action) in flop_actions.iter().enumerate() {
                    let flop_path = format!("{},{:?}", action_str, flop_action);

                    game.play(flop_action_idx);

                    if !game.is_terminal_node() {
                        game.cache_normalized_weights();

                        if game.is_chance_node() {
                            extract_turn_nodes(game, conn, scenario, flop_board, &flop_path, inserted, nodes_visited)?;
                        }
                    }

                    game.apply_history(&history_flop);
                    game.cache_normalized_weights();
                }
            }
        }

        game.apply_history(&history_root);
        game.cache_normalized_weights();
    }

    Ok(())
}

fn extract_turn_nodes(
    game: &mut PostFlopGame,
    conn: &Connection,
    scenario: &str,
    flop_board: &str,
    action_path: &str,
    inserted: &mut i32,
    nodes_visited: &mut i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let possible_cards = game.possible_cards();
    let history_before_turn = game.history().to_vec();

    let mut turn_count = 0;

    for card in 0..52u8 {
        if (possible_cards & (1u64 << card)) != 0 {
            turn_count += 1;
            let card_str = card_to_string(card);
            let turn_board = format!("{}-{}", flop_board, card_str);

            game.play(card as usize);

            if !game.is_terminal_node() {
                game.cache_normalized_weights();
                extract_node_strategies(game, conn, scenario, &turn_board, action_path, inserted)?;
                *nodes_visited += 1;

                if *nodes_visited % 50 == 0 {
                    print!("\r  Progress: {} nodes, {} records", nodes_visited, inserted);
                    io::stdout().flush()?;
                }
            }

            game.apply_history(&history_before_turn);
            game.cache_normalized_weights();
        }
    }

    println!("\n  Extracted {} turn cards for action path: {}", turn_count, action_path);

    Ok(())
}

fn extract_node_strategies(
    game: &mut PostFlopGame,
    conn: &Connection,
    scenario: &str,
    board: &str,
    action_path: &str,
    inserted: &mut i32,
) -> Result<(), Box<dyn std::error::Error>> {
    if game.is_terminal_node() || game.is_chance_node() {
        return Ok(());
    }

    let current_player = game.current_player();
    let private_cards = game.private_cards(current_player);
    let actions = game.available_actions();
    let strategy = game.strategy();
    let equity = game.equity(current_player);

    let position = if current_player == 0 { "OOP" } else { "IP" };

    for (hand_idx, &(c1, c2)) in private_cards.iter().enumerate() {
        let hand_str = format!("{}{}", card_to_string(c1), card_to_string(c2));
        let hand_equity = equity[hand_idx];

        for (action_idx, action) in actions.iter().enumerate() {
            let freq = strategy[hand_idx + action_idx * private_cards.len()];

            if freq > 0.001 {
                let action_str = format!("{:?}", action);

                conn.execute(
                    "INSERT OR REPLACE INTO strategies
                     (scenario, board, action_path, hand, position, action, frequency, equity)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![scenario, board, action_path, &hand_str, position, &action_str, freq, hand_equity],
                )?;
                *inserted += 1;
            }
        }
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
