/// Extract just root from v1.1_KhQs6h.bin to database

use postflop_solver::*;
use rusqlite::{Connection, params};
use std::fs;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Extract v1.1 Root to Database ===\n");

    let solution_path = "solutions/v1.1_KhQs6h.bin";

    println!("Loading: {}", solution_path);
    let load_start = Instant::now();
    let (mut game, _): (PostFlopGame, String) = load_data_from_file(solution_path, None)?;
    println!("  Load time: {:.2}s", load_start.elapsed().as_secs_f64());

    game.back_to_root();
    game.cache_normalized_weights();

    let board = "KhQs6h";
    let scenario = "BTN_RFI_vs_BB_defend";

    println!("\nExtracting root strategies...");
    let extract_start = Instant::now();

    let conn = Connection::open("v11_root.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS strategies (
            scenario TEXT, board TEXT, hand TEXT, position TEXT,
            action TEXT, frequency REAL, equity REAL,
            PRIMARY KEY (scenario, board, hand, position, action)
        )", [])?;

    conn.execute("CREATE INDEX IF NOT EXISTS idx_lookup ON strategies(scenario, board, hand)", [])?;

    let oop_cards = game.private_cards(0);
    let ip_cards = game.private_cards(1);
    let actions = game.available_actions();
    let strategy = game.strategy();
    let oop_equity = game.equity(0);
    let ip_equity = game.equity(1);

    let mut inserted = 0;

    // Extract OOP
    for (hand_idx, &(c1, c2)) in oop_cards.iter().enumerate() {
        let hand_str = format!("{}{}", card_to_string(c1), card_to_string(c2));
        for (action_idx, action) in actions.iter().enumerate() {
            let freq = strategy[hand_idx + action_idx * oop_cards.len()];
            if freq > 0.001 {
                conn.execute(
                    "INSERT OR REPLACE INTO strategies VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    params![scenario, board, &hand_str, "OOP",
                            format!("{:?}", action), freq, oop_equity[hand_idx]])?;
                inserted += 1;
            }
        }
    }

    // Store IP equity
    for (hand_idx, &(c1, c2)) in ip_cards.iter().enumerate() {
        let hand_str = format!("{}{}", card_to_string(c1), card_to_string(c2));
        conn.execute(
            "INSERT OR REPLACE INTO strategies VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![scenario, board, &hand_str, "IP", "WAIT", 1.0, ip_equity[hand_idx]])?;
        inserted += 1;
    }

    let extract_time = extract_start.elapsed();

    println!("✓ Extracted {} records in {:.2}s", inserted, extract_time.as_secs_f64());

    let metadata = fs::metadata("v11_root.db")?;
    println!("✓ Database: v11_root.db");
    println!("  Size: {:.2} KB", metadata.len() as f64 / 1024.0);

    // Show sample
    println!("\nSample query - AsAd:");
    let mut stmt = conn.prepare(
        "SELECT action, frequency FROM strategies
         WHERE scenario=? AND board=? AND hand=? AND position='OOP'
         ORDER BY frequency DESC")?;

    let mut rows = stmt.query(params![scenario, board, "AsAd"])?;
    while let Some(row) = rows.next()? {
        let action: String = row.get(0)?;
        let freq: f64 = row.get(1)?;
        println!("  {}: {:.1}%", action, freq * 100.0);
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
