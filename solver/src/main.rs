use postflop_solver::*;
use std::fs;

fn load_range_from_file(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;

    // Convert from our format (AA:1.0000) to postflop-solver format (AA,KK,QQ:0.5)
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

        // postflop-solver format: hand or hand:freq
        if (freq - 1.0).abs() < 0.001 {
            // Frequency is 1.0, just add hand
            hands.push(hand.to_string());
        } else {
            // Frequency is not 1.0, add hand:freq
            hands.push(format!("{}:{:.4}", hand, freq));
        }
    }

    Ok(hands.join(","))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Poker Solver v1.0 ===");
    println!("Scenario: BTN RFI vs BB call");
    println!("Board: KhQs6h (full tree - all turns/rivers for v1.1)");
    println!();

    // Load ranges
    println!("Loading ranges...");
    let ip_range = load_range_from_file("ranges/gto/BTN/RFI.txt")?;
    let oop_range = load_range_from_file("ranges/gto/BB/defend_vs_RFI_BTN.txt")?;

    println!("  IP (BTN): {} chars", ip_range.len());
    println!("  OOP (BB): {} chars", oop_range.len());
    println!();

    // Card configuration
    // v1.1: Full tree with all possible turns and rivers (NOT_DEALT)
    // This validates the full solving pipeline before scaling to 184 flops
    println!("Configuring game...");
    let card_config = CardConfig {
        range: [
            oop_range.parse()?,  // BB is OOP (out of position)
            ip_range.parse()?,   // BTN is IP (in position)
        ],
        flop: flop_from_str("KhQs6h")?,
        turn: NOT_DEALT,
        river: NOT_DEALT,
    };

    // Bet sizes configuration
    // Flop: 25%, 50%, 100%
    // Turn: 25%, 50%, 100%, all-in
    // River: 50%, 100%, 150%, all-in
    let flop_bet_sizes = BetSizeOptions::try_from(("25%, 50%, 100%", "2.5x"))?;
    let turn_bet_sizes = BetSizeOptions::try_from(("25%, 50%, 100%, a", "2.5x"))?;
    let river_bet_sizes = BetSizeOptions::try_from(("50%, 100%, 150%, a", "2.5x"))?;

    // Donk bet sizes: 33%, 50%
    let donk_sizes = DonkSizeOptions::try_from("33%, 50%")?;

    // Tree configuration
    // Starting pot: 5.5bb (SB 0.5bb + BB 2.5bb + BTN 2.5bb)
    // Effective stack: 97.5bb
    // At $0.50/$1: starting_pot = $5.50, effective_stack = $97.50
    // v1.1: Full tree (turn/river NOT_DEALT) so we start from Flop state
    let tree_config = TreeConfig {
        initial_state: BoardState::Flop,  // Flop state since turn/river are NOT_DEALT
        starting_pot: 550,      // $5.50 in cents
        effective_stack: 9750,  // $97.50 in cents
        rake_rate: 0.05,        // 5% ($0.01 per $0.20)
        rake_cap: 300.0,        // $3.00 cap in cents
        flop_bet_sizes: [flop_bet_sizes.clone(), flop_bet_sizes.clone()],
        turn_bet_sizes: [turn_bet_sizes.clone(), turn_bet_sizes.clone()],
        river_bet_sizes: [river_bet_sizes.clone(), river_bet_sizes],
        turn_donk_sizes: Some(donk_sizes.clone()),
        river_donk_sizes: Some(donk_sizes),
        add_allin_threshold: 1.5,
        force_allin_threshold: 0.15,
        merging_threshold: 0.1,
    };

    println!("  Starting pot: ${:.2} (5.5bb)", tree_config.starting_pot as f32 / 100.0);
    println!("  Effective stack: ${:.2} (97.5bb)", tree_config.effective_stack as f32 / 100.0);
    println!("  Rake: {:.1}% capped at ${:.2}", tree_config.rake_rate * 100.0, tree_config.rake_cap / 100.0);
    println!();

    // Build game tree
    println!("Building game tree...");
    let action_tree = ActionTree::new(tree_config)?;
    let mut game = PostFlopGame::with_config(card_config, action_tree)?;

    // Check memory usage
    let (mem_usage, mem_usage_compressed) = game.memory_usage();
    println!("  Memory (32-bit float): {:.2} MB", mem_usage as f64 / (1024.0 * 1024.0));
    println!("  Memory (16-bit compressed): {:.2} MB", mem_usage_compressed as f64 / (1024.0 * 1024.0));
    println!();

    // Allocate memory (use 32-bit float for now)
    println!("Allocating memory...");
    game.allocate_memory(false);
    println!();

    // Solve the game
    println!("Solving game...");
    let max_iterations = 1000;
    let target_exploitability = game.tree_config().starting_pot as f32 * 0.005; // 0.5% of pot
    println!("  Max iterations: {}", max_iterations);
    println!("  Target exploitability: {:.2} cents ({:.1}% of pot)", target_exploitability, 0.5);
    println!();

    let exploitability = solve(&mut game, max_iterations, target_exploitability, true);
    println!();
    println!("✓ Solve complete!");
    println!("  Final exploitability: {:.2} cents", exploitability);
    println!();

    // Save the solved game tree
    println!("Saving solution...");
    let output_path = "solutions/v1.0_KhQs6h.bin";
    fs::create_dir_all("solutions")?;

    game.save(output_path, true)?;
    let file_size = fs::metadata(output_path)?.len();
    println!("  Saved to: {}", output_path);
    println!("  File size: {:.2} MB", file_size as f64 / (1024.0 * 1024.0));

    println!();
    println!("=== Done! ===");

    Ok(())
}
