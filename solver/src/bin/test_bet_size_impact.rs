/// Test: Does simplifying turn/river bet sizes affect flop strategy?
use postflop_solver::*;
use std::time::Instant;

fn solve_with_config(
    turn_sizes: &str,
    river_sizes: &str,
    name: &str,
) -> Result<(PostFlopGame, f64), Box<dyn std::error::Error>> {
    println!("=== Solving: {} ===", name);
    println!("Turn: {}, River: {}", turn_sizes, river_sizes);

    let oop_range = "88+,ATs+,KTs+,QTs+,JTs,T9s,98s,87s,76s,AJo+,KQo";
    let ip_range = "66+,A2s+,K5s+,Q8s+,J8s+,T8s+,98s,87s,ATo+,KJo+";

    let card_config = CardConfig {
        range: [oop_range.parse()?, ip_range.parse()?],
        flop: flop_from_str("KhQs6h")?,
        turn: NOT_DEALT,
        river: NOT_DEALT,
    };

    let flop_bet = BetSizeOptions::try_from(("50%, 100%", "3x"))?;
    let turn_bet = BetSizeOptions::try_from((turn_sizes, "3x"))?;
    let river_bet = BetSizeOptions::try_from((river_sizes, "3x"))?;

    let tree_config = TreeConfig {
        initial_state: BoardState::Flop,
        starting_pot: 550,
        effective_stack: 9750,
        rake_rate: 0.0,
        rake_cap: 0.0,
        flop_bet_sizes: [flop_bet.clone(), flop_bet],
        turn_bet_sizes: [turn_bet.clone(), turn_bet],
        river_bet_sizes: [river_bet.clone(), river_bet],
        turn_donk_sizes: None,
        river_donk_sizes: None,
        add_allin_threshold: 1.5,
        force_allin_threshold: 0.15,
        merging_threshold: 0.1,
    };

    let action_tree = ActionTree::new(tree_config)?;
    let mut game = PostFlopGame::with_config(card_config, action_tree)?;
    game.allocate_memory(true);

    let start = Instant::now();
    solve(&mut game, 200, 0.5, false);
    let solve_time = start.elapsed().as_secs_f64();

    println!("Solve time: {:.2}s\n", solve_time);

    Ok((game, solve_time))
}

fn show_strategy(game: &PostFlopGame, hand: &str) -> Result<(), Box<dyn std::error::Error>> {
    let hand_card1 = card_from_str(&hand[0..2])?;
    let hand_card2 = card_from_str(&hand[2..4])?;

    let oop_cards = game.private_cards(0);
    let actions = game.available_actions();
    let strategy = game.strategy();

    let hand_idx = oop_cards.iter().position(|&(c1, c2)|
        (c1 == hand_card1 && c2 == hand_card2) || (c1 == hand_card2 && c2 == hand_card1));

    if let Some(idx) = hand_idx {
        println!("Flop strategy for {}:", hand);
        for (i, action) in actions.iter().enumerate() {
            let freq = strategy[idx + i * oop_cards.len()];
            if freq > 0.01 {
                println!("  {:?}: {:.1}%", action, freq * 100.0);
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Bet Size Impact Test ===\n");
    println!("Testing how turn/river bet size complexity affects flop strategy\n");

    // Test 1: Full complexity (baseline)
    let (mut game1, time1) = solve_with_config(
        "25%, 50%, 75%, 100%, 150%, a",
        "50%, 75%, 100%, 150%, 200%, a",
        "Full complexity (6 turn, 6 river sizes)"
    )?;
    game1.cache_normalized_weights();
    show_strategy(&game1, "6c6d")?;
    println!();

    // Test 2: Moderate complexity
    let (mut game2, time2) = solve_with_config(
        "50%, 100%",
        "75%, 150%, a",
        "Moderate (2 turn, 3 river sizes)"
    )?;
    game2.cache_normalized_weights();
    show_strategy(&game2, "6c6d")?;
    println!();

    // Test 3: Minimal complexity (1 size)
    let (mut game3, time3) = solve_with_config(
        "75%",
        "75%",
        "Minimal (1 turn, 1 river size)"
    )?;
    game3.cache_normalized_weights();
    show_strategy(&game3, "6c6d")?;
    println!();

    println!("=== Summary ===");
    println!("Full complexity: {:.2}s", time1);
    println!("Moderate: {:.2}s ({:.1}x faster)", time2, time1 / time2);
    println!("Minimal: {:.2}s ({:.1}x faster)", time3, time1 / time3);

    Ok(())
}
