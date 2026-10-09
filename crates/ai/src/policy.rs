use buttonmen_engine::{
    Action, AttackKind, DeclaredAttack, GameState, LegalActions, PlayerId, SeededRng, SpecChoice,
    WaitingFor, apply, candidate_actions, legal_actions,
};

use crate::eval::{capture_gain, evaluate};
use crate::observe::reseeded;

/// Bounds a rollout even in a round that would never end.
const MAX_ROLLOUT_STEPS: usize = 400;

/// What `attack` is expected to swing towards `player` (spec §7, greedy move ordering).
/// A Trip only captures sometimes: half its gain.
pub fn attack_gain(state: &GameState, player: PlayerId, attack: &DeclaredAttack) -> f64 {
    let defender = state.player(player.other());
    let gain: f64 = attack
        .targets
        .iter()
        .filter_map(|&id| defender.die(id))
        .map(capture_gain)
        .sum();
    if attack.kind == AttackKind::Trip {
        gain / 2.0
    } else {
        gain
    }
}

/// A cheap move (spec §7, "politique gloutonne"): the attack with the best immediate
/// gain; a random Swing/Option choice; the first reserve die; let the opponent start.
/// `None` when `player` has nothing to do.
pub fn quick_action(state: &GameState, player: PlayerId, rng: &mut SeededRng) -> Option<Action> {
    match legal_actions(state, player) {
        LegalActions::Wait | LegalActions::GameOver => None,
        LegalActions::SpecifyDice { swing, options } => Some(Action::SpecifyDice {
            choice: SpecChoice {
                swing: swing
                    .iter()
                    .map(|r| {
                        let span = u64::from(r.max - r.min) + 1;
                        (r.letter, r.min + (rng.next_u64() % span) as u8)
                    })
                    .collect(),
                options: options
                    .iter()
                    .map(|o| (o.die, if rng.coin() { o.a } else { o.b }))
                    .collect(),
            },
        }),
        LegalActions::ChooseReserve { dice } => Some(Action::ChooseReserve {
            die: dice.first().copied(),
        }),
        LegalActions::ReactToInitiative { .. } => Some(Action::PassInitiative),
        LegalActions::Attack { attacks, can_pass } => {
            if can_pass {
                return Some(Action::Pass);
            }
            // Prefer attacks that need no Fire; a Fire-only position is completed below.
            let best = |needs_fire: bool| {
                attacks
                    .iter()
                    .filter(|o| o.needs_fire == needs_fire)
                    .max_by(|a, b| {
                        attack_gain(state, player, &a.attack)
                            .total_cmp(&attack_gain(state, player, &b.attack))
                    })
                    .map(|o| Action::Attack {
                        attack: o.attack.clone(),
                    })
            };
            best(false).or_else(|| best(true))
        }
        LegalActions::AdjustFire { .. } => {
            let candidates = candidate_actions(state, player);
            candidates
                .iter()
                .find(|a| matches!(a, Action::FireTurnDown { .. }))
                .or_else(|| candidates.first())
                .cloned()
        }
    }
}

/// Plays `state` forward with `quick_action` until the round changes or the match ends,
/// then evaluates it for `me` (spec §7: Swing sizes are judged on simulated rounds).
pub fn rollout(state: &GameState, me: PlayerId, seed: u64) -> f64 {
    let mut rng = SeededRng::new(seed);
    let mut current = reseeded(state, seed ^ 0x5EED);
    let start = current.round;
    for _ in 0..MAX_ROLLOUT_STEPS {
        if matches!(current.waiting_for, WaitingFor::GameOver { .. }) || current.round != start {
            break;
        }
        let Some(&actor) = current.actors().first() else {
            break;
        };
        let Some(action) = quick_action(&current, actor, &mut rng) else {
            break;
        };
        match apply(&current, actor, action) {
            Ok((next, _)) => current = next,
            Err(_) => break,
        }
    }
    evaluate(&current, me)
}

#[cfg(test)]
mod tests {
    use super::*;
    use PlayerId::{One, Two};
    use buttonmen_engine::testing::{attack_state, button, die, fixed, id};
    use buttonmen_engine::{DieShape, DieSkill, DieSpec, MatchConfig, SwingLetter, new_game};

    #[test]
    fn quick_action_takes_the_biggest_capture() {
        let s = attack_state(vec![die(20, 20)], vec![die(4, 1), die(20, 10)], One);
        let action = quick_action(&s, One, &mut SeededRng::new(1)).unwrap();
        let Action::Attack { attack } = action else {
            panic!("{action:?}")
        };
        assert_eq!(attack.targets, vec![id(&s, Two, 1)]);
    }

    #[test]
    fn quick_action_passes_when_stuck_and_waits_otherwise() {
        let s = attack_state(vec![die(6, 1)], vec![die(6, 6)], One);
        assert_eq!(
            quick_action(&s, One, &mut SeededRng::new(1)),
            Some(Action::Pass)
        );
        assert_eq!(quick_action(&s, Two, &mut SeededRng::new(1)), None);
    }

    #[test]
    fn quick_action_completes_a_fire_attack() {
        let s = attack_state(
            vec![die(6, 3), die(10, 8).with(DieSkill::Fire)],
            vec![die(6, 5), die(20, 20)],
            One,
        );
        let mut rng = SeededRng::new(1);
        let declare = quick_action(&s, One, &mut rng).unwrap();
        let (s, _) = apply(&s, One, declare).unwrap();
        let finish = quick_action(&s, One, &mut rng).unwrap();
        assert!(matches!(finish, Action::FireTurnDown { .. }), "{finish:?}");
        assert!(apply(&s, One, finish).is_ok());
    }

    #[test]
    fn rollout_plays_the_round_to_its_end() {
        let x = DieSpec::new(DieShape::Swing(SwingLetter::X));
        let s = new_game(
            [
                button("A", vec![fixed(6), fixed(10), x.clone()]),
                button("B", vec![fixed(8), fixed(12), x]),
            ],
            9,
            MatchConfig::default(),
        );
        // From the very first decision, a rollout ends with one round decided: its value
        // includes a full round (±1000) unless the round was drawn.
        let value = rollout(&s, One, 4);
        assert!(value.abs() >= 500.0 || value.abs() < 100.0, "{value}");
        assert_eq!(rollout(&s, One, 4), value, "deterministic for a seed");
    }
}
