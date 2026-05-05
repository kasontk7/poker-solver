/// Find the closest solved flop for any given flop
///
/// Ranks flops by similarity using:
/// 1. High card match (most important)
/// 2. Connectivity (gapped vs connected)
/// 3. Paired vs unpaired
/// 4. Suit pattern (monotone vs two-tone vs rainbow)

use std::cmp::Ordering;

const RANKS: &[char] = &['2', '3', '4', '5', '6', '7', '8', '9', 'T', 'J', 'Q', 'K', 'A'];

fn rank_value(r: char) -> usize {
    RANKS.iter().position(|&x| x == r).unwrap_or(0)
}

#[derive(Debug, Clone)]
pub struct FlopFeatures {
    pub high_card: usize,
    pub mid_card: usize,
    pub low_card: usize,
    pub is_paired: bool,
    pub is_trips: bool,
    pub connectivity: i32,  // 0=rainbow, 1=one gap, 2=connected
    pub suit_pattern: SuitPattern,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SuitPattern {
    Monotone,
    TwoTone,
    Rainbow,
}

pub fn parse_flop(flop: &str) -> FlopFeatures {
    let r1 = flop.chars().nth(0).unwrap();
    let s1 = flop.chars().nth(1).unwrap();
    let r2 = flop.chars().nth(2).unwrap();
    let s2 = flop.chars().nth(3).unwrap();
    let r3 = flop.chars().nth(4).unwrap();
    let s3 = flop.chars().nth(5).unwrap();

    let mut ranks = vec![rank_value(r1), rank_value(r2), rank_value(r3)];
    ranks.sort_by(|a, b| b.cmp(a)); // Descending

    let high_card = ranks[0];
    let mid_card = ranks[1];
    let low_card = ranks[2];

    // Check paired/trips
    let is_paired = ranks[0] == ranks[1] || ranks[1] == ranks[2] || ranks[0] == ranks[2];
    let is_trips = ranks[0] == ranks[1] && ranks[1] == ranks[2];

    // Connectivity (for unpaired)
    let connectivity = if is_paired {
        0
    } else {
        let gap1 = (ranks[0] as i32 - ranks[1] as i32).abs();
        let gap2 = (ranks[1] as i32 - ranks[2] as i32).abs();
        let total_gap = gap1 + gap2;

        if total_gap == 2 {
            2 // Connected (e.g., JT9)
        } else if total_gap <= 4 {
            1 // One gap (e.g., J95)
        } else {
            0 // Disconnected (e.g., A72)
        }
    };

    // Suit pattern
    let unique_suits = vec![s1, s2, s3].iter().collect::<std::collections::HashSet<_>>().len();
    let suit_pattern = match unique_suits {
        1 => SuitPattern::Monotone,
        2 => SuitPattern::TwoTone,
        _ => SuitPattern::Rainbow,
    };

    FlopFeatures {
        high_card,
        mid_card,
        low_card,
        is_paired,
        is_trips,
        connectivity,
        suit_pattern,
    }
}

pub fn similarity_score(query: &FlopFeatures, candidate: &FlopFeatures) -> i32 {
    let mut score = 0;

    // High card match (most important) - within 2 ranks
    let high_diff = (query.high_card as i32 - candidate.high_card as i32).abs();
    score += match high_diff {
        0 => 100,
        1 => 80,
        2 => 50,
        _ => 0,
    };

    // Mid card match
    let mid_diff = (query.mid_card as i32 - candidate.mid_card as i32).abs();
    score += match mid_diff {
        0 => 50,
        1 => 30,
        2 => 15,
        _ => 0,
    };

    // Low card match (least important)
    let low_diff = (query.low_card as i32 - candidate.low_card as i32).abs();
    score += match low_diff {
        0 => 25,
        1 => 15,
        2 => 8,
        _ => 0,
    };

    // Paired/trips match (critical)
    if query.is_trips == candidate.is_trips {
        score += 80;
    }
    if query.is_paired == candidate.is_paired {
        score += 60;
    }

    // Connectivity match
    let conn_diff = (query.connectivity - candidate.connectivity).abs();
    score += match conn_diff {
        0 => 40,
        1 => 20,
        _ => 0,
    };

    // Suit pattern match (less important after canonicalization)
    if query.suit_pattern == candidate.suit_pattern {
        score += 20;
    }

    score
}

pub fn find_best_match(query_flop: &str, solved_flops: &[String]) -> (String, i32) {
    let query_features = parse_flop(query_flop);

    let mut best_match = solved_flops[0].clone();
    let mut best_score = -1;

    for candidate in solved_flops {
        let candidate_features = parse_flop(candidate);
        let score = similarity_score(&query_features, &candidate_features);

        if score > best_score {
            best_score = score;
            best_match = candidate.clone();
        }
    }

    (best_match, best_score)
}

fn main() {
    // Example solved flops (subset)
    let solved = vec![
        "AsKsQs".to_string(),
        "AsKsJs".to_string(),
        "KsQsJs".to_string(),
        "JsTs9s".to_string(),
        "9s8s7s".to_string(),
        "AsAsKs".to_string(),
        "KsKsQs".to_string(),
        "7s6s5s".to_string(),
        "As9s2s".to_string(),
    ];

    // Test queries
    let test_queries = vec![
        "AdKdQd",  // Should match AsKsQs
        "Th9h8h",  // Should match 9s8s7s or JsTs9s
        "KhKdQs",  // Should match KsKsQs
        "Ac8d3h",  // Should match As9s2s (high + low)
    ];

    println!("=== Flop Similarity Matcher ===\n");

    for query in test_queries {
        let (best_match, score) = find_best_match(query, &solved);
        println!("Query: {} → Best match: {} (score: {})", query, best_match, score);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let solved = vec!["AsKsQs".to_string()];
        let (match_flop, score) = find_best_match("AdKdQd", &solved);
        assert_eq!(match_flop, "AsKsQs");
        assert!(score > 200);
    }

    #[test]
    fn test_connectivity() {
        let solved = vec!["JsTs9s".to_string(), "As9s2s".to_string()];
        let (match_flop, _) = find_best_match("Th9h8h", &solved);
        assert_eq!(match_flop, "JsTs9s"); // Should prefer connected
    }
}
