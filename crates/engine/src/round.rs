use crate::attack;
use crate::error::RuleError;
use crate::event::GameEvent;
use crate::ids::PlayerId;
use crate::scoring;
use crate::setup;
use crate::state::{EndReason, GameState, MatchResult, WaitingFor};
use std::cmp::Ordering;

/// R 2.4: the opponent wins the match.
pub(crate) fn surrender(state: &mut GameState, player: PlayerId, events: &mut Vec<GameEvent>) {
    let result = MatchResult {
        winner: player.other(),
        reason: EndReason::Surrender,
    };
    events.push(GameEvent::MatchEnded { result });
    state.waiting_for = WaitingFor::GameOver { result };
}

/// After an attack or a pass: R 7.4 (dizziness lasts one turn), then R 4.5.
pub(crate) fn end_turn(state: &mut GameState, player: PlayerId, events: &mut Vec<GameEvent>) {
    for p in PlayerId::ALL {
        for die in &mut state.player_mut(p).active {
            die.dizzy = false;
        }
    }
    let out_of_dice = PlayerId::ALL
        .into_iter()
        .any(|p| state.player(p).active.is_empty());
    if out_of_dice || state.consecutive_passes >= 2 {
        end_round(state, events);
    } else {
        state.waiting_for = WaitingFor::Attack {
            player: player.other(),
        };
    }
}

/// R 4.1: pass only without any possible attack.
pub(crate) fn pass(
    state: &mut GameState,
    player: PlayerId,
    events: &mut Vec<GameEvent>,
) -> Result<(), RuleError> {
    match state.waiting_for {
        WaitingFor::Attack { player: p } if p == player => {}
        WaitingFor::Attack { .. } => return Err(RuleError::NotYourTurn),
        _ => return Err(RuleError::WrongPhase),
    }
    // R 4.1 + R 6.3: an attack that Fire makes possible also forbids passing.
    if !attack::possible_attacks(state, player).is_empty()
        || !crate::fire::assisted_attacks(state, player).is_empty()
    {
        return Err(RuleError::PassNotAllowed);
    }
    state.consecutive_passes += 1;
    events.push(GameEvent::Passed { player });
    end_turn(state, player, events);
    Ok(())
}

/// R 5.2 + R 2.2 + R 2.3.
fn end_round(state: &mut GameState, events: &mut Vec<GameEvent>) {
    let scores = [
        scoring::round_score(state, PlayerId::One),
        scoring::round_score(state, PlayerId::Two),
    ];
    let winner = match scores[0].cmp(&scores[1]) {
        Ordering::Greater => Some(PlayerId::One),
        Ordering::Less => Some(PlayerId::Two),
        Ordering::Equal => None,
    };
    events.push(GameEvent::RoundEnded {
        round: state.round,
        scores,
        winner,
    });
    match winner {
        Some(w) => {
            state.player_mut(w).record.wins += 1;
            state.player_mut(w.other()).record.losses += 1;
            state.last_round_loser = Some(w.other());
        }
        None => {
            for p in PlayerId::ALL {
                state.player_mut(p).record.draws += 1;
            }
            state.last_round_loser = None;
        }
    }
    let max_wins = state.config.max_wins;
    if let Some(champion) = PlayerId::ALL
        .into_iter()
        .find(|&p| state.player(p).record.wins >= max_wins)
    {
        let result = MatchResult {
            winner: champion,
            reason: EndReason::Rounds,
        };
        events.push(GameEvent::MatchEnded { result });
        state.waiting_for = WaitingFor::GameOver { result };
        return;
    }
    state.round = state.round.saturating_add(1);
    setup::begin_round(state, events);
}
