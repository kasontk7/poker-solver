/// Minimal test solver with full tree (flop/turn/river) but tiny ranges
///
/// Usage: tiny_solver_db
///
/// Tests full tree extraction with:
/// - Tiny ranges (just premium hands)
/// - Minimal bet sizes
/// - Low iterations
/// - Full streets (flop → turn → river)

use postflop_solver::*;
use rusqlite::{Connection, params};
use std::time::Instant;
use std::io::{self, Write as IoWrite};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let total_start = Instant::now();

    println!("=== Tiny Test Solver → Database ===");
    println!("Testing full tree extraction with minimal inputs\n");

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

    // Minimal bet sizes - just one option per street
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
        turn_donk_sizes: None, // No donk bets
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
    let max_iterations = 100; // Low for testing
    let target_exploitability = 10.0; // Relaxed

    let solve_start = Instant::now();
    let exploitability = solve(&mut game, max_iterations, target_exploitability, true);
    let solve_duration = solve_start.elapsed();

    println!();
    println!("✓ Solve complete!");
    println!("  Exploitability: {:.2}¢", exploitability);
    println!("  Time: {:.2} seconds", solve_duration.as_secs_f64());
    println!();

    // Extract to database
    println!("Extracting full tree to database...");
    let extract_start = Instant::now();

    let conn = Connection::open("test_strategies.db")?;

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

    extract_full_tree(&mut game, &conn, "TEST", board_str, board_str, "", &mut inserted, &mut nodes_visited)?;

    let extract_duration = extract_start.elapsed();

    println!("\n✓ Extraction complete!");
    println!("  Nodes visited: {}", nodes_visited);
    println!("  Records inserted: {}", inserted);
    println!("  Extraction time: {:.2} seconds", extract_duration.as_secs_f64());
    println!();

    // Show database size
    let metadata = std::fs::metadata("test_strategies.db")?;
    println!("Database: test_strategies.db");
    println!("  Size: {:.2} KB", metadata.len() as f64 / 1024.0);
    println!();

    let total_duration = total_start.elapsed();
    println!("Total time: {:.2} seconds", total_duration.as_secs_f64());

    // Sample query
    println!("\n=== Sample Query ===");
    let mut stmt = conn.prepare(
        "SELECT board, action_path, position, action, frequency
         FROM strategies
         WHERE scenario = 'TEST' AND hand = 'AsAd'
         LIMIT 5"
    )?;

    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let board: String = row.get(0)?;
        let action_path: String = row.get(1)?;
        let position: String = row.get(2)?;
        let action: String = row.get(3)?;
        let frequency: f64 = row.get(4)?;

        println!("AsAd @ {} [{}] {}: {:.1}%",
            board,
            if action_path.is_empty() { "root" } else { &action_path },
            position,
            frequency * 100.0
        );
    }

    Ok(())
}

fn extract_full_tree(
    game: &mut PostFlopGame,
    conn: &Connection,
    scenario: &str,
    flop_board: &str,
    current_board: &str,
    action_path: &str,
    inserted: &mut i32,
    nodes_visited: &mut i32,
) -> Result<(), Box<dyn std::error::Error>> {
    *nodes_visited += 1;

    if *nodes_visited % 50 == 0 {
        print!("\r  Progress: {} nodes, {} records", nodes_visited, inserted);
        io::stdout().flush()?;
    }

    if game.is_terminal_node() {
        return Ok(());
    }

    // Chance node
    if game.is_chance_node() {
        let possible_cards = game.possible_cards();
        let history = game.history().to_vec();

        for card in 0..52u8 {
            if (possible_cards & (1u64 << card)) != 0 {
                let card_str = card_to_string(card);
                let new_board = format!("{}-{}", current_board, card_str);

                game.play(card as usize);

                if !game.is_terminal_node() {
                    game.cache_normalized_weights();
                    extract_full_tree(game, conn, scenario, flop_board, &new_board, action_path, inserted, nodes_visited)?;
                }

                game.apply_history(&history);
                if !history.is_empty() {
                    game.cache_normalized_weights();
                }
            }
        }
        return Ok(());
    }

    // Decision node
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
                    params![scenario, current_board, action_path, &hand_str, position, &action_str, freq, hand_equity],
                )?;
                *inserted += 1;
            }
        }
    }

    // Recurse
    let history = game.history().to_vec();

    for (action_idx, action) in actions.iter().enumerate() {
        let action_str = format!("{:?}", action);
        let new_path = if action_path.is_empty() {
            action_str
        } else {
            format!("{},{}", action_path, action_str)
        };

        game.play(action_idx);

        if !game.is_terminal_node() {
            game.cache_normalized_weights();
            extract_full_tree(game, conn, scenario, flop_board, current_board, &new_path, inserted, nodes_visited)?;
        }

        game.apply_history(&history);
        if !history.is_empty() {
            game.cache_normalized_weights();
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
