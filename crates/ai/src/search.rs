use buttonmen_engine::{
    Action, GameState, LegalActions, PlayerId, SeededRng, WaitingFor, apply, candidate_actions,
    legal_actions,
};
use serde::{Deserialize, Serialize};

use crate::eval::evaluate;
use crate::observe::{observe, reseeded};
use crate::policy::{attack_gain, rollout};

/// Strength of the AI (spec §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum Level {
    /// Greedy: one layer, one sample; random choices before the round.
    Easy,
    /// Depth 1 with chance: several samples of the rolls; rollouts before the round.
    Normal,
    /// Depth 2: the opponent's best reply is considered; more samples and rollouts.
    Hard,
}

struct Params {
    /// Samples of the rerolls per candidate (chance nodes).
    samples: u32,
    /// Simulated rounds per candidate before the round starts (Swing, Reserve, initiative).
    rollouts: u32,
    /// 1: my move; 2: my move and the opponent's best reply.
    depth: u8,
    /// Opponent replies examined at depth 2, best immediate gain first.
    replies: usize,
}

impl Level {
    fn params(self) -> Params {
        match self {
            Level::Easy => Params {
                samples: 1,
                rollouts: 0,
                depth: 1,
                replies: 0,
            },
            Level::Normal => Params {
                samples: 4,
                rollouts: 6,
                depth: 1,
                replies: 0,
            },
            Level::Hard => Params {
                samples: 16,
                rollouts: 32,
                depth: 2,
                replies: 16,
            },
        }
    }
}

/// Moves searched at depth 2, chosen by a one-sample first look.
const DEEP_CANDIDATES: usize = 12;
/// Total rerolls sampled per decision (all candidates together).
const SAMPLE_BUDGET: u32 = 160;
/// Total rounds simulated per decision before the round starts.
const ROLLOUT_BUDGET: u32 = 384;
/// Never fewer samples or rollouts per candidate than this.
const MIN_RUNS: u32 = 3;

/// Picks `me`'s action (spec §7): sampled expectimax over `candidate_actions`, seen
/// through `observe` (no hidden information). Deterministic for a given `seed`.
pub fn choose_action(state: &GameState, me: PlayerId, level: Level, seed: u64) -> Option<Action> {
    let seen = observe(state, me, seed);
    let mut candidates = candidate_actions(&seen, me);
    // An attack the AI declared with Fire is completed, never cancelled: cancelling only
    // returns to the same choice, and would loop forever.
    if candidates
        .iter()
        .any(|a| matches!(a, Action::FireTurnDown { .. }))
    {
        candidates.retain(|a| *a != Action::CancelFire);
    }
    if candidates.len() <= 1 {
        return candidates.pop();
    }
    let params = level.params();
    let mut rng = SeededRng::new(seed ^ 0xA11CE);
    let before_round = matches!(
        seen.waiting_for,
        WaitingFor::SpecifyDice { .. }
            | WaitingFor::ChooseReserve { .. }
            | WaitingFor::ReactToInitiative { .. }
    );
    if before_round && params.rollouts == 0 {
        let pick = (rng.next_u64() % candidates.len() as u64) as usize;
        return Some(candidates.swap_remove(pick));
    }
    // Bounded effort (spec §7, "échantillonnées au-delà d'un seuil"): at depth 2 only the
    // most promising moves are searched, and the samples or rollouts per move shrink when
    // there are many moves, so that a decision stays fast on large buttons.
    if params.depth >= 2 && !before_round && candidates.len() > DEEP_CANDIDATES {
        let mut first_look: Vec<(Action, f64)> = candidates
            .into_iter()
            .map(|action| {
                let sample = reseeded(&seen, rng.next_u64());
                let value = apply(&sample, me, action.clone())
                    .map_or(f64::MIN / 2.0, |(next, _)| evaluate(&settle(next, me), me));
                (action, value)
            })
            .collect();
        first_look.sort_by(|a, b| b.1.total_cmp(&a.1));
        candidates = first_look
            .into_iter()
            .take(DEEP_CANDIDATES)
            .map(|(a, _)| a)
            .collect();
    }
    let (wanted, budget) = if before_round {
        (params.rollouts, ROLLOUT_BUDGET)
    } else {
        (params.samples, SAMPLE_BUDGET)
    };
    let runs = wanted.min((budget / candidates.len() as u32).max(MIN_RUNS));
    let mut best: Option<(Action, f64)> = None;
    for action in candidates {
        let mut total = 0.0;
        for _ in 0..runs {
            let sample = reseeded(&seen, rng.next_u64());
            total += match apply(&sample, me, action.clone()).map(|(next, _)| settle(next, me)) {
                Ok(next) if before_round => rollout(&next, me, rng.next_u64()),
                Ok(next) if params.depth >= 2 => reply_value(&next, me, params.replies, &mut rng),
                Ok(next) => evaluate(&next, me),
                // Candidates are legal by construction; a refusal only scores as worst.
                Err(_) => f64::MIN / 2.0,
            };
        }
        let value = total / f64::from(runs);
        if best.as_ref().is_none_or(|(_, v)| value > *v) {
            best = Some((action, value));
        }
    }
    best.map(|(action, _)| action)
}

/// A declared attack that waits for Fire is judged by its completion: the Fire
/// turn-down with the best immediate value (spec §7, Fire is an `Action` like the others).
fn settle(state: GameState, me: PlayerId) -> GameState {
    if !matches!(&state.waiting_for, WaitingFor::AdjustFire { player, .. } if *player == me) {
        return state;
    }
    candidate_actions(&state, me)
        .into_iter()
        .filter(|a| matches!(a, Action::FireTurnDown { .. }))
        .filter_map(|a| apply(&state, me, a).ok().map(|(next, _)| next))
        .max_by(|a, b| evaluate(a, me).total_cmp(&evaluate(b, me)))
        .unwrap_or(state)
}

/// Depth 2: when the opponent attacks next, `me` gets the worst value among its most
/// promising replies (best immediate gain first, plus passing when allowed).
fn reply_value(state: &GameState, me: PlayerId, replies: usize, rng: &mut SeededRng) -> f64 {
    let opponent = me.other();
    if state.waiting_for != (WaitingFor::Attack { player: opponent }) {
        return evaluate(state, me);
    }
    let LegalActions::Attack { attacks, can_pass } = legal_actions(state, opponent) else {
        return evaluate(state, me);
    };
    let mut ordered: Vec<_> = attacks
        .into_iter()
        .filter(|o| !o.needs_fire)
        .map(|o| o.attack)
        .collect();
    ordered.sort_by(|a, b| {
        attack_gain(state, opponent, b).total_cmp(&attack_gain(state, opponent, a))
    });
    let mut options: Vec<Action> = ordered
        .into_iter()
        .take(replies)
        .map(|attack| Action::Attack { attack })
        .collect();
    if can_pass {
        options.push(Action::Pass);
    }
    if options.is_empty() {
        return evaluate(state, me);
    }
    options
        .into_iter()
        .map(|reply| {
            let sample = reseeded(state, rng.next_u64());
            apply(&sample, opponent, reply)
                .map_or_else(|_| evaluate(state, me), |(next, _)| evaluate(&next, me))
        })
        .fold(f64::INFINITY, f64::min)
}
