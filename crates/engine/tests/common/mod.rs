#![allow(dead_code)]

use buttonmen_engine::{Action, GameEvent, GameState, PlayerId, RuleError, apply};

/// Applies an action that must succeed.
pub fn ok(state: &GameState, player: PlayerId, action: Action) -> (GameState, Vec<GameEvent>) {
    apply(state, player, action).unwrap_or_else(|e| panic!("action refused: {e}"))
}

/// Applies an action that must fail, and returns the error.
pub fn err(state: &GameState, player: PlayerId, action: Action) -> RuleError {
    apply(state, player, action).expect_err("action unexpectedly accepted")
}
