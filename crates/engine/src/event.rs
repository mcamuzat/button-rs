use serde::{Deserialize, Serialize};

use crate::action::{DeclaredAttack, SpecChoice};
use crate::die::Sides;
use crate::ids::{DieId, PlayerId};
use crate::state::MatchResult;

/// Why a die changed size during a round.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum ResizeCause {
    Mighty,
}

/// What happened during one `apply`, in order. Used for animations and the action log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum GameEvent {
    DiceSpecified {
        player: PlayerId,
        choice: SpecChoice,
    },
    ReserveAdded {
        player: PlayerId,
        die: DieId,
    },
    RoundStarted {
        round: u8,
    },
    DiceRolled {
        player: PlayerId,
        dice: Vec<(DieId, u8)>,
    },
    InitiativeDecided {
        player: PlayerId,
        by_coin: bool,
    },
    FocusTurnedDown {
        player: PlayerId,
        dice: Vec<(DieId, u8)>,
    },
    ChanceRerolled {
        player: PlayerId,
        die: DieId,
        value: u8,
    },
    InitiativePassed {
        player: PlayerId,
    },
    Attacked {
        player: PlayerId,
        attack: DeclaredAttack,
    },
    FireUsed {
        player: PlayerId,
        fire: Vec<(DieId, u8)>,
        boosts: Vec<(DieId, u8)>,
    },
    TripRolled {
        attacker: DieId,
        attacker_value: u8,
        target: DieId,
        target_value: u8,
        captured: bool,
    },
    DieCaptured {
        by: PlayerId,
        die: DieId,
    },
    DieRerolled {
        die: DieId,
        value: u8,
    },
    DieShrunk {
        die: DieId,
        sides: Sides,
    },
    /// R 6.17 (and milestone 4B): a die took a new size before rolling.
    DieResized {
        die: DieId,
        from: Sides,
        to: Sides,
        cause: ResizeCause,
    },
    Passed {
        player: PlayerId,
    },
    /// Scores in half-points.
    RoundEnded {
        round: u8,
        scores: [i32; 2],
        winner: Option<PlayerId>,
    },
    MatchEnded {
        result: MatchResult,
    },
}
