/// Extract complete flop tree using BFS (breadth-first search)
/// This avoids the complex recursive navigation issues

use postflop_solver::*;
use rusqlite::{Connection, params};
use std::collections::VecDeque;
use std::fs;
use std::time::Instant;

#[derive(Clone, Debug)]
struct NodePath {
    actions: Vec<Action>,  // Sequence of actions from root
    history_str: String,   // Action history string for database
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Extract v1.1 Complete Flop Tree (BFS) ===\n");

    let solution_path = "solutions/v1.1_KhQs6h.bin";

    println!("Loading: {}", solution_path);
    let load_start = Instant::now();
    let (mut game, _): (PostFlopGame, String) = load_data_from_file(solution_path, None)?;
    println!("  Load time: {:.2}s", load_start.elapsed().as_secs_f64());

    let board = "KhQs6h";
    let scenario = "BTN_RFI_vs_BB_defend";

    println!("\nExtracting complete flop tree using BFS...");
    let extract_start = Instant::now();

    let conn = Connection::open("v11_flop_complete.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS strategies (
            scenario TEXT, board TEXT, hand TEXT, position TEXT,
            action_history TEXT, action TEXT, frequency REAL, equity REAL,
            PRIMARY KEY (scenario, board, hand, position, action_history, action)
        )", [])?;

    conn.execute("CREATE INDEX IF NOT EXISTS idx_lookup
        ON strategies(scenario, board, hand, action_history)", [])?;

    let mut inserted = 0;
    let mut queue = VecDeque::new();

    // Start with root
    queue.push_back(NodePath {
        actions: vec![],
        history_str: String::new(),
    });

    while let Some(path) = queue.pop_front() {
        // Navigate to this node
        game.back_to_root();
        game.cache_normalized_weights();

        let mut nav_failed = false;
        for (i, action) in path.actions.iter().enumerate() {
            let available_actions = game.available_actions();

            // Find the index of this action in the available actions
            let action_idx = available_actions.iter().position(|a| a == action);

            match action_idx {
                Some(idx) => {
                    game.play(idx);
                }
                None => {
                    eprintln!("Warning: Action {:?} not available at step {}. Available: {:?}", action, i, available_actions);
                    nav_failed = true;
                    break;
                }
            }
        }

        if nav_failed {
            continue;
        }

        game.cache_normalized_weights();

        // Check if we should stop here
        if game.is_terminal_node() {
            continue;
        }

        // Check if next actions would be Chance (transitioning to next street)
        let current_actions = game.available_actions();
        if current_actions.is_empty() || current_actions.iter().all(|a| matches!(a, Action::Chance(_))) {
            // All actions are chance or no actions - we've reached end of this street
            continue;
        }

        // Don't go too deep
        if path.actions.len() >= 6 {
            continue;
        }

        let current_player = game.current_player();

        // Extract this node
        inserted += extract_node(&conn, &game, scenario, board, &path.history_str, current_player)?;

        // Get available actions and queue up child nodes WITHOUT navigating yet
        // We need to snapshot the actions first, then queue them
        let available_actions: Vec<Action> = game.available_actions().to_vec();

        for action in available_actions.iter() {
            // Check if we should stop recursing at this action
            let is_call = matches!(action, Action::Call);
            let is_fold = matches!(action, Action::Fold);
            let is_chance = matches!(action, Action::Chance(_));

            if is_fold || is_chance {
                // Don't recurse into fold or chance (next street)
                continue;
            }

            // Build new path - store the actual Action, not the index
            let mut new_actions = path.actions.clone();
            new_actions.push(action.clone());

            let action_str = format!("{:?}", action);
            let new_history = if path.history_str.is_empty() {
                action_str
            } else {
                format!("{}-{}", path.history_str, action_str)
            };

            // If it's a call, we still want to extract that node but not recurse further
            // But we need to do it in a separate pass to avoid modifying game state during iteration
            // For now, just skip calls
            if is_call {
                // Don't recurse into calls
                continue;
            } else {
                // Queue for later processing
                queue.push_back(NodePath {
                    actions: new_actions,
                    history_str: new_history,
                });
            }
        }
    }

    let extract_time = extract_start.elapsed();

    println!("✓ Extracted {} records in {:.2}s", inserted, extract_time.as_secs_f64());

    let metadata = fs::metadata("v11_flop_complete.db")?;
    println!("✓ Database: v11_flop_complete.db");
    println!("  Size: {:.2} MB", metadata.len() as f64 / (1024.0 * 1024.0));

    // Show sample queries
    println!("\n=== Sample Queries ===");

    println!("\n1. Root (OOP's initial decision) - ThJh:");
    query_node(&conn, scenario, board, "ThJh", "OOP", "")?;

    println!("\n2. After OOP checks (IP's decision) - ThJh:");
    query_node(&conn, scenario, board, "ThJh", "IP", "Check")?;

    println!("\n3. After OOP checks, IP bets 275 (OOP response) - ThJh:");
    query_node(&conn, scenario, board, "ThJh", "OOP", "Check-Bet(275)")?;

    println!("\n4. After OOP bets 275 (IP's decision) - AcAd:");
    query_node(&conn, scenario, board, "AcAd", "IP", "Bet(275)")?;

    println!("\n5. After OOP checks, IP bets 275, OOP calls - check if exists:");
    query_node(&conn, scenario, board, "ThJh", "OOP", "Check-Bet(275)-Call")?;

    Ok(())
}

fn extract_node(
    conn: &Connection,
    game: &PostFlopGame,
    scenario: &str,
    board: &str,
    action_history: &str,
    player: usize,
) -> Result<usize, Box<dyn std::error::Error>> {
    let cards = game.private_cards(player);
    let actions = game.available_actions();
    let strategy = game.strategy();
    let equity = game.equity(player);

    let position = if player == 0 { "OOP" } else { "IP" };
    let mut count = 0;

    for (hand_idx, &(c1, c2)) in cards.iter().enumerate() {
        let hand_str = format!("{}{}", card_to_string(c1), card_to_string(c2));
        for (action_idx, action) in actions.iter().enumerate() {
            let freq = strategy[hand_idx + action_idx * cards.len()];
            if freq > 0.001 {
                conn.execute(
                    "INSERT OR REPLACE INTO strategies VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                    params![scenario, board, &hand_str, position,
                            action_history, format!("{:?}", action), freq, equity[hand_idx]])?;
                count += 1;
            }
        }
    }

    Ok(count)
}

fn query_node(
    conn: &Connection,
    scenario: &str,
    board: &str,
    hand: &str,
    position: &str,
    action_history: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare(
        "SELECT action, frequency, equity FROM strategies
         WHERE scenario=? AND board=? AND hand=? AND position=? AND action_history=?
         ORDER BY frequency DESC")?;

    let mut rows = stmt.query(params![scenario, board, hand, position, action_history])?;
    let mut found = false;
    let mut equity_val = 0.0;

    while let Some(row) = rows.next()? {
        if !found {
            found = true;
        }
        let action: String = row.get(0)?;
        let freq: f64 = row.get(1)?;
        equity_val = row.get(2)?;
        println!("  {}: {:.1}%", action, freq * 100.0);
    }

    if found {
        println!("  Equity: {:.1}%", equity_val * 100.0);
    } else {
        println!("  (Not found)");
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
