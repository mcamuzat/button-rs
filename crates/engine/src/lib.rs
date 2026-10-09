//! Button Men rules engine. Rule references (`R x.y`) point to `docs/RULES.md`.

pub mod action;
mod apply;
mod attack;
pub mod die;
pub mod error;
pub mod event;
mod fire;
mod hooks;
pub mod ids;
mod initiative;
mod resolve;
pub mod rng;
mod round;
mod scoring;
mod setup;
pub mod skill;
pub mod state;
#[cfg(feature = "testing")]
pub mod testing;

pub use action::{Action, AttackKind, DeclaredAttack, SpecChoice};
pub use apply::apply;
pub use attack::check_attack;
pub use die::{Die, DieShape, DieSpec, Sides, SwingLetter, TwinPart};
pub use error::{AttackError, FireError, ReactionError, RuleError};
pub use event::GameEvent;
pub use ids::{DieId, PlayerId};
pub use rng::SeededRng;
pub use scoring::round_score;
pub use setup::new_game;
pub use skill::{DieSkill, SkillInfo, SkillSet, skill_info};
pub use state::{
    Button, CaptureScoring, CapturedDie, DieSlot, EndReason, GameState, MatchConfig, MatchResult,
    PlayerState, Record, WaitingFor,
};
