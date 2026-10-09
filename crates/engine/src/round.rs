use crate::event::GameEvent;
use crate::ids::PlayerId;
use crate::state::{EndReason, GameState, MatchResult, WaitingFor};

/// R 2.4: the opponent wins the match.
pub(crate) fn surrender(state: &mut GameState, player: PlayerId, events: &mut Vec<GameEvent>) {
    let result = MatchResult {
        winner: player.other(),
        reason: EndReason::Surrender,
    };
    events.push(GameEvent::MatchEnded { result });
    state.waiting_for = WaitingFor::GameOver { result };
}
