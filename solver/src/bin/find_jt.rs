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

    let jh = card_from_str("Jh")?;
    let th = card_from_str("Th")?;

    println!("Searching for JhTh in OOP range...\n");

    for (idx, &(c1, c2)) in oop_hands.iter().enumerate() {
        let equity = oop_equity[idx] * 100.0;
        let c1_str = card_to_string(c1);
        let c2_str = card_to_string(c2);

        // Check if this is JT in any form
        if (c1 == jh || c2 == jh) && (c1 == th || c2 == th) {
            println!("FOUND: {:3}. {} {} - {:.1}%", idx + 1, c1_str, c2_str, equity);
        }

        // Also print all JT combinations
        let c1_rank = c1 / 4;
        let c2_rank = c2 / 4;
        if (c1_rank == 9 && c2_rank == 8) || (c1_rank == 8 && c2_rank == 9) {
            println!("{:3}. {} {} - {:.1}%", idx + 1, c1_str, c2_str, equity);
        }
    }

    Ok(())
}
