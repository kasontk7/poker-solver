use postflop_solver::*;
use std::collections::HashMap;

/// Map any flop to its canonical representation (spades primary).
///
/// Examples:
/// - KhQd6h → KsQs6s (two-tone: h→s, d→h)
/// - AhKhQh → AsKsQs (monotone: h→s)
/// - As9d3c → As9h3d (rainbow: already canonical pattern)
pub fn map_to_canonical(flop_str: &str) -> Result<(String, SuitMapping), String> {
    if flop_str.len() != 6 {
        return Err(format!("Invalid flop string: {}", flop_str));
    }

    // Parse the three cards
    let c1 = &flop_str[0..2];
    let c2 = &flop_str[2..4];
    let c3 = &flop_str[4..6];

    let r1 = &c1[0..1];
    let r2 = &c2[0..1];
    let r3 = &c3[0..1];

    let s1 = &c1[1..2];
    let s2 = &c2[1..2];
    let s3 = &c3[1..2];

    // Count suits
    let mut suit_counts: HashMap<&str, usize> = HashMap::new();
    *suit_counts.entry(s1).or_insert(0) += 1;
    *suit_counts.entry(s2).or_insert(0) += 1;
    *suit_counts.entry(s3).or_insert(0) += 1;

    let num_unique_suits = suit_counts.len();

    // Build suit mapping
    let suit_map = match num_unique_suits {
        1 => {
            // Monotone - all → spades
            let original_suit = s1;
            SuitMapping {
                original_to_canonical: vec![(original_suit.to_string(), "s".to_string())]
                    .into_iter()
                    .collect(),
                canonical_to_original: vec![("s".to_string(), original_suit.to_string())]
                    .into_iter()
                    .collect(),
            }
        }
        2 => {
            // Two-tone - primary → spades, secondary → hearts
            let suits_vec: Vec<&str> = suit_counts.keys().copied().collect();

            // Primary = suit that appears twice (or first if both appear once - shouldn't happen)
            let primary = if suit_counts[&suits_vec[0]] >= 2 {
                suits_vec[0]
            } else if suit_counts[&suits_vec[1]] >= 2 {
                suits_vec[1]
            } else {
                // Both appear once (weird edge case for 2 suits), pick first
                suits_vec[0]
            };

            let secondary = if suits_vec[0] == primary {
                suits_vec[1]
            } else {
                suits_vec[0]
            };

            SuitMapping {
                original_to_canonical: vec![
                    (primary.to_string(), "s".to_string()),
                    (secondary.to_string(), "h".to_string()),
                ]
                .into_iter()
                .collect(),
                canonical_to_original: vec![
                    ("s".to_string(), primary.to_string()),
                    ("h".to_string(), secondary.to_string()),
                ]
                .into_iter()
                .collect(),
            }
        }
        3 => {
            // Rainbow - map to spades, hearts, diamonds in card order
            SuitMapping {
                original_to_canonical: vec![
                    (s1.to_string(), "s".to_string()),
                    (s2.to_string(), "h".to_string()),
                    (s3.to_string(), "d".to_string()),
                ]
                .into_iter()
                .collect(),
                canonical_to_original: vec![
                    ("s".to_string(), s1.to_string()),
                    ("h".to_string(), s2.to_string()),
                    ("d".to_string(), s3.to_string()),
                ]
                .into_iter()
                .collect(),
            }
        }
        _ => return Err("Invalid number of suits".to_string()),
    };

    // Apply mapping to create canonical flop
    let canonical_s1 = suit_map.to_canonical(s1);
    let canonical_s2 = suit_map.to_canonical(s2);
    let canonical_s3 = suit_map.to_canonical(s3);

    let canonical_flop = format!("{}{}{}{}{}{}", r1, canonical_s1, r2, canonical_s2, r3, canonical_s3);

    Ok((canonical_flop, suit_map))
}

/// Stores bidirectional suit mapping
#[derive(Debug)]
pub struct SuitMapping {
    pub original_to_canonical: HashMap<String, String>,
    pub canonical_to_original: HashMap<String, String>,
}

impl SuitMapping {
    pub fn to_canonical(&self, original_suit: &str) -> String {
        self.original_to_canonical
            .get(original_suit)
            .cloned()
            .unwrap_or_else(|| original_suit.to_string())
    }

    pub fn from_canonical(&self, canonical_suit: &str) -> String {
        self.canonical_to_original
            .get(canonical_suit)
            .cloned()
            .unwrap_or_else(|| canonical_suit.to_string())
    }

    /// Map a hand (e.g., "AhKd") to canonical suits
    pub fn map_hand_to_canonical(&self, hand: &str) -> String {
        if hand.len() != 4 {
            return hand.to_string();
        }

        let r1 = &hand[0..1];
        let s1 = &hand[1..2];
        let r2 = &hand[2..3];
        let s2 = &hand[3..4];

        format!(
            "{}{}{}{}",
            r1,
            self.to_canonical(s1),
            r2,
            self.to_canonical(s2)
        )
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Suit Mapper Test ===\n");

    let test_flops = vec![
        "KhQd6h", // Two-tone
        "AhKhQh", // Monotone
        "As9d3c", // Rainbow
        "KhQs6h", // Our test board
        "7d7h7s", // Three suits but paired
    ];

    for flop in test_flops {
        match map_to_canonical(flop) {
            Ok((canonical, mapping)) => {
                println!("Original: {}", flop);
                println!("Canonical: {}", canonical);
                println!("Mapping: {:?}", mapping);

                // Test hand mapping
                let test_hand = "AhKd";
                let canonical_hand = mapping.map_hand_to_canonical(test_hand);
                println!("  Hand {} → {}", test_hand, canonical_hand);

                println!();
            }
            Err(e) => {
                println!("Error mapping {}: {}\n", flop, e);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monotone() {
        let (canonical, _) = map_to_canonical("AhKhQh").unwrap();
        assert_eq!(canonical, "AsKsQs");
    }

    #[test]
    fn test_two_tone() {
        let (canonical, _) = map_to_canonical("KhQd6h").unwrap();
        // h appears twice, d once → h→s, d→h
        assert_eq!(canonical, "KsQh6s");
    }

    #[test]
    fn test_rainbow() {
        let (canonical, _) = map_to_canonical("As9d3c").unwrap();
        // First card → s, second → h, third → d
        assert_eq!(canonical, "As9h3d");
    }
}
