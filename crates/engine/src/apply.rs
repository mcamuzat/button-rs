use crate::action::Action;
use crate::error::RuleError;
use crate::event::GameEvent;
use crate::ids::PlayerId;
use crate::state::{GameState, WaitingFor};
use crate::{round, setup};

/// The rules reducer: a new state and what happened, or why the action is refused.
pub fn apply(
    state: &GameState,
    player: PlayerId,
    action: Action,
) -> Result<(GameState, Vec<GameEvent>), RuleError> {
    if let WaitingFor::GameOver { .. } = state.waiting_for {
        return Err(RuleError::GameOver);
    }
    let mut next = state.clone();
    let mut events = Vec::new();
    match action {
        Action::Surrender => round::surrender(&mut next, player, &mut events),
        Action::SpecifyDice { choice } => {
            setup::specify_dice(&mut next, player, choice, &mut events)?
        }
        // Wired by later tasks: initiative reactions (6), attacks and pass (8),
        // fire (9), reserve (10).
        Action::ChooseReserve { .. }
        | Action::FocusTurnDown { .. }
        | Action::ChanceReroll { .. }
        | Action::PassInitiative
        | Action::Attack { .. }
        | Action::FireTurnDown { .. }
        | Action::CancelFire
        | Action::Pass => return Err(RuleError::WrongPhase),
    }
    Ok((next, events))
}
