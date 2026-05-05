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
    let bin_path = "../solutions/v1.1_KhQs6h.bin";
    let (mut game, _): (PostFlopGame, String) = load_data_from_file(bin_path, None)?;

    game.back_to_root();
    game.cache_normalized_weights();

    let oop_hands = game.private_cards(0).to_vec();
    let oop_equity = game.equity(0);

    println!("OOP hands with equity:\n");

    for (idx, &(c1, c2)) in oop_hands.iter().enumerate() {
        let equity = oop_equity[idx] * 100.0;
        let hand_str = format!("{}{}", card_to_string(c1), card_to_string(c2));

        if hand_str == "JhTh" || equity > 40.0 && equity < 50.0 {
            println!("{:3}. {} - {:.1}%", idx + 1, hand_str, equity);
        }
    }

    println!("\nLooking for hands around 43.9%:");
    for (idx, &(c1, c2)) in oop_hands.iter().enumerate() {
        let equity = oop_equity[idx] * 100.0;
        if (equity - 43.9).abs() < 1.0 {
            println!("{:3}. {} {} - {:.1}%", idx + 1, card_to_string(c1), card_to_string(c2), equity);
        }
    }

    Ok(())
}
