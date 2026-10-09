mod common;

use PlayerId::{One, Two};
use buttonmen_engine::testing::{attack_state, button, die, fixed, id};
use buttonmen_engine::{
    Action, AttackError, AttackKind, CaptureScoring, DeclaredAttack, DieShape, DieSkill, DieSpec,
    EndReason, GameEvent, GameState, MatchConfig, MatchResult, PlayerId, RuleError, SeededRng,
    Sides, SpecChoice, SwingLetter, WaitingFor, new_game,
};
use common::{err, ok};

fn attack(state: &GameState, kind: AttackKind, attackers: &[usize], targets: &[usize]) -> Action {
    Action::Attack {
        attack: DeclaredAttack {
            kind,
            attackers: attackers.iter().map(|&i| id(state, One, i)).collect(),
            targets: targets.iter().map(|&i| id(state, Two, i)).collect(),
        },
    }
}

#[test]
fn power_attack_captures_rerolls_and_passes_the_turn() {
    let s = attack_state(vec![die(6, 4)], vec![die(6, 3), die(20, 20)], One);
    let (after, events) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
    assert_eq!(after.player(Two).active.len(), 1);
    assert_eq!(after.player(One).captured.len(), 1);
    assert_eq!(
        after.player(One).captured[0].scoring,
        CaptureScoring::BySize
    );
    assert!((1..=6).contains(&after.player(One).active[0].value));
    assert_eq!(after.waiting_for, WaitingFor::Attack { player: Two });
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::DieCaptured { by: One, .. }))
    );
}

#[test]
fn illegal_or_out_of_turn_attacks_are_refused() {
    let s = attack_state(vec![die(6, 2)], vec![die(6, 3)], One);
    assert_eq!(
        err(&s, One, attack(&s, AttackKind::Power, &[0], &[0])),
        RuleError::IllegalAttack {
            reason: AttackError::ValuesDoNotMatch
        }
    );
    assert_eq!(err(&s, Two, Action::Pass), RuleError::NotYourTurn);
}

#[test]
fn konstant_attackers_keep_their_value() {
    let s = attack_state(
        vec![die(6, 5), die(4, 2).with(DieSkill::Konstant)],
        vec![die(6, 3), die(20, 20)],
        One,
    );
    let (after, _) = ok(&s, One, attack(&s, AttackKind::Skill, &[0, 1], &[0]));
    assert_eq!(after.player(One).active[1].value, 2);
}

#[test]
fn berserk_halves_and_loses_berserk() {
    let s = attack_state(
        vec![die(10, 7).with(DieSkill::Berserk)],
        vec![die(6, 3), die(6, 4), die(20, 20)],
        One,
    );
    let (after, _) = ok(&s, One, attack(&s, AttackKind::Berserk, &[0], &[0, 1]));
    let b = &after.player(One).active[0];
    assert_eq!(b.sides, Sides::Single(5));
    assert!(!b.has(DieSkill::Berserk));
    assert!((1..=5).contains(&b.value));
}

#[test]
fn trip_rerolls_both_and_captures_on_greater_or_equal() {
    let mut captures = 0;
    let mut misses = 0;
    for seed in 0..40 {
        let mut s = attack_state(
            vec![die(6, 1).with(DieSkill::Trip), die(20, 20)],
            vec![die(6, 6), die(20, 20)],
            One,
        );
        s.rng = SeededRng::new(seed);
        let (after, events) = ok(&s, One, attack(&s, AttackKind::Trip, &[0], &[0]));
        let rolled = events.iter().find_map(|e| match e {
            GameEvent::TripRolled {
                attacker_value,
                target_value,
                captured,
                ..
            } => Some((*attacker_value, *target_value, *captured)),
            _ => None,
        });
        let (a, t, captured) = rolled.expect("a trip event");
        assert_eq!(captured, a >= t);
        if captured {
            captures += 1;
            assert_eq!(after.player(Two).active.len(), 1);
        } else {
            misses += 1;
            assert_eq!(after.player(Two).active[0].value, t);
        }
        assert_eq!(after.player(One).active[0].value, a);
    }
    assert!(captures > 0 && misses > 0);
}

#[test]
fn null_and_value_attackers_change_capture_scoring() {
    let s = attack_state(
        vec![die(6, 4).with(DieSkill::Null)],
        vec![die(6, 3), die(20, 20)],
        One,
    );
    let (after, _) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
    assert_eq!(after.player(One).captured[0].scoring, CaptureScoring::Zero);

    let s = attack_state(
        vec![die(6, 4).with(DieSkill::Value)],
        vec![die(20, 3), die(20, 20)],
        One,
    );
    let (after, _) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
    assert_eq!(
        after.player(One).captured[0].scoring,
        CaptureScoring::ByValue(3)
    );
}

#[test]
fn pass_is_refused_when_an_attack_exists() {
    let s = attack_state(vec![die(6, 4)], vec![die(6, 3)], One);
    assert_eq!(err(&s, One, Action::Pass), RuleError::PassNotAllowed);
}

#[test]
fn round_ends_when_a_player_runs_out_of_dice() {
    let s = attack_state(vec![die(6, 4), die(10, 1)], vec![die(6, 3)], One);
    let (after, events) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
    // One: kept 6 and 10 (3 + 5 points) + captured 6 (6 points) = 14 points = 28 half-points.
    assert!(events.contains(&GameEvent::RoundEnded {
        round: 1,
        scores: [28, 0],
        winner: Some(One)
    }));
    assert_eq!(after.round, 2);
    assert_eq!(after.player(One).record.wins, 1);
    assert_eq!(after.player(Two).record.losses, 1);
    assert_eq!(after.last_round_loser, Some(Two));
    assert_eq!(
        after.player(Two).active.len(),
        1,
        "a new round rebuilt Two's dice"
    );
}

#[test]
fn two_consecutive_passes_end_round() {
    // Review Focus 5. Stealth dice can neither attack alone nor be captured by one die.
    let stealth_x = DieSpec::new(DieShape::Swing(SwingLetter::X)).with(DieSkill::Stealth);
    let state = new_game(
        [
            button("A", vec![fixed(6).with(DieSkill::Stealth)]),
            button("B", vec![stealth_x]),
        ],
        2,
        MatchConfig::default(),
    );
    let x4 = Action::SpecifyDice {
        choice: SpecChoice {
            swing: vec![(SwingLetter::X, 4)],
            options: vec![],
        },
    };
    let (state, _) = ok(&state, Two, x4.clone());
    let first = state.initiative.expect("initiative decided");
    let (state, _) = ok(&state, first, Action::Pass);
    assert_eq!(
        state.waiting_for,
        WaitingFor::Attack {
            player: first.other()
        }
    );
    let (state, events) = ok(&state, first.other(), Action::Pass);
    // Kept dice score half their size: 6 → 3 points, 4 → 2 points.
    assert!(events.contains(&GameEvent::RoundEnded {
        round: 1,
        scores: [6, 4],
        winner: Some(One)
    }));
    // R 2.3: the loser may change their swing dice.
    assert_eq!(
        state.waiting_for,
        WaitingFor::SpecifyDice { pending: vec![Two] }
    );
    assert_eq!(state.round, 2);
    let (state, _) = ok(&state, Two, x4);
    assert!(matches!(state.waiting_for, WaitingFor::Attack { .. }));
}

#[test]
fn the_match_ends_at_max_wins() {
    let mut s = attack_state(vec![die(6, 4)], vec![die(6, 3)], One);
    s.config.max_wins = 1;
    let (after, events) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
    let result = MatchResult {
        winner: One,
        reason: EndReason::Rounds,
    };
    assert_eq!(after.waiting_for, WaitingFor::GameOver { result });
    assert!(events.contains(&GameEvent::MatchEnded { result }));
}
