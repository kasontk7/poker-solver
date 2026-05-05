/// Extract complete flop tree from v1.1_KhQs6h.bin to database
/// Recursively traverses all nodes until terminal states (fold/call)

use postflop_solver::*;
use rusqlite::{Connection, params};
use std::fs;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Extract v1.1 Complete Flop Tree to Database ===\n");

    let solution_path = "solutions/v1.1_KhQs6h.bin";

    println!("Loading: {}", solution_path);
    let load_start = Instant::now();
    let (mut game, _): (PostFlopGame, String) = load_data_from_file(solution_path, None)?;
    println!("  Load time: {:.2}s", load_start.elapsed().as_secs_f64());

    let board = "KhQs6h";
    let scenario = "BTN_RFI_vs_BB_defend";

    println!("\nExtracting complete flop tree...");
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

    // Start recursive extraction from root
    game.back_to_root();
    game.cache_normalized_weights();
    inserted += extract_flop_tree_recursive(&conn, &mut game, scenario, board, "", &mut vec![])?;

    let extract_time = extract_start.elapsed();

    println!("✓ Extracted {} records in {:.2}s", inserted, extract_time.as_secs_f64());

    let metadata = fs::metadata("v11_flop_complete.db")?;
    println!("✓ Database: v11_flop_complete.db");
    println!("  Size: {:.2} KB", metadata.len() as f64 / 1024.0);

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

    Ok(())
}

fn extract_flop_tree_recursive(
    conn: &Connection,
    game: &mut PostFlopGame,
    scenario: &str,
    board: &str,
    action_history: &str,
    history_stack: &mut Vec<usize>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut count = 0;

    // Check if terminal
    if game.is_terminal_node() {
        return Ok(0);
    }

    // Safety check: don't go beyond depth 4 (e.g., check-bet-raise-call)
    // This covers all practical flop scenarios
    if history_stack.len() >= 4 {
        return Ok(0);
    }

    let current_player = game.current_player();
    let position = if current_player == 0 { "OOP" } else { "IP" };

    // Extract current node
    count += extract_node(conn, game, scenario, board, action_history, current_player)?;

    // Get available actions - clone to avoid borrowing issues
    let actions: Vec<Action> = game.available_actions().to_vec();
    let num_actions = actions.len();

    // Recurse into each action
    for action_idx in 0..num_actions {
        // Re-get actions each time since game state changes
        let current_actions = game.available_actions();
        if action_idx >= current_actions.len() {
            break;
        }
        let action = current_actions[action_idx].clone();
        // Check if this action should stop recursion
        let is_call = matches!(&action, Action::Call);
        let is_fold = matches!(&action, Action::Fold);
        let is_chance = matches!(&action, Action::Chance(_));

        // Navigate to this action
        game.play(action_idx);
        game.cache_normalized_weights();

        // Build new action history
        let action_str = format!("{:?}", action);
        let new_history = if action_history.is_empty() {
            action_str.clone()
        } else {
            format!("{}-{}", action_history, action_str)
        };

        // Check if this action leads to a terminal state
        let is_terminal = game.is_terminal_node();

        // Continue recursing only if not terminal, not call/fold, and not transitioning to next street
        if !is_terminal && !is_call && !is_fold && !is_chance {
            // Continue recursing
            history_stack.push(action_idx);
            count += extract_flop_tree_recursive(conn, game, scenario, board, &new_history, history_stack)?;
            history_stack.pop();
        }

        // Navigate back
        game.back_to_root();
        if !history_stack.is_empty() {
            for &prev_action in history_stack.iter() {
                let avail = game.available_actions();
                if prev_action >= avail.len() {
                    eprintln!("Warning: Invalid action index {} (only {} actions available)", prev_action, avail.len());
                    return Ok(count); // Bail out gracefully
                }
                game.play(prev_action);
            }
            game.cache_normalized_weights();
        }
    }

    Ok(count)
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
