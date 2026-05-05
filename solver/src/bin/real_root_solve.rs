/// Real root-only solve with BTN RFI vs BB defend ranges
/// Extracts to database for testing

use postflop_solver::*;
use rusqlite::{Connection, params};
use std::fs;
use std::time::Instant;

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
    println!("=== Real Root Solve: BTN RFI vs BB Defend ===\n");

    let board_str = "AsKs2h";

    println!("Loading real ranges...");
    let ip_range = load_range_from_file("ranges/gto/rfi/BTN_RFI_vs_BB_defend/btn.txt")?;
    let oop_range = load_range_from_file("ranges/gto/rfi/BTN_RFI_vs_BB_defend/bb.txt")?;

    println!("Board: {}", board_str);
    println!();

    let card_config = CardConfig {
        range: [oop_range.parse()?, ip_range.parse()?],
        flop: flop_from_str(board_str)?,
        turn: NOT_DEALT,
        river: NOT_DEALT,
    };

    let flop_bet_sizes = BetSizeOptions::try_from(("50%, 100%", "3x, 5x"))?;
    let turn_bet_sizes = BetSizeOptions::try_from(("50%, 100%, 150%", "3x, 5x"))?;
    let river_bet_sizes = BetSizeOptions::try_from(("75%, 150%, a", "3x, 5x"))?;
    let donk_sizes = DonkSizeOptions::try_from("50%")?;

    let tree_config = TreeConfig {
        initial_state: BoardState::Flop,
        starting_pot: 550,
        effective_stack: 9750,
        rake_rate: 0.0,
        rake_cap: 0.0,
        flop_bet_sizes: [flop_bet_sizes.clone(), flop_bet_sizes],
        turn_bet_sizes: [turn_bet_sizes.clone(), turn_bet_sizes],
        river_bet_sizes: [river_bet_sizes.clone(), river_bet_sizes],
        turn_donk_sizes: Some(donk_sizes.clone()),
        river_donk_sizes: Some(donk_sizes),
        add_allin_threshold: 1.5,
        force_allin_threshold: 0.15,
        merging_threshold: 0.1,
    };

    println!("Building tree...");
    let action_tree = ActionTree::new(tree_config)?;
    let mut game = PostFlopGame::with_config(card_config, action_tree)?;

    let (_, mem) = game.memory_usage();
    println!("  Memory: {:.2} GB", mem as f64 / (1024.0 * 1024.0 * 1024.0));

    game.allocate_memory(true);

    println!("\nSolving...");
    let solve_start = Instant::now();
    let exp = solve(&mut game, 500, 5.0, true);
    let solve_time = solve_start.elapsed();

    println!("\n✓ Solve: {:.2} min, Exp: {:.2}¢", solve_time.as_secs_f64() / 60.0, exp);

    println!("\nExtracting root to DB...");
    let conn = Connection::open("real_root.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS strategies (
            scenario TEXT, board TEXT, hand TEXT, position TEXT,
            action TEXT, frequency REAL, equity REAL,
            PRIMARY KEY (scenario, board, hand, position, action)
        )", [])?;

    conn.execute("CREATE INDEX IF NOT EXISTS idx_lookup ON strategies(scenario, board, hand)", [])?;

    game.back_to_root();
    game.cache_normalized_weights();

    let oop_cards = game.private_cards(0);
    let actions = game.available_actions();
    let strategy = game.strategy();
    let oop_equity = game.equity(0);

    let mut inserted = 0;
    for (hand_idx, &(c1, c2)) in oop_cards.iter().enumerate() {
        let hand_str = format!("{}{}", card_to_string(c1), card_to_string(c2));
        for (action_idx, action) in actions.iter().enumerate() {
            let freq = strategy[hand_idx + action_idx * oop_cards.len()];
            if freq > 0.001 {
                conn.execute(
                    "INSERT OR REPLACE INTO strategies VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    params!["BTN_RFI_vs_BB_defend", board_str, &hand_str, "OOP",
                            format!("{:?}", action), freq, oop_equity[hand_idx]])?;
                inserted += 1;
            }
        }
    }

    let metadata = fs::metadata("real_root.db")?;
    println!("✓ DB: {} records, {:.2} MB", inserted, metadata.len() as f64 / (1024.0 * 1024.0));

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
