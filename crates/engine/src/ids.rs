use serde::{Deserialize, Serialize};

/// A seat at the table. Button Men is played by two players (R 2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum PlayerId {
    One,
    Two,
}

impl PlayerId {
    pub const ALL: [PlayerId; 2] = [PlayerId::One, PlayerId::Two];

    pub fn index(self) -> usize {
        match self {
            PlayerId::One => 0,
            PlayerId::Two => 1,
        }
    }

    pub fn other(self) -> PlayerId {
        match self {
            PlayerId::One => PlayerId::Two,
            PlayerId::Two => PlayerId::One,
        }
    }
}

/// Stable identifier of a die for the whole match: it survives rounds and captures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct DieId(pub u16);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn other_is_an_involution() {
        for p in PlayerId::ALL {
            assert_ne!(p, p.other());
            assert_eq!(p, p.other().other());
        }
        assert_eq!(PlayerId::One.index(), 0);
        assert_eq!(PlayerId::Two.index(), 1);
    }
}
