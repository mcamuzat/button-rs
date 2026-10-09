//! Button Men rules engine. Rule references (`R x.y`) point to `docs/RULES.md`.

pub mod die;
pub mod ids;
pub mod rng;
pub mod skill;

pub use die::{Die, DieShape, DieSpec, Sides, SwingLetter, TwinPart};
pub use ids::{DieId, PlayerId};
pub use rng::SeededRng;
pub use skill::{DieSkill, SkillInfo, SkillSet, skill_info};
