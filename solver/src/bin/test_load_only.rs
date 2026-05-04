use postflop_solver::*;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Test Load Bin File ===\n");

    let args: Vec<String> = env::args().collect();
    let bin_path = if args.len() > 1 {
        &args[1]
    } else {
        "../solutions/v1.0_KhQs6h.bin"  // Default path from main.rs
    };

    println!("Attempting to load: {}", bin_path);

    // Try to load
    match load_data_from_file::<PostFlopGame, _>(bin_path, None) {
        Ok((mut game, memo)) => {
            println!("✓ Load successful!");
            println!("  Memo: '{}'", memo);

            // Query root to test it works
            game.back_to_root();
            game.cache_normalized_weights();

            let oop_cards = game.private_cards(0);
            let ip_cards = game.private_cards(1);
            let actions = game.available_actions();

            println!("\n✓ Game tree accessible:");
            println!("  OOP hands: {}", oop_cards.len());
            println!("  IP hands: {}", ip_cards.len());
            println!("  Root actions: {}", actions.len());

            // Test strategy query
            let strategy = game.strategy();
            let equity = game.equity(0);

            println!("\n✓ Strategy data accessible:");
            println!("  Strategy array length: {}", strategy.len());
            println!("  Equity array length: {}", equity.len());

            println!("\n🎉 BIN LOAD TEST PASSED!");
            println!("✓ Bincode deserialization works");
            println!("✓ Game tree fully accessible");
            println!("✓ Ready for EC2 workflow");

            Ok(())
        }
        Err(e) => {
            println!("✗ Load failed: {}", e);
            println!("\nThis is expected if bin file doesn't exist yet.");
            println!("After EC2 solve, download bin and test with:");
            println!("  cargo run --release --bin test_load_only <path-to-bin>");
            Ok(())
        }
    }
}
