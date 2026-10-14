use PlayerId::{One, Two};
use buttonmen_ai::observe;
use buttonmen_engine::testing::{button, fixed};
use buttonmen_engine::{
    Action, DieShape, DieSpec, MatchConfig, PlayerId, SpecChoice, SwingLetter, WaitingFor, apply,
    new_game,
};

fn x() -> DieSpec {
    DieSpec::new(DieShape::Swing(SwingLetter::X))
}

/// Both players have an X swing die; `One` has already chosen `size`.
fn after_one_chose(size: u8) -> buttonmen_engine::GameState {
    let state = new_game(
        [
            button("A", vec![fixed(6), fixed(10), x()]),
            button("B", vec![fixed(8), fixed(12), x()]),
        ],
        3,
        MatchConfig::default(),
    );
    let choice = SpecChoice {
        swing: vec![(SwingLetter::X, size)],
        options: vec![],
    };
    apply(&state, One, Action::SpecifyDice { choice })
        .expect("valid choice")
        .0
}

#[test]
fn observe_forgets_the_opponents_pending_choice_and_asks_again() {
    let seen = observe(&after_one_chose(17), Two, 1);
    assert_eq!(seen.player(One).pending_choice, None);
    assert_eq!(
        seen.waiting_for,
        WaitingFor::SpecifyDice {
            pending: vec![Two, One]
        }
    );
}

#[test]
fn observe_keeps_my_own_pending_choice() {
    let seen = observe(&after_one_chose(17), One, 1);
    assert!(seen.player(One).pending_choice.is_some());
}

#[test]
fn observe_hides_future_rolls() {
    let state = after_one_chose(17);
    assert_ne!(observe(&state, Two, 1).rng, state.rng);
    assert_eq!(observe(&state, Two, 1).rng, observe(&state, Two, 1).rng);
}

use buttonmen_ai::{Level, choose_action};
use buttonmen_engine::testing::{attack_state, die, id};
use buttonmen_engine::{DieSkill, GameState, SeededRng};

const LEVELS: [Level; 3] = [Level::Easy, Level::Normal, Level::Hard];

#[test]
fn every_level_takes_the_obvious_big_capture() {
    let s = attack_state(
        vec![die(20, 20), die(4, 4)],
        vec![die(4, 1), die(20, 10)],
        One,
    );
    for level in LEVELS {
        let Some(Action::Attack { attack }) = choose_action(&s, One, level, 3) else {
            panic!("{level:?}")
        };
        assert_eq!(attack.targets, vec![id(&s, Two, 1)], "{level:?}");
    }
}

#[test]
fn nothing_to_do_means_none() {
    let s = attack_state(vec![die(20, 20)], vec![die(4, 1)], One);
    assert_eq!(choose_action(&s, Two, Level::Hard, 1), None);
}

#[test]
fn does_not_read_the_opponents_hidden_swing() {
    // Review Focus 1: the human (One) already chose; the AI (Two) must decide the same way.
    for level in LEVELS {
        let small = choose_action(&after_one_chose(4), Two, level, 11);
        let large = choose_action(&after_one_chose(20), Two, level, 11);
        assert_eq!(small, large, "{level:?}");
        assert!(small.is_some());
    }
}

#[test]
fn does_not_read_future_rolls() {
    let mut a = attack_state(
        vec![die(6, 4).with(DieSkill::Trip), die(10, 5)],
        vec![die(6, 3), die(8, 6)],
        One,
    );
    let mut b = a.clone();
    a.rng = SeededRng::new(1);
    b.rng = SeededRng::new(99);
    for level in LEVELS {
        assert_eq!(
            choose_action(&a, One, level, 5),
            choose_action(&b, One, level, 5),
            "{level:?}"
        );
    }
}

fn pool() -> Vec<Vec<DieSpec>> {
    use DieSkill::*;
    vec![
        vec![fixed(4), fixed(4), fixed(10), fixed(12), x()],
        vec![
            fixed(6).with(Poison),
            fixed(8).with(Shadow),
            fixed(10),
            fixed(12).with(Speed),
            x(),
        ],
        vec![
            fixed(4).with(Trip),
            fixed(6).with(Berserk),
            fixed(8).with(Focus),
            fixed(10).with(Chance),
            DieSpec::new(DieShape::Option(6, 12)),
        ],
        // Review Focus 2: Fire and Konstant on the same button.
        vec![
            fixed(6).with(Konstant),
            fixed(4),
            fixed(10).with(Fire),
            fixed(8).with(Trip),
            fixed(20).with(Konstant),
        ],
        vec![
            fixed(4).with(Stealth),
            fixed(6).with(Value),
            fixed(10).with(Fire),
            fixed(12),
            fixed(20).with(Reserve),
        ],
    ]
}

/// Plays a whole match, each seat driven by `choose_action` at its level; returns the winner.
fn play(a: usize, b: usize, levels: [Level; 2], seed: u64) -> PlayerId {
    let pool = pool();
    let mut state: GameState = new_game(
        [button("A", pool[a].clone()), button("B", pool[b].clone())],
        seed,
        MatchConfig::default(),
    );
    for step in 0..3000u64 {
        if let WaitingFor::GameOver { result } = state.waiting_for {
            return result.winner;
        }
        let actor = state.actors()[0];
        let level = levels[actor.index()];
        let action = choose_action(&state, actor, level, seed.wrapping_add(step))
            .expect("an actor has an action");
        state = apply(&state, actor, action)
            .unwrap_or_else(|e| panic!("AI played an illegal action: {e}"))
            .0;
    }
    panic!("match {a} vs {b} (seed {seed}) did not end");
}

#[test]
fn ai_matches_always_end() {
    for a in 0..5 {
        for b in 0..5 {
            play(a, b, [Level::Easy, Level::Easy], (a * 5 + b) as u64);
        }
    }
    play(3, 4, [Level::Normal, Level::Normal], 1);
    play(3, 1, [Level::Hard, Level::Normal], 2);
}

#[test]
fn completes_a_fire_attack_it_started() {
    // Found by ai_matches_always_end: the AI declared a Fire-assisted attack, then
    // cancelled it, then declared it again, forever.
    let s = attack_state(
        vec![die(4, 1), die(10, 2).with(DieSkill::Fire), die(20, 20)],
        vec![die(4, 2), die(20, 20)],
        One,
    );
    let attack = buttonmen_engine::DeclaredAttack {
        kind: buttonmen_engine::AttackKind::Power,
        attackers: vec![id(&s, One, 0)],
        targets: vec![id(&s, Two, 0)],
    };
    let (s, _) = apply(&s, One, Action::Attack { attack }).unwrap();
    for level in LEVELS {
        for seed in 0..5 {
            let action = choose_action(&s, One, level, seed).unwrap();
            assert!(
                matches!(action, Action::FireTurnDown { .. }),
                "{level:?}/{seed}: {action:?}"
            );
        }
    }
}

#[test]
fn ai_matches_with_family_a_skills_always_end() {
    use buttonmen_ai::{Level, choose_action};
    use buttonmen_engine::DieSkill::*;
    let a = button(
        "A",
        vec![
            fixed(4).with(Boom),
            fixed(6).with(Rush),
            fixed(8).with(Rage),
            fixed(10).with(Ornery),
            fixed(12).with(TimeAndSpace),
        ],
    );
    let b = button(
        "B",
        vec![
            fixed(4).with(Slow),
            fixed(6).with(Mighty),
            fixed(8).with(Rush),
            fixed(10),
            fixed(20).with(Boom),
        ],
    );
    for seed in 0..6u64 {
        let mut state = new_game([a.clone(), b.clone()], seed, MatchConfig::default());
        let mut steps = 0;
        while !matches!(state.waiting_for, WaitingFor::GameOver { .. }) {
            let actor = state.actors()[0];
            let action =
                choose_action(&state, actor, Level::Normal, seed + steps).expect("an action");
            state = apply(&state, actor, action).expect("legal").0;
            steps += 1;
            assert!(steps < 3000, "no end");
        }
    }
}

#[test]
fn attack_gain_never_values_a_boom_as_a_capture() {
    use buttonmen_ai::attack_gain;
    use buttonmen_engine::testing::{attack_state, die, id};
    use buttonmen_engine::{AttackKind, DeclaredAttack, DieSkill};
    let s = attack_state(
        vec![die(8, 2).with(DieSkill::Boom), die(4, 1)],
        vec![die(20, 20)],
        One,
    );
    let boom = DeclaredAttack {
        kind: AttackKind::Boom,
        attackers: vec![id(&s, One, 0)],
        targets: vec![id(&s, Two, 0)],
    };
    assert!(attack_gain(&s, One, &boom) < 0.0);
}
