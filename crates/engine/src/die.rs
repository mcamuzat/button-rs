use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::ids::DieId;
use crate::rng::SeededRng;
use crate::skill::{DieSkill, SkillSet};

/// R 1.2: swing letters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum SwingLetter {
    P,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
}

impl SwingLetter {
    pub const ALL: [SwingLetter; 10] = [
        SwingLetter::P,
        SwingLetter::R,
        SwingLetter::S,
        SwingLetter::T,
        SwingLetter::U,
        SwingLetter::V,
        SwingLetter::W,
        SwingLetter::X,
        SwingLetter::Y,
        SwingLetter::Z,
    ];

    /// R 1.2: allowed sizes, from buttonweavers' `BMDieSwing::$swingRanges`.
    pub fn range(self) -> RangeInclusive<u8> {
        match self {
            SwingLetter::P => 1..=30,
            SwingLetter::R => 2..=16,
            SwingLetter::S => 6..=20,
            SwingLetter::T => 2..=12,
            SwingLetter::U => 8..=30,
            SwingLetter::V => 6..=12,
            SwingLetter::W => 4..=12,
            SwingLetter::X => 4..=20,
            SwingLetter::Y => 1..=20,
            SwingLetter::Z => 4..=30,
        }
    }

    pub fn letter(self) -> char {
        match self {
            SwingLetter::P => 'P',
            SwingLetter::R => 'R',
            SwingLetter::S => 'S',
            SwingLetter::T => 'T',
            SwingLetter::U => 'U',
            SwingLetter::V => 'V',
            SwingLetter::W => 'W',
            SwingLetter::X => 'X',
            SwingLetter::Y => 'Y',
            SwingLetter::Z => 'Z',
        }
    }

    pub fn from_letter(c: char) -> Option<SwingLetter> {
        SwingLetter::ALL.into_iter().find(|l| l.letter() == c)
    }
}

/// R 1.4: one half of a twin die.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum TwinPart {
    Fixed(u8),
    Swing(SwingLetter),
}

/// The shape of a die as written in a recipe (R 1.1–1.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum DieShape {
    Fixed(u8),
    Swing(SwingLetter),
    Option(u8, u8),
    Twin(TwinPart, TwinPart),
}

/// One die of a recipe: shape + skills.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct DieSpec {
    pub shape: DieShape,
    pub skills: SkillSet,
}

impl DieSpec {
    pub fn new(shape: DieShape) -> Self {
        Self {
            shape,
            skills: SkillSet::default(),
        }
    }

    pub fn with(mut self, skill: DieSkill) -> Self {
        self.skills.insert(skill);
        self
    }

    /// R 1.1: the recipe notation of this die, skills first.
    pub fn notation(&self) -> String {
        let skills: String = self.skills.iter().map(DieSkill::letter).collect();
        let part = |p: &TwinPart| match p {
            TwinPart::Fixed(n) => n.to_string(),
            TwinPart::Swing(l) => l.letter().to_string(),
        };
        let core = match &self.shape {
            DieShape::Fixed(n) => n.to_string(),
            DieShape::Swing(l) => l.letter().to_string(),
            DieShape::Option(a, b) => format!("{a}/{b}"),
            DieShape::Twin(a, b) => format!("{},{}", part(a), part(b)),
        };
        format!("{skills}({core})")
    }
}

/// The resolved number of sides of a die in play.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum Sides {
    Single(u8),
    Twin(u8, u8),
}

impl Sides {
    /// R 1.4–1.5: the highest value (the size).
    pub fn max(self) -> u8 {
        match self {
            Sides::Single(n) => n,
            Sides::Twin(a, b) => a + b,
        }
    }

    /// R 1.4–1.5: the lowest value.
    pub fn min(self) -> u8 {
        match self {
            Sides::Single(_) => 1,
            Sides::Twin(_, _) => 2,
        }
    }

    pub fn roll(self, rng: &mut SeededRng) -> u8 {
        match self {
            Sides::Single(n) => rng.roll(n),
            Sides::Twin(a, b) => rng.roll(a) + rng.roll(b),
        }
    }

    /// R 6.1: half the sides, rounded up; a twin halves each of its halves.
    pub fn halved(self) -> Sides {
        match self {
            Sides::Single(n) => Sides::Single(n.div_ceil(2)),
            Sides::Twin(a, b) => Sides::Twin(a.div_ceil(2), b.div_ceil(2)),
        }
    }
}

/// A die in play (or captured) during a round.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Die {
    pub id: DieId,
    pub skills: SkillSet,
    pub sides: Sides,
    pub value: u8,
    /// R 6.4 / R 7.4: a Focus die turned down by the starting player cannot attack
    /// until the end of that player's first turn.
    pub dizzy: bool,
}

impl Die {
    pub fn max(&self) -> u8 {
        self.sides.max()
    }

    pub fn min(&self) -> u8 {
        self.sides.min()
    }

    pub fn has(&self, skill: DieSkill) -> bool {
        self.skills.has(skill)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::DieSkill;

    #[test]
    fn swing_ranges_match_buttonweavers() {
        assert_eq!(SwingLetter::X.range(), 4..=20);
        assert_eq!(SwingLetter::R.range(), 2..=16);
        assert_eq!(SwingLetter::P.range(), 1..=30);
        assert_eq!(SwingLetter::from_letter('Z'), Some(SwingLetter::Z));
        assert_eq!(SwingLetter::from_letter('Q'), None);
    }

    #[test]
    fn sides_bounds() {
        assert_eq!(Sides::Single(6).max(), 6);
        assert_eq!(Sides::Single(6).min(), 1);
        assert_eq!(Sides::Twin(4, 6).max(), 10);
        assert_eq!(Sides::Twin(4, 6).min(), 2);
    }

    #[test]
    fn halved_rounds_up_and_halves_twins_per_half() {
        assert_eq!(Sides::Single(7).halved(), Sides::Single(4));
        assert_eq!(Sides::Single(1).halved(), Sides::Single(1));
        assert_eq!(Sides::Twin(5, 4).halved(), Sides::Twin(3, 2));
    }

    #[test]
    fn twin_roll_is_sum_of_halves() {
        let mut rng = SeededRng::new(9);
        for _ in 0..500 {
            let v = Sides::Twin(1, 1).roll(&mut rng);
            assert_eq!(v, 2);
            let w = Sides::Twin(6, 6).roll(&mut rng);
            assert!((2..=12).contains(&w));
        }
    }

    #[test]
    fn notation_follows_recipe_syntax() {
        let spec = DieSpec::new(DieShape::Swing(SwingLetter::X))
            .with(DieSkill::Poison)
            .with(DieSkill::Speed);
        assert_eq!(spec.notation(), "pz(X)");
        assert_eq!(DieSpec::new(DieShape::Option(6, 10)).notation(), "(6/10)");
        assert_eq!(
            DieSpec::new(DieShape::Twin(
                TwinPart::Fixed(4),
                TwinPart::Swing(SwingLetter::Y)
            ))
            .notation(),
            "(4,Y)"
        );
    }
}
