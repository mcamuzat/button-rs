use serde::{Deserialize, Serialize};

use crate::action::SpecChoice;
use crate::die::{Die, Sides, SwingLetter};
use crate::ids::{DieId, PlayerId};
use crate::legal::{LegalActions, legal_actions};
use crate::scoring::{Holding, die_points, round_score};
use crate::setup::required_letters;
use crate::skill::DieSkill;
use crate::state::{GameState, Record, WaitingFor};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum Viewer {
    Seat { player: PlayerId },
    Spectator,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct DieView {
    pub id: DieId,
    /// R 1.1: the die as written in its button's recipe.
    pub notation: String,
    pub skills: Vec<DieSkill>,
    pub sides: Sides,
    pub min: u8,
    pub max: u8,
    pub value: u8,
    pub dizzy: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct CapturedView {
    pub die: DieView,
    /// Half-points.
    pub points: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct SlotView {
    pub id: DieId,
    pub notation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct SwingView {
    pub letter: SwingLetter,
    pub size: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct PlayerView {
    pub seat: PlayerId,
    pub button: String,
    pub recipe: String,
    pub active: Vec<DieView>,
    pub captured: Vec<CapturedView>,
    /// R 6.11: reserve dice not yet in play.
    pub reserve: Vec<SlotView>,
    pub record: Record,
    /// Half-points.
    pub round_score: i32,
    /// Committed swing sizes (public).
    pub swing: Vec<SwingView>,
    /// R 3.1: this player has submitted a choice that is not revealed yet.
    pub has_chosen: bool,
    /// Only shown to the player who made it.
    pub pending_choice: Option<SpecChoice>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct GameView {
    pub viewer: Viewer,
    pub round: u8,
    pub max_wins: u8,
    pub initiative: Option<PlayerId>,
    pub players: Vec<PlayerView>,
    pub waiting_for: WaitingFor,
    pub actors: Vec<PlayerId>,
    pub legal: LegalActions,
}

fn notation_of(state: &GameState, id: DieId) -> String {
    PlayerId::ALL
        .iter()
        .find_map(|&p| state.player(p).slot(id))
        .map(|s| s.spec.notation())
        .unwrap_or_default()
}

fn die_view(state: &GameState, die: &Die) -> DieView {
    DieView {
        id: die.id,
        notation: notation_of(state, die.id),
        skills: die.skills.iter().collect(),
        sides: die.sides,
        min: die.min(),
        max: die.max(),
        value: die.value,
        dizzy: die.dizzy,
    }
}

fn player_view(state: &GameState, seat: PlayerId, viewer: Viewer) -> PlayerView {
    let p = state.player(seat);
    let is_me = viewer == Viewer::Seat { player: seat };
    PlayerView {
        seat,
        button: p.button.clone(),
        recipe: p
            .slots
            .iter()
            .map(|s| s.spec.notation())
            .collect::<Vec<_>>()
            .join(" "),
        active: p.active.iter().map(|d| die_view(state, d)).collect(),
        captured: p
            .captured
            .iter()
            .map(|c| CapturedView {
                die: die_view(state, &c.die),
                points: die_points(&c.die, Holding::Captured(c.scoring)),
            })
            .collect(),
        reserve: p
            .slots
            .iter()
            .filter(|s| !s.in_play)
            .map(|s| SlotView {
                id: s.id,
                notation: s.spec.notation(),
            })
            .collect(),
        record: p.record,
        round_score: round_score(state, seat),
        swing: required_letters(p)
            .into_iter()
            .map(|letter| SwingView {
                letter,
                size: p.choice.swing_size(letter),
            })
            .collect(),
        has_chosen: p.pending_choice.is_some(),
        pending_choice: if is_me {
            p.pending_choice.clone()
        } else {
            None
        },
    }
}

/// What `viewer` may see of the match (R 3.1: unrevealed choices stay hidden).
pub fn view_for(state: &GameState, viewer: Viewer) -> GameView {
    GameView {
        viewer,
        round: state.round,
        max_wins: state.config.max_wins,
        initiative: state.initiative,
        players: PlayerId::ALL
            .iter()
            .map(|&p| player_view(state, p, viewer))
            .collect(),
        waiting_for: state.waiting_for.clone(),
        actors: state.actors(),
        legal: match viewer {
            Viewer::Seat { player } => legal_actions(state, player),
            Viewer::Spectator => LegalActions::Wait,
        },
    }
}
