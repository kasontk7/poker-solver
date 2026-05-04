use postflop_solver::*;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Simple Binary Save ===");

    // Minimal test
    let card_config = CardConfig {
        range: [
            "AA:1.0,KK:1.0".parse()?,
            "AA:1.0,KK:1.0".parse()?,
        ],
        flop: flop_from_str("KhQs6h")?,
        turn: NOT_DEALT,
        river: NOT_DEALT,
    };

    let bet_sizes = BetSizeOptions::try_from(("50%", "2x"))?;
    let tree_config = TreeConfig {
        initial_state: BoardState::Flop,
        starting_pot: 550,
        effective_stack: 9750,
        rake_rate: 0.0,
        rake_cap: 0.0,
        flop_bet_sizes: [bet_sizes.clone(), bet_sizes.clone()],
        turn_bet_sizes: [bet_sizes.clone(), bet_sizes.clone()],
        river_bet_sizes: [bet_sizes.clone(), bet_sizes],
        turn_donk_sizes: None,
        river_donk_sizes: None,
        add_allin_threshold: 1.5,
        force_allin_threshold: 0.15,
        merging_threshold: 0.1,
    };

    println!("Building and solving...");
    let action_tree = ActionTree::new(tree_config)?;
    let mut game = PostFlopGame::with_config(card_config, action_tree)?;
    game.allocate_memory(false);

    solve(&mut game, 50, 0.1, false);
    println!("✓ Solved\n");

    // Try to serialize using serde or similar
    println!("Attempting manual serialization...");

    // Get basic info we CAN extract
    game.back_to_root();
    game.cache_normalized_weights();

    let oop_cards = game.private_cards(0);
    let ip_cards = game.private_cards(1);
    let actions = game.available_actions();
    let strategy = game.strategy();
    let equity_oop = game.equity(0);
    let equity_ip = game.equity(1);

    println!("✓ Can extract strategy data:");
    println!("  OOP hands: {}", oop_cards.len());
    println!("  IP hands: {}", ip_cards.len());
    println!("  Actions: {}", actions.len());
    println!("  Strategy array length: {}", strategy.len());
    println!("  OOP equity array length: {}", equity_oop.len());
    println!("  IP equity array length: {}", equity_ip.len());

    println!("\n✓ We CAN query solved strategies!");
    println!("✓ The solve produces usable data!");
    println!("\n⚠️  We just need to decide HOW to persist it for EC2");

    Ok(())
}
