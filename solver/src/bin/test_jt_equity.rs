use postflop_solver::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading solution...");
    let (mut game, _): (PostFlopGame, String) = load_data_from_file("../solutions/v1.1_KhQs6h.bin", None)?;
    
    game.back_to_root();
    game.cache_normalized_weights();
    
    let oop_hands = game.private_cards(0);
    let ip_hands = game.private_cards(1);
    
    // Find JhTh for OOP
    let jh_th_oop = oop_hands.iter().position(|&(c1, c2)| {
        let c1_str = format!("{}", if c1 / 4 == 9 { 'J' } else if c1 / 4 == 8 { 'T' } else { '?' });
        let c2_str = format!("{}", if c2 / 4 == 9 { 'J' } else if c2 / 4 == 8 { 'T' } else { '?' });
        let s1 = c1 % 4 == 1; // hearts
        let s2 = c2 % 4 == 1; // hearts
        (c1_str == "J" || c1_str == "T") && (c2_str == "J" || c2_str == "T") && s1 && s2 && c1 != c2
    });
    
    if let Some(idx) = jh_th_oop {
        let (c1, c2) = oop_hands[idx];
        let equity_oop = game.equity(0);
        println!("JhTh OOP: cards {} {}, equity {:.1}%", c1, c2, equity_oop[idx] * 100.0);
    } else {
        println!("JhTh not found in OOP range (might be blocked or not in defend range)");
    }
    
    // Find JhTh for IP
    let jh_th_ip = ip_hands.iter().position(|&(c1, c2)| {
        let c1_str = format!("{}", if c1 / 4 == 9 { 'J' } else if c1 / 4 == 8 { 'T' } else { '?' });
        let c2_str = format!("{}", if c2 / 4 == 9 { 'J' } else if c2 / 4 == 8 { 'T' } else { '?' });
        let s1 = c1 % 4 == 1; // hearts
        let s2 = c2 % 4 == 1; // hearts
        (c1_str == "J" || c1_str == "T") && (c2_str == "J" || c2_str == "T") && s1 && s2 && c1 != c2
    });
    
    if let Some(idx) = jh_th_ip {
        let (c1, c2) = ip_hands[idx];
        let equity_ip = game.equity(1);
        println!("JhTh IP: cards {} {}, equity {:.1}%", c1, c2, equity_ip[idx] * 100.0);
    } else {
        println!("JhTh not found in IP range");
    }
    
    Ok(())
}
