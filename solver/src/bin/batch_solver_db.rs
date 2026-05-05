/// Solve and write directly to database (skip .bin file)
///
/// Usage: batch_solver_db <scenario> <board>
///
/// Solves the game and extracts strategies directly to SQLite database
/// Skips the intermediate .bin file entirely

use postflop_solver::*;
use rusqlite::{Connection, params};
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

    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: batch_solver_db <scenario> <board>");
        eprintln!("Example: batch_solver_db BTN_RFI_vs_BB_defend AsKsQs");
        std::process::exit(1);
    }

    let scenario = &args[1];
    let board_str = &args[2];

    println!("=== Poker Solver → Database ===");
    println!("Scenario: {}", scenario);
    println!("Board: {}", board_str);
    println!();

    // Parse scenario to get range paths
    let (oop_path, ip_path, pot_size, stack_size) = parse_scenario(scenario)?;

    // Load ranges
    println!("Loading ranges...");
    let oop_range = load_range_from_file(&oop_path)?;
    let ip_range = load_range_from_file(&ip_path)?;

    println!("  OOP: {}", oop_path);
    println!("  IP: {}", ip_path);
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

    // Bet sizes
    let flop_bet_sizes = BetSizeOptions::try_from(("50%, 100%", "3x, 5x"))?;
    let turn_bet_sizes = BetSizeOptions::try_from(("50%, 100%, 150%", "3x, 5x"))?;
    let river_bet_sizes = BetSizeOptions::try_from(("75%, 150%, a", "3x, 5x"))?;
    let donk_sizes = DonkSizeOptions::try_from("50%")?;

    let tree_config = TreeConfig {
        initial_state: BoardState::Flop,
        starting_pot: pot_size,
        effective_stack: stack_size,
        flop_bet_sizes: [flop_bet_sizes.clone(), flop_bet_sizes.clone()],
        turn_bet_sizes: [turn_bet_sizes.clone(), turn_bet_sizes.clone()],
        river_bet_sizes: [river_bet_sizes.clone(), river_bet_sizes],
        turn_donk_sizes: Some(donk_sizes.clone()),
        river_donk_sizes: Some(donk_sizes),
        add_all_in_threshold: 1.5,
        force_all_in_threshold: 0.15,
        merging_threshold: 0.1,
    };

    // Build game tree
    println!("Building game tree...");
    let action_tree = ActionTree::new(tree_config)?;
    let mut game = PostFlopGame::with_config(card_config, action_tree)?;

    let (_, mem_usage_compressed) = game.memory_usage();
    println!("  Memory: {:.2} GB", mem_usage_compressed as f64 / (1024.0 * 1024.0 * 1024.0));
    println!();

    println!("Allocating memory...");
    game.allocate_memory(true);

    // Solve
    println!("Solving game...");
    let max_iterations = 500;
    let target_exploitability = 5.0;

    let solve_start = Instant::now();
    let exploitability = solve(&mut game, max_iterations, target_exploitability, true);
    let solve_duration = solve_start.elapsed();

    println!();
    println!("✓ Solve complete!");
    println!("  Exploitability: {:.2}¢", exploitability);
    println!("  Time: {:.2} min", solve_duration.as_secs_f64() / 60.0);
    println!();

    // Extract to database
    println!("Extracting to database...");
    let extract_start = Instant::now();

    let db_path = env::var("DB_PATH").unwrap_or_else(|_| "strategies.db".to_string());
    let conn = Connection::open(&db_path)?;

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
    extract_full_tree(&mut game, &conn, scenario, board_str, board_str, "", &mut inserted, &mut nodes_visited)?;

    println!("  Nodes visited: {}", nodes_visited);

    let extract_duration = extract_start.elapsed();

    println!("✓ Inserted {} records", inserted);
    println!("  Extraction time: {:.2} min", extract_duration.as_secs_f64() / 60.0);
    println!();

    let total_duration = total_start.elapsed();
    println!("Total time: {:.2} min", total_duration.as_secs_f64() / 60.0);

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

    if *nodes_visited % 100 == 0 {
        print!("\r  Progress: {} nodes, {} records", nodes_visited, inserted);
        use std::io::{self, Write};
        io::stdout().flush()?;
    }

    // Terminal node - nothing to extract
    if game.is_terminal_node() {
        return Ok(());
    }

    // Chance node - iterate all possible cards
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

                // Navigate back
                game.apply_history(&history);
                if !history.is_empty() {
                    game.cache_normalized_weights();
                }
            }
        }
        return Ok(());
    }

    // Decision node - extract strategies
    let current_player = game.current_player();
    let private_cards = game.private_cards(current_player);
    let actions = game.available_actions();
    let strategy = game.strategy();
    let equity = game.equity(current_player);

    let position = if current_player == 0 { "OOP" } else { "IP" };

    // Extract strategies for all hands
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

    // Recurse into child nodes
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

        // Navigate back
        game.apply_history(&history);
        if !history.is_empty() {
            game.cache_normalized_weights();
        }
    }

    Ok(())
}

fn parse_scenario(scenario: &str) -> Result<(String, String, i32, i32), Box<dyn std::error::Error>> {
    let base = "ranges/gto";

    // RFI scenarios
    if scenario.contains("_RFI_vs_") && scenario.contains("_defend") {
        let parts: Vec<&str> = scenario.split("_RFI_vs_").collect();
        let opener = parts[0];
        let defender = parts[1].replace("_defend", "");
        return Ok((
            format!("{}/rfi/{}/{}.txt", base, scenario, defender.to_lowercase()),
            format!("{}/rfi/{}/{}.txt", base, scenario, opener.to_lowercase()),
            550,
            9750,
        ));
    }

    if scenario.contains("_RFI_vs_") && scenario.contains("_cold_call") {
        let parts: Vec<&str> = scenario.split("_RFI_vs_").collect();
        let opener = parts[0];
        let caller = parts[1].replace("_cold_call", "");
        return Ok((
            format!("{}/rfi/{}/{}.txt", base, scenario, caller.to_lowercase()),
            format!("{}/rfi/{}/{}.txt", base, scenario, opener.to_lowercase()),
            550,
            9750,
        ));
    }

    // 3bet scenarios
    if scenario.contains("_3bet_vs_") && scenario.contains("_call") {
        let parts: Vec<&str> = scenario.split("_3bet_vs_").collect();
        let threebettor = parts[0];
        let caller = parts[1].replace("_call", "");
        return Ok((
            format!("{}/3bet/{}/{}.txt", base, scenario, threebettor.to_lowercase()),
            format!("{}/3bet/{}/{}.txt", base, scenario, caller.to_lowercase()),
            2100,
            7900,
        ));
    }

    // 4bet scenarios
    if scenario.contains("_4bet_vs_") && scenario.contains("_call") {
        let parts: Vec<&str> = scenario.split("_4bet_vs_").collect();
        let fourbettor = parts[0];
        let caller = parts[1].replace("_call", "");
        return Ok((
            format!("{}/4bet/{}/{}.txt", base, scenario, fourbettor.to_lowercase()),
            format!("{}/4bet/{}/{}.txt", base, scenario, caller.to_lowercase()),
            5100,
            4900,
        ));
    }

    Err(format!("Unknown scenario format: {}", scenario).into())
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
