use crate::action::Action;
use crate::error::RuleError;
use crate::event::GameEvent;
use crate::ids::PlayerId;
use crate::state::{GameState, WaitingFor};
use crate::{fire, initiative, resolve, round, setup};

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
        Action::FocusTurnDown { dice } => {
            initiative::focus_turn_down(&mut next, player, dice, &mut events)?
        }
        Action::ChanceReroll { die } => {
            initiative::chance_reroll(&mut next, player, die, &mut events)?
        }
        Action::PassInitiative => initiative::pass_initiative(&mut next, player, &mut events)?,
        Action::Attack { attack } => {
            resolve::declare_attack(&mut next, player, attack, &mut events)?
        }
        Action::Pass => round::pass(&mut next, player, &mut events)?,
        Action::FireTurnDown {
            fire: amounts,
            boosts,
        } => fire::fire_turn_down(&mut next, player, amounts, boosts, &mut events)?,
        Action::CancelFire => fire::cancel_fire(&mut next, player)?,
        Action::ChooseReserve { die } => {
            setup::choose_reserve(&mut next, player, die, &mut events)?
        }
    }
    Ok((next, events))
}
