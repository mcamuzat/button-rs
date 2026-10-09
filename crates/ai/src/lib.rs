//! Button Men AI (spec §7). It plays through the engine's public API and never reads
//! hidden information: see `observe`.

mod eval;
mod observe;
mod policy;
mod search;

pub use eval::{capture_gain, evaluate};
pub use observe::{observe, reseeded};
pub use policy::{attack_gain, quick_action, rollout};
pub use search::{Level, choose_action};
