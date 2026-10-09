use buttonmen_engine::testing::{button, fixed};
use buttonmen_engine::{
    Action, AttackKind, DeclaredAttack, DieId, DieShape, DieSkill, DieSpec, GameState,
    LegalActions, MatchConfig, PlayerId, SeededRng, SwingLetter, TwinPart, WaitingFor, apply,
    candidate_actions, legal_actions, new_game,
};
use proptest::prelude::*;

fn swing(l: SwingLetter) -> DieSpec {
    DieSpec::new(DieShape::Swing(l))
}

/// Buttons covering every v1 skill and die shape.
fn pool() -> Vec<Vec<DieSpec>> {
    use DieSkill::*;
    vec![
        vec![
            fixed(4),
            fixed(4),
            fixed(10),
            fixed(12),
            swing(SwingLetter::X),
        ],
        vec![
            fixed(6).with(Poison),
            fixed(8).with(Shadow),
            fixed(10),
            fixed(12).with(Speed),
            swing(SwingLetter::Y),
        ],
        vec![
            fixed(4).with(Trip),
            fixed(6).with(Berserk),
            fixed(8).with(Focus),
            fixed(10).with(Chance),
            DieSpec::new(DieShape::Option(6, 12)),
        ],
        vec![
            fixed(6).with(Konstant),
            fixed(8).with(Stinger),
            fixed(10).with(Queer),
            DieSpec::new(DieShape::Twin(TwinPart::Fixed(4), TwinPart::Fixed(4))),
            fixed(20).with(Null),
        ],
        vec![
            fixed(4).with(Stealth),
            fixed(6).with(Value),
            fixed(10).with(Fire),
            fixed(12),
            fixed(20).with(Reserve),
            swing(SwingLetter::Z).with(Reserve),
        ],
        vec![
            fixed(4).with(Stealth),
            fixed(6).with(Stealth),
            fixed(8),
            DieSpec::new(DieShape::Twin(
                TwinPart::Swing(SwingLetter::X),
                TwinPart::Swing(SwingLetter::X),
            )),
        ],
    ]
}

fn total_dice(state: &GameState) -> usize {
    PlayerId::ALL
        .iter()
        .map(|&p| {
            let ps = state.player(p);
            ps.active.len() + ps.captured.len()
        })
        .sum()
}

fn in_play_slots(state: &GameState) -> usize {
    PlayerId::ALL
        .iter()
        .map(|&p| state.player(p).slots.iter().filter(|s| s.in_play).count())
        .sum()
}

/// A random attack shape: random kind, random subsets of each side's dice (in table order,
/// like the engine's own enumeration).
fn random_attack(state: &GameState, actor: PlayerId, rng: &mut SeededRng) -> DeclaredAttack {
    let kind = AttackKind::ALL[(rng.next_u64() % AttackKind::ALL.len() as u64) as usize];
    let mut pick = |p: PlayerId| -> Vec<DieId> {
        state
            .player(p)
            .active
            .iter()
            .map(|d| d.id)
            .filter(|_| rng.coin())
            .collect()
    };
    let attackers = pick(actor);
    let targets = pick(actor.other());
    DeclaredAttack {
        kind,
        attackers,
        targets,
    }
}

/// Plays a whole match with random candidate actions, checking invariants on the way.
fn play(a: usize, b: usize, seed: u64) -> Result<(), TestCaseError> {
    let pool = pool();
    let mut state = new_game(
        [button("A", pool[a].clone()), button("B", pool[b].clone())],
        seed,
        MatchConfig::default(),
    );
    let mut chooser = SeededRng::new(seed ^ 0xABCD);
    for _ in 0..20_000 {
        if let WaitingFor::GameOver { .. } = state.waiting_for {
            return Ok(());
        }
        let actor = state.actors()[0];
        if let LegalActions::Attack { attacks, .. } = legal_actions(&state, actor) {
            // Conservation: every die in play is either active or captured.
            prop_assert_eq!(total_dice(&state), in_play_slots(&state));
            // Every listed attack is accepted.
            for option in &attacks {
                let accepted = apply(
                    &state,
                    actor,
                    Action::Attack {
                        attack: option.attack.clone(),
                    },
                )
                .is_ok();
                prop_assert!(accepted, "listed attack refused: {:?}", option.attack);
            }
            // No attack outside the list is accepted.
            for _ in 0..8 {
                let attack = random_attack(&state, actor, &mut chooser);
                if apply(
                    &state,
                    actor,
                    Action::Attack {
                        attack: attack.clone(),
                    },
                )
                .is_ok()
                {
                    prop_assert!(
                        attacks.iter().any(|o| o.attack == attack),
                        "accepted but not listed: {:?}",
                        attack
                    );
                }
            }
        }
        let candidates = candidate_actions(&state, actor);
        prop_assert!(
            !candidates.is_empty(),
            "no action for {:?} in {:?}",
            actor,
            state.waiting_for
        );
        let pick = candidates[(chooser.next_u64() % candidates.len() as u64) as usize].clone();
        let (next, _) =
            apply(&state, actor, pick).map_err(|e| TestCaseError::fail(e.to_string()))?;
        state = next;
    }
    Err(TestCaseError::fail(
        "match did not end within 20 000 actions",
    ))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn random_matches_end_and_keep_invariants(a in 0usize..6, b in 0usize..6, seed in any::<u64>()) {
        play(a, b, seed)?;
    }

    #[test]
    fn surrender_is_always_accepted(a in 0usize..6, b in 0usize..6, seed in any::<u64>()) {
        let pool = pool();
        let state = new_game([button("A", pool[a].clone()), button("B", pool[b].clone())], seed, MatchConfig::default());
        prop_assert!(apply(&state, PlayerId::Two, Action::Surrender).is_ok());
    }
}
