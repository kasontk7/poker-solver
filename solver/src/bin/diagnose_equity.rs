use postflop_solver::*;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Equity Diagnostic Tool ===\n");

    let bin_path = "../solutions/v1.1_KhQs6h.bin";
    println!("Loading {}...", bin_path);

    let (mut game, memo): (PostFlopGame, String) = load_data_from_file(bin_path, None)?;

    println!("✓ Loaded: {}", memo);
    println!();

    game.back_to_root();
    game.cache_normalized_weights();

    let oop_cards = game.private_cards(0);
    let ip_cards = game.private_cards(1);

    println!("═══════════════════════════════════════");
    println!("  GAME INFO");
    println!("═══════════════════════════════════════");
    println!("Board: Kh Qs 6h");
    println!("OOP (BB) hands: {}", oop_cards.len());
    println!("IP (BTN) hands: {}", ip_cards.len());
    println!();

    // Test specific hands with known equities from GTO Wizard
    let test_cases = vec![
        ("Jh", "Th", 0, "JhTh", "OOP", 62.9),  // GTO Wizard says 62.9% for BB
        ("Jh", "Th", 1, "JhTh", "IP", 65.8),   // GTO Wizard says 65.8% for BTN
        ("9s", "9d", 0, "99", "OOP", 0.0),     // Unknown from GTO Wizard
        ("8s", "8d", 0, "88", "OOP", 0.0),
    ];

    println!("═══════════════════════════════════════");
    println!("  EQUITY TESTS");
    println!("═══════════════════════════════════════\n");

    for (c1_str, c2_str, player, hand_name, pos, expected_equity) in test_cases {
        let card1 = card_from_str(c1_str)?;
        let card2 = card_from_str(c2_str)?;

        // Check board conflicts
        let board = [
            card_from_str("Kh")?,
            card_from_str("Qs")?,
            card_from_str("6h")?,
        ];

        if board.contains(&card1) || board.contains(&card2) {
            println!("  {} ({}): ⚠️ Blocked by board\n", hand_name, pos);
            continue;
        }

        let cards = if player == 0 { &oop_cards } else { &ip_cards };

        // Find hand
        if let Some(hand_idx) = cards.iter().position(|&(c1, c2)| {
            (c1 == card1 && c2 == card2) || (c1 == card2 && c2 == card1)
        }) {
            let equity_vec = game.equity(player);
            let actual_equity = equity_vec[hand_idx] * 100.0;

            println!("  {} ({})", hand_name, pos);
            println!("    Actual equity: {:.1}%", actual_equity);

            if expected_equity > 0.0 {
                let diff = actual_equity - expected_equity;
                println!("    Expected (GTO Wizard): {:.1}%", expected_equity);
                println!("    Difference: {:.1}%", diff);

                if diff.abs() > 5.0 {
                    println!("    ❌ MAJOR DISCREPANCY!");
                }
            }
            println!();
        } else {
            println!("  {} ({}): Not in range\n", hand_name, pos);
        }
    }

    // Compute average equity across all hands for both players
    println!("═══════════════════════════════════════");
    println!("  AVERAGE EQUITY");
    println!("═══════════════════════════════════════\n");

    let oop_equity = game.equity(0);
    let ip_equity = game.equity(1);

    let avg_oop: f32 = oop_equity.iter().sum::<f32>() / oop_equity.len() as f32;
    let avg_ip: f32 = ip_equity.iter().sum::<f32>() / ip_equity.len() as f32;

    println!("  OOP (BB) average: {:.1}%", avg_oop * 100.0);
    println!("  IP (BTN) average: {:.1}%", avg_ip * 100.0);
    println!("  Sum: {:.1}%", (avg_oop + avg_ip) * 100.0);
    println!();

    if (avg_oop + avg_ip - 1.0).abs() > 0.01 {
        println!("  ❌ WARNING: Equities don't sum to 100%!");
        println!("     This indicates a fundamental calculation error.");
    }

    // Check exploitability
    println!("═══════════════════════════════════════");
    println!("  SOLUTION QUALITY");
    println!("═══════════════════════════════════════\n");

    println!("  Computing exploitability...");
    let exploitability = compute_exploitability(&game);
    println!("  Exploitability: {:.2} cents", exploitability);

    if exploitability > 5.0 {
        println!("  ⚠️ Higher than target (5¢)");
    } else {
        println!("  ✓ Within target");
    }

    println!();
    println!("═══════════════════════════════════════");

    Ok(())
}
