/// Query root strategies from SQLite database
///
/// Usage: query_db <scenario> <board> <hand>
///
/// Example: query_db BTN_RFI_vs_BB_defend AsKsQs AhKd
///
/// Query time: ~1-5ms (vs ~26 seconds loading .bin from HDD)

use rusqlite::{Connection, params};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: query_db <scenario> <board> <hand>");
        eprintln!("Example: query_db BTN_RFI_vs_BB_defend AsKsQs AhKd");
        std::process::exit(1);
    }

    let scenario = &args[1];
    let board = &args[2];
    let hand = &args[3];

    let conn = Connection::open("strategies.db")?;

    println!("=== Strategy Lookup (Root) ===");
    println!("Scenario: {}", scenario);
    println!("Board: {}", board);
    println!("Hand: {}", hand);
    println!();

    // Query strategies
    let mut stmt = conn.prepare(
        "SELECT position, action, frequency, equity
         FROM strategies
         WHERE scenario = ?1 AND board = ?2 AND hand = ?3
         ORDER BY frequency DESC"
    )?;

    let mut rows = stmt.query(params![scenario, board, hand])?;

    let mut found = false;
    let mut current_position = String::new();

    while let Some(row) = rows.next()? {
        found = true;
        let position: String = row.get(0)?;
        let action: String = row.get(1)?;
        let frequency: f64 = row.get(2)?;
        let equity: f64 = row.get(3)?;

        if position != current_position {
            if !current_position.is_empty() {
                println!();
            }
            println!("Position: {}", position);
            println!("Equity: {:.1}%", equity * 100.0);
            println!("\nGTO Strategy:");
            current_position = position;
        }

        if action != "WAIT" {
            println!("  {}: {:.1}%", action, frequency * 100.0);
        }
    }

    if !found {
        println!("⚠️  No strategy found for this hand");
        println!("\nPossible reasons:");
        println!("- Hand not in range for this scenario");
        println!("- Hand conflicts with board");
        println!("- Data not yet extracted (run extract_to_db first)");
    }

    Ok(())
}
