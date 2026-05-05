/// Extract ALL strategies using library's extraction API
///
/// Usage: extract_to_db <solution.bin>

use postflop_solver::*;
use rusqlite::{Connection, params};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: extract_to_db <solution.bin>");
        eprintln!("Example: extract_to_db solutions/v1.1_KhQs6h.bin");
        std::process::exit(1);
    }

    let solution_path = &args[1];

    let filename = solution_path.split('/').last().unwrap();

    let (scenario, _flop_board) = if filename.starts_with("v1.1_") {
        let board_part = filename.trim_start_matches("v1.1_").trim_end_matches(".bin");
        ("BTN_RFI_vs_BB_defend", board_part)
    } else {
        let parts: Vec<&str> = filename.trim_end_matches(".bin").split("___").collect();
        if parts.len() != 2 {
            return Err("Invalid filename format. Expected: scenario___board.bin".into());
        }
        (parts[0], parts[1])
    };

    println!("Loading solution: {}", solution_path);
    let (mut game, _memo): (PostFlopGame, String) = load_data_from_file(solution_path, None)?;

    println!("Scenario: {}", scenario);

    println!("\nExtracting strategies...");
    let entries = game.extract_all_strategies()?;

    println!("✓ Extracted {} strategy entries", entries.len());

    println!("\nConnecting to database...");
    let conn = Connection::open("strategies.db")?;

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

    println!("Inserting into database...");
    let mut inserted = 0;

    for entry in entries {
        let hand_str = format!("{}{}", card_to_string(entry.hand.0), card_to_string(entry.hand.1));
        let position = if entry.player == 0 { "OOP" } else { "IP" };
        let action_str = format!("{:?}", entry.action);

        conn.execute(
            "INSERT OR REPLACE INTO strategies
             (scenario, board, action_path, hand, position, action, frequency, equity)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![scenario, entry.board, entry.action_path, &hand_str, position, &action_str, entry.frequency, entry.equity],
        )?;
        inserted += 1;

        if inserted % 10000 == 0 {
            print!("\rInserted: {}", inserted);
            use std::io::{self, Write};
            io::stdout().flush()?;
        }
    }

    println!("\n✓ Inserted {} records", inserted);
    println!("✓ Database: strategies.db");

    let metadata = std::fs::metadata("strategies.db")?;
    println!("  Size: {:.2} MB", metadata.len() as f64 / (1024.0 * 1024.0));

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
