use serde::{Deserialize, Serialize};

use crate::action::{DeclaredAttack, SpecChoice};
use crate::die::{Die, DieSpec};
use crate::ids::{DieId, PlayerId};
use crate::rng::SeededRng;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Button {
    pub name: String,
    pub dice: Vec<DieSpec>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct MatchConfig {
    pub max_wins: u8,
}

impl Default for MatchConfig {
    /// R 2.2: three round wins by default.
    fn default() -> Self {
        Self { max_wins: 3 }
    }
}

/// A die of the button, with its stable id. Reserve dice are out of play until chosen (R 6.11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DieSlot {
    pub id: DieId,
    pub spec: DieSpec,
    pub in_play: bool,
}

/// How a captured die scores (R 5.1, R 6.6, R 6.15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum CaptureScoring {
    BySize,
    ByValue(u8),
    Zero,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapturedDie {
    pub die: Die,
    pub scoring: CaptureScoring,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Record {
    pub wins: u8,
    pub losses: u8,
    pub draws: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerState {
    pub button: String,
    pub slots: Vec<DieSlot>,
    /// Committed swing/option sizes (public once revealed).
    pub choice: SpecChoice,
    /// R 3.1: submitted during SpecifyDice, hidden until every player has chosen.
    pub pending_choice: Option<SpecChoice>,
    pub active: Vec<Die>,
    /// Dice captured from the opponent this round.
    pub captured: Vec<CapturedDie>,
    /// R 6.22: Boom dice that left play this round; they score for nobody.
    pub out_of_play: Vec<Die>,
    pub record: Record,
}

impl PlayerState {
    pub fn die(&self, id: DieId) -> Option<&Die> {
        self.active.iter().find(|d| d.id == id)
    }

    pub fn die_mut(&mut self, id: DieId) -> Option<&mut Die> {
        self.active.iter_mut().find(|d| d.id == id)
    }

    pub fn slot(&self, id: DieId) -> Option<&DieSlot> {
        self.slots.iter().find(|s| s.id == id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum EndReason {
    Rounds,
    Surrender,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct MatchResult {
    pub winner: PlayerId,
    pub reason: EndReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum WaitingFor {
    SpecifyDice {
        pending: Vec<PlayerId>,
    },
    ChooseReserve {
        player: PlayerId,
    },
    ReactToInitiative {
        player: PlayerId,
    },
    Attack {
        player: PlayerId,
    },
    AdjustFire {
        player: PlayerId,
        attack: DeclaredAttack,
    },
    GameOver {
        result: MatchResult,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameState {
    pub config: MatchConfig,
    pub players: [PlayerState; 2],
    /// 1-based.
    pub round: u8,
    pub initiative: Option<PlayerId>,
    /// R 4.5: two consecutive passes end the round.
    pub consecutive_passes: u8,
    /// R 2.3: `None` in round 1 and after a draw.
    pub last_round_loser: Option<PlayerId>,
    pub waiting_for: WaitingFor,
    pub rng: SeededRng,
}

impl GameState {
    pub fn player(&self, p: PlayerId) -> &PlayerState {
        &self.players[p.index()]
    }

    pub fn player_mut(&mut self, p: PlayerId) -> &mut PlayerState {
        &mut self.players[p.index()]
    }

    /// An id no die of this match uses yet (R 6.20: Rage clones). Ids of slots are never
    /// reused, so a clone never collides with a die of the recipe.
    pub fn fresh_die_id(&self) -> DieId {
        let used = self.players.iter().flat_map(|p| {
            p.slots
                .iter()
                .map(|s| s.id)
                .chain(p.active.iter().map(|d| d.id))
                .chain(p.captured.iter().map(|c| c.die.id))
        });
        DieId(used.map(|d| d.0 + 1).max().unwrap_or(0))
    }

    /// The players who must act now (both during a simultaneous SpecifyDice).
    pub fn actors(&self) -> Vec<PlayerId> {
        match &self.waiting_for {
            WaitingFor::SpecifyDice { pending } => pending.clone(),
            WaitingFor::ChooseReserve { player }
            | WaitingFor::ReactToInitiative { player }
            | WaitingFor::Attack { player }
            | WaitingFor::AdjustFire { player, .. } => vec![*player],
            WaitingFor::GameOver { .. } => Vec::new(),
        }
    }
}
