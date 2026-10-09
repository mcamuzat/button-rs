use serde::{Deserialize, Serialize};

/// The only source of randomness of a match (splitmix64). Serializable, so a match
/// can be saved and replayed exactly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeededRng {
    state: u64,
}

impl SeededRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// splitmix64.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A fair roll in `1..=sides`, by rejection sampling (no modulo bias).
    pub fn roll(&mut self, sides: u8) -> u8 {
        assert!(sides >= 1, "a die has at least one side");
        let n = u64::from(sides);
        let zone = u64::MAX - (u64::MAX % n);
        loop {
            let x = self.next_u64();
            if x < zone {
                return (x % n) as u8 + 1;
            }
        }
    }

    pub fn coin(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = SeededRng::new(42);
        let mut b = SeededRng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn roll_stays_in_range_and_hits_every_face() {
        let mut rng = SeededRng::new(7);
        let mut seen = [false; 7];
        for _ in 0..10_000 {
            let v = rng.roll(6);
            assert!((1..=6).contains(&v));
            seen[v as usize] = true;
        }
        assert!(seen[1..=6].iter().all(|&s| s));
    }

    #[test]
    fn one_sided_die_always_rolls_one() {
        let mut rng = SeededRng::new(1);
        for _ in 0..50 {
            assert_eq!(rng.roll(1), 1);
        }
    }

    #[test]
    fn coin_produces_both_sides() {
        let mut rng = SeededRng::new(3);
        let heads = (0..1000).filter(|_| rng.coin()).count();
        assert!((350..650).contains(&heads));
    }
}
