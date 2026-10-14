use serde::{Deserialize, Serialize};

use crate::die::SwingLetter;
use crate::ids::DieId;

/// R 1.2–1.3: the sizes chosen for swing letters and option dice.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct SpecChoice {
    pub swing: Vec<(SwingLetter, u8)>,
    pub options: Vec<(DieId, u8)>,
}

impl SpecChoice {
    pub fn swing_size(&self, letter: SwingLetter) -> Option<u8> {
        self.swing
            .iter()
            .find(|(l, _)| *l == letter)
            .map(|(_, s)| *s)
    }

    pub fn option_size(&self, die: DieId) -> Option<u8> {
        self.options
            .iter()
            .find(|(d, _)| *d == die)
            .map(|(_, s)| *s)
    }
}

/// R 4.2, R 4.3, R 6.1, R 6.9, R 6.10, R 6.14, R 6.21.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum AttackKind {
    Power,
    Skill,
    Shadow,
    Speed,
    Trip,
    Berserk,
    Rush,
}

impl AttackKind {
    pub const ALL: [AttackKind; 7] = [
        AttackKind::Power,
        AttackKind::Skill,
        AttackKind::Shadow,
        AttackKind::Speed,
        AttackKind::Trip,
        AttackKind::Berserk,
        AttackKind::Rush,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct DeclaredAttack {
    pub kind: AttackKind,
    pub attackers: Vec<DieId>,
    pub targets: Vec<DieId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum Action {
    /// R 3.1 / R 2.3.
    SpecifyDice {
        choice: SpecChoice,
    },
    /// R 6.11: `None` declines.
    ChooseReserve {
        die: Option<DieId>,
    },
    /// R 6.4: the new (lower) values.
    FocusTurnDown {
        dice: Vec<(DieId, u8)>,
    },
    /// R 6.2.
    ChanceReroll {
        die: DieId,
    },
    /// R 3.5: let the opponent start.
    PassInitiative,
    Attack {
        attack: DeclaredAttack,
    },
    /// R 6.3: `fire` = amounts taken from Fire dice, `boosts` = amounts added to attackers.
    FireTurnDown {
        fire: Vec<(DieId, u8)>,
        boosts: Vec<(DieId, u8)>,
    },
    CancelFire,
    /// R 4.1.
    Pass,
    /// R 2.4.
    Surrender,
}
