use postflop_solver::*;
use std::fs;
use std::time::Instant;
use std::env;

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
    let total_start = Instant::now();

    // Args: scenario board
    // Example: batch_solver BTN_RFI_vs_BB_defend AsKsQs
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: batch_solver <scenario> <board>");
        eprintln!("Example: batch_solver BTN_RFI_vs_BB_defend AsKsQs");
        std::process::exit(1);
    }

    let scenario = &args[1];
    let board_str = &args[2];

    println!("=== Poker Solver v1.2 ===");
    println!("Scenario: {}", scenario);
    println!("Board: {} (full tree)", board_str);
    println!();

    // Parse scenario to get range paths
    let (oop_path, ip_path, pot_size, stack_size) = parse_scenario(scenario)?;

    // Load ranges
    println!("Loading ranges...");
    let oop_range = load_range_from_file(&oop_path)?;
    let ip_range = load_range_from_file(&ip_path)?;

    println!("  OOP: {}", oop_path);
    println!("  IP: {}", ip_path);
    println!();

    // Parse board
    println!("Configuring game...");
    let card_config = CardConfig {
        range: [
            oop_range.parse()?,
            ip_range.parse()?,
        ],
        flop: flop_from_str(board_str)?,
        turn: NOT_DEALT,
        river: NOT_DEALT,
    };

    // Bet sizes
    let flop_bet_sizes = BetSizeOptions::try_from(("50%, 100%", "3x, 5x"))?;
    let turn_bet_sizes = BetSizeOptions::try_from(("50%, 100%, 150%", "3x, 5x"))?;
    let river_bet_sizes = BetSizeOptions::try_from(("75%, 150%, a", "3x, 5x"))?;
    let donk_sizes = DonkSizeOptions::try_from("50%")?;

    // Tree configuration
    let tree_config = TreeConfig {
        initial_state: BoardState::Flop,
        starting_pot: pot_size,
        effective_stack: stack_size,
        flop_bet_sizes: [flop_bet_sizes.clone(), flop_bet_sizes.clone()],
        turn_bet_sizes: [turn_bet_sizes.clone(), turn_bet_sizes.clone()],
        river_bet_sizes: [river_bet_sizes.clone(), river_bet_sizes],
        turn_donk_sizes: Some(donk_sizes.clone()),
        river_donk_sizes: Some(donk_sizes),
        add_all_in_threshold: 1.5,
        force_all_in_threshold: 0.15,
        merging_threshold: 0.1,
    };

    // Build game tree
    println!("Building game tree...");
    let action_tree = ActionTree::new(tree_config)?;
    let mut game = PostFlopGame::with_config(card_config, action_tree)?;

    let (mem_usage, mem_usage_compressed) = game.memory_usage();
    println!("  Memory (16-bit compressed): {:.2} GB", mem_usage_compressed as f64 / (1024.0 * 1024.0 * 1024.0));
    println!();

    // Allocate memory with 16-bit compression
    println!("Allocating memory...");
    game.allocate_memory(true);

    // Solve
    println!("Solving game...");
    let max_iterations = 500;
    let target_exploitability = 5.0;

    let solve_start = Instant::now();
    let exploitability = solve(&mut game, max_iterations, target_exploitability, true);
    let solve_duration = solve_start.elapsed();

    println!();
    println!("✓ Solve complete!");
    println!("  Final exploitability: {:.2} cents", exploitability);
    println!("  Solve time: {:.2} minutes", solve_duration.as_secs_f64() / 60.0);
    println!();

    // Save with new naming: {scenario}___{board}.bin
    println!("Saving solution...");
    let output_path = format!("solutions/{}___{}.bin", scenario, board_str);
    fs::create_dir_all("solutions")?;

    let memo = format!("{} {}", scenario, board_str);
    save_data_to_file(&game, &memo, &output_path, None)?;
    let file_size = fs::metadata(&output_path)?.len();
    println!("  Saved to: {}", output_path);
    println!("  File size: {:.2} GB", file_size as f64 / (1024.0 * 1024.0 * 1024.0));

    let total_duration = total_start.elapsed();
    println!();
    println!("Total time: {:.2} minutes", total_duration.as_secs_f64() / 60.0);

    Ok(())
}

fn parse_scenario(scenario: &str) -> Result<(String, String, i32, i32), Box<dyn std::error::Error>> {
    let base = "ranges/gto";

    // RFI scenarios (pot: 5.5bb, stack: 97.5bb)
    if scenario.contains("_RFI_vs_") && scenario.contains("_defend") {
        let parts: Vec<&str> = scenario.split("_RFI_vs_").collect();
        let opener = parts[0];
        let defender = parts[1].replace("_defend", "");
        return Ok((
            format!("{}/rfi/{}/{}.txt", base, scenario, defender.to_lowercase()),
            format!("{}/rfi/{}/{}.txt", base, scenario, opener.to_lowercase()),
            550,
            9750,
        ));
    }

    if scenario.contains("_RFI_vs_") && scenario.contains("_cold_call") {
        let parts: Vec<&str> = scenario.split("_RFI_vs_").collect();
        let opener = parts[0];
        let caller = parts[1].replace("_cold_call", "");
        return Ok((
            format!("{}/rfi/{}/{}.txt", base, scenario, caller.to_lowercase()),
            format!("{}/rfi/{}/{}.txt", base, scenario, opener.to_lowercase()),
            550,
            9750,
        ));
    }

    // 3bet scenarios (pot: 21bb, stack: 79bb)
    if scenario.contains("_3bet_vs_") && scenario.contains("_call") {
        let parts: Vec<&str> = scenario.split("_3bet_vs_").collect();
        let threebettor = parts[0];
        let caller = parts[1].replace("_call", "");
        return Ok((
            format!("{}/3bet/{}/{}.txt", base, scenario, threebettor.to_lowercase()),
            format!("{}/3bet/{}/{}.txt", base, scenario, caller.to_lowercase()),
            2100,
            7900,
        ));
    }

    // 4bet scenarios (pot: 51bb, stack: 49bb)
    if scenario.contains("_4bet_vs_") && scenario.contains("_call") {
        let parts: Vec<&str> = scenario.split("_4bet_vs_").collect();
        let fourbettor = parts[0];
        let caller = parts[1].replace("_call", "");
        return Ok((
            format!("{}/4bet/{}/{}.txt", base, scenario, fourbettor.to_lowercase()),
            format!("{}/4bet/{}/{}.txt", base, scenario, caller.to_lowercase()),
            5100,
            4900,
        ));
    }

    Err(format!("Unknown scenario format: {}", scenario).into())
}
