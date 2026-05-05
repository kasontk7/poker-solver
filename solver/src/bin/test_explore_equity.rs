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
    println!("=== Testing Explore Equity Display ===\n");

    let bin_path = "../solutions/v1.1_KhQs6h.bin";
    let (mut game, _): (PostFlopGame, String) = load_data_from_file(bin_path, None)?;

    game.back_to_root();
    game.cache_normalized_weights();

    let oop_hands = game.private_cards(0).to_vec();
    let ip_hands = game.private_cards(1).to_vec();

    // Test JhTh for both OOP and IP
    let jh = card_from_str("Jh")?;
    let th = card_from_str("Th")?;

    println!("Testing JhTh equity display:\n");

    // OOP test
    if let Some(oop_idx) = oop_hands.iter().position(|&(c1, c2)| {
        (c1 == jh && c2 == th) || (c1 == th && c2 == jh)
    }) {
        let oop_equity = game.equity(0);
        println!("OOP (player 0):");
        println!("  Hand index: {}", oop_idx);
        println!("  Total OOP hands: {}", oop_hands.len());
        println!("  Equity vector length: {}", oop_equity.len());
        println!("  Equity at index: {:.1}%", oop_equity[oop_idx] * 100.0);
        println!();
    }

    // IP test
    if let Some(ip_idx) = ip_hands.iter().position(|&(c1, c2)| {
        (c1 == jh && c2 == th) || (c1 == th && c2 == jh)
    }) {
        let ip_equity = game.equity(1);
        println!("IP (player 1):");
        println!("  Hand index: {}", ip_idx);
        println!("  Total IP hands: {}", ip_hands.len());
        println!("  Equity vector length: {}", ip_equity.len());
        println!("  Equity at index: {:.1}%", ip_equity[ip_idx] * 100.0);
        println!();
    }

    // Now test what happens if we use wrong player number
    println!("═══════════════════════════════════════");
    println!("Testing potential bugs:\n");

    if let Some(oop_idx) = oop_hands.iter().position(|&(c1, c2)| {
        (c1 == jh && c2 == th) || (c1 == th && c2 == jh)
    }) {
        // What if we accidentally query IP equity with OOP index?
        let wrong_equity = game.equity(1); // IP equity

        println!("BUG TEST: Using OOP hand_idx ({}) with IP equity:", oop_idx);
        if oop_idx < wrong_equity.len() {
            println!("  Would show: {:.1}%", wrong_equity[oop_idx] * 100.0);
            println!("  (This is WRONG - it's showing IP's hand #{}'s equity!)", oop_idx + 1);
        }
        println!();
    }

    if let Some(ip_idx) = ip_hands.iter().position(|&(c1, c2)| {
        (c1 == jh && c2 == th) || (c1 == th && c2 == jh)
    }) {
        // What if we accidentally query OOP equity with IP index?
        let wrong_equity = game.equity(0); // OOP equity

        println!("BUG TEST: Using IP hand_idx ({}) with OOP equity:", ip_idx);
        if ip_idx < wrong_equity.len() {
            println!("  Would show: {:.1}%", wrong_equity[ip_idx] * 100.0);
            println!("  (This is WRONG - it's showing OOP's hand #{}'s equity!)", ip_idx + 1);
        }
        println!();
    }

    Ok(())
}
