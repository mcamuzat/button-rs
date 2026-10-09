//! Button Men rules engine. Rule references (`R x.y`) point to `docs/RULES.md`.

pub mod ids;
pub mod rng;

pub use ids::{DieId, PlayerId};
pub use rng::SeededRng;
