//! Button Men AI (spec §7). It plays through the engine's public API and never reads
//! hidden information: see `observe`.

mod observe;

pub use observe::{observe, reseeded};
