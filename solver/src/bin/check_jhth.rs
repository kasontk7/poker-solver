use postflop_solver::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let jh = card_from_str("Jh")?;
    let th = card_from_str("Th")?;
    let jd = card_from_str("Jd")?;
    let td = card_from_str("Td")?;

    println!("Card values:");
    println!("Jh = {}", jh);
    println!("Th = {}", th);
    println!("Jd = {}", jd);
    println!("Td = {}", td);
    println!();

    println!("Jh rank: {} ({})", jh / 4, if jh / 4 == 9 { "J" } else { "?" });
    println!("Th rank: {} ({})", th / 4, if th / 4 == 8 { "T" } else { "?" });
    println!("Jd rank: {} ({})", jd / 4, if jd / 4 == 9 { "J" } else { "?" });
    println!("Td rank: {} ({})", td / 4, if td / 4 == 8 { "T" } else { "?" });
    println!();

    println!("Jh suit: {} ({})", jh % 4, match jh % 4 { 1 => "h", _ => "?" });
    println!("Th suit: {} ({})", th % 4, match th % 4 { 1 => "h", _ => "?" });
    println!("Jd suit: {} ({})", jd % 4, match jd % 4 { 2 => "d", _ => "?" });
    println!("Td suit: {} ({})", td % 4, match td % 4 { 2 => "d", _ => "?" });
    println!();

    println!("JhTh is suited? {}", (jh % 4) == (th % 4));
    println!("JdTd is suited? {}", (jd % 4) == (td % 4));

    Ok(())
}
