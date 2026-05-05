/// Extract entire flop tree from v1.1_KhQs6h.bin to database
/// Not just root, but all nodes reachable on flop

use postflop_solver::*;
use rusqlite::{Connection, params};
use std::fs;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Extract v1.1 Full Flop Tree to Database ===\n");

    let solution_path = "solutions/v1.1_KhQs6h.bin";

    println!("Loading: {}", solution_path);
    let load_start = Instant::now();
    let (mut game, _): (PostFlopGame, String) = load_data_from_file(solution_path, None)?;
    println!("  Load time: {:.2}s", load_start.elapsed().as_secs_f64());

    let board = "KhQs6h";
    let scenario = "BTN_RFI_vs_BB_defend";

    println!("\nExtracting full flop tree...");
    let extract_start = Instant::now();

    let conn = Connection::open("v11_flop_tree.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS strategies (
            scenario TEXT, board TEXT, hand TEXT, position TEXT,
            action_history TEXT, action TEXT, frequency REAL, equity REAL,
            PRIMARY KEY (scenario, board, hand, position, action_history, action)
        )", [])?;

    conn.execute("CREATE INDEX IF NOT EXISTS idx_lookup
        ON strategies(scenario, board, hand, action_history)", [])?;

    let mut inserted = 0;

    // Extract root (OOP's initial decision)
    game.back_to_root();
    game.cache_normalized_weights();
    inserted += extract_node(&conn, &game, scenario, board, "", 0)?;

    // Extract after OOP checks (IP's decision)
    game.back_to_root();
    game.play(0); // OOP checks
    game.cache_normalized_weights();
    inserted += extract_node(&conn, &game, scenario, board, "check", 1)?;

    // Extract after OOP bets (each bet size)
    game.back_to_root();
    let root_actions = game.available_actions();

    for (action_idx, action) in root_actions.iter().enumerate() {
        if action_idx == 0 {
            continue; // Skip check, already did it
        }

        game.back_to_root();
        game.play(action_idx);
        game.cache_normalized_weights();

        let action_str = format!("{:?}", action);
        inserted += extract_node(&conn, &game, scenario, board, &action_str, 1)?;

        // Extract after OOP bets -> IP calls
        let ip_actions = game.available_actions();
        for (ip_action_idx, ip_action) in ip_actions.iter().enumerate() {
            game.back_to_root();
            game.play(action_idx);
            game.play(ip_action_idx);

            // Check if we're still on flop (not terminal)
            if !game.is_terminal_node() && game.current_player() == 0 {
                game.cache_normalized_weights();
                let history = format!("{}-{:?}", action_str, ip_action);
                inserted += extract_node(&conn, &game, scenario, board, &history, 0)?;
            }
        }
    }

    let extract_time = extract_start.elapsed();

    println!("✓ Extracted {} records in {:.2}s", inserted, extract_time.as_secs_f64());

    let metadata = fs::metadata("v11_flop_tree.db")?;
    println!("✓ Database: v11_flop_tree.db");
    println!("  Size: {:.2} KB", metadata.len() as f64 / 1024.0);

    // Show sample queries
    println!("\n=== Sample Queries ===");

    println!("\n1. Root (OOP's initial decision) - ThJh:");
    query_node(&conn, scenario, board, "ThJh", "OOP", "")?;

    println!("\n2. After OOP checks (IP's decision) - ThJh:");
    query_node(&conn, scenario, board, "ThJh", "IP", "check")?;

    println!("\n3. After OOP bets 275 (IP's decision) - AcAd:");
    query_node(&conn, scenario, board, "AcAd", "IP", "Bet(275)")?;

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
