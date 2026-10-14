mod common;

use PlayerId::{One, Two};
use buttonmen_engine::testing::{attack_state, die, id, opening_state};
use buttonmen_engine::{
    Action, AttackKind, DeclaredAttack, DieSkill, GameEvent, GameState, PlayerId, ResizeCause,
    SeededRng, Sides, WaitingFor,
};
use common::ok;

fn attack(state: &GameState, kind: AttackKind, attackers: &[usize], targets: &[usize]) -> Action {
    Action::Attack {
        attack: DeclaredAttack {
            kind,
            attackers: attackers.iter().map(|&i| id(state, One, i)).collect(),
            targets: targets.iter().map(|&i| id(state, Two, i)).collect(),
        },
    }
}

/// The die `index` of `player`'s button, wherever it is still active.
fn active(state: &GameState, player: PlayerId, index: usize) -> &buttonmen_engine::Die {
    state
        .player(player)
        .die(id(state, player, index))
        .expect("still active")
}

#[test]
fn slow_dice_do_not_count_for_initiative() {
    // R 6.16: without Slow, One's 1 would win the initiative.
    let s = opening_state(
        vec![die(6, 1).with(DieSkill::Slow), die(10, 9)],
        vec![die(6, 3), die(10, 8)],
    );
    assert_eq!(s.initiative, Some(Two));
}

#[test]
fn sides_grow_through_the_mighty_sizes() {
    // R 6.17: 1, 2, 4, 6, 8, 10, 12, 16, 20, 30; an off-list size grows to the next one.
    assert_eq!(Sides::Single(4).grown(), Sides::Single(6));
    assert_eq!(Sides::Single(12).grown(), Sides::Single(16));
    assert_eq!(Sides::Single(5).grown(), Sides::Single(6));
    assert_eq!(Sides::Single(30).grown(), Sides::Single(30));
    assert_eq!(Sides::Single(40).grown(), Sides::Single(40));
    assert_eq!(Sides::Twin(4, 6).grown(), Sides::Twin(6, 8));
}

#[test]
fn a_mighty_attacker_grows_before_rerolling() {
    let s = attack_state(
        vec![die(6, 5).with(DieSkill::Mighty)],
        vec![die(6, 3), die(20, 20)],
        One,
    );
    let (after, events) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
    let grown = active(&after, One, 0);
    assert_eq!(grown.sides, Sides::Single(8));
    assert!((1..=8).contains(&grown.value));
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::DieResized {
            from: Sides::Single(6),
            to: Sides::Single(8),
            cause: ResizeCause::Mighty,
            ..
        }
    )));
}

#[test]
fn mighty_grows_on_a_trip_target_but_not_on_the_trip_attacker() {
    // BMSkillMighty::pre_roll skips only the die that just made the Trip attack.
    let s = attack_state(
        vec![die(6, 2).with(DieSkill::Trip).with(DieSkill::Mighty)],
        vec![die(4, 4).with(DieSkill::Mighty), die(20, 20)],
        One,
    );
    let (after, _) = ok(&s, One, attack(&s, AttackKind::Trip, &[0], &[0]));
    assert_eq!(active(&after, One, 0).sides, Sides::Single(6));
    let target = after
        .player(Two)
        .die(id(&s, Two, 0))
        .map(|d| d.sides)
        .or_else(|| after.player(One).captured.first().map(|c| c.die.sides));
    assert_eq!(target, Some(Sides::Single(6)));
}

#[test]
fn a_konstant_mighty_die_neither_rerolls_nor_grows() {
    // R 6.5 + R 6.17: the die does not reroll, so Mighty does not trigger.
    let s = attack_state(
        vec![
            die(6, 3).with(DieSkill::Konstant).with(DieSkill::Mighty),
            die(4, 2),
        ],
        vec![die(10, 5), die(20, 20)],
        One,
    );
    let (after, _) = ok(&s, One, attack(&s, AttackKind::Skill, &[0, 1], &[0]));
    let k = active(&after, One, 0);
    assert_eq!((k.sides, k.value), (Sides::Single(6), 3));
}

#[test]
fn a_chance_reroll_makes_a_mighty_die_grow() {
    // Interaction documented by buttonweavers: Chance + Mighty grows.
    let mut s = opening_state(
        vec![die(6, 6).with(DieSkill::Chance).with(DieSkill::Mighty)],
        vec![die(6, 1)],
    );
    s.rng = SeededRng::new(3);
    assert_eq!(s.waiting_for, WaitingFor::ReactToInitiative { player: One });
    let (after, _) = ok(
        &s,
        One,
        Action::ChanceReroll {
            die: id(&s, One, 0),
        },
    );
    assert_eq!(active(&after, One, 0).sides, Sides::Single(8));
}

#[test]
fn ornery_dice_reroll_after_any_attack_of_their_owner() {
    // R 6.18: the Ornery d20 did not attack, yet it rerolls; it stays put on a pass.
    let mut rerolled = 0;
    for seed in 0..20 {
        let mut s = attack_state(
            vec![die(6, 5), die(20, 20).with(DieSkill::Ornery)],
            vec![die(6, 3), die(30, 30)],
            One,
        );
        s.rng = SeededRng::new(seed);
        let (after, events) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
        let ornery = id(&s, One, 1);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, GameEvent::DieRerolled { die, .. } if *die == ornery))
        );
        if active(&after, One, 1).value != 20 {
            rerolled += 1;
        }
    }
    assert!(
        rerolled > 15,
        "the Ornery die kept 20 too often: {rerolled}"
    );
}

#[test]
fn ornery_dice_do_not_reroll_when_their_owner_passes() {
    let s = attack_state(
        vec![die(6, 1), die(20, 2).with(DieSkill::Ornery)],
        vec![die(6, 6), die(20, 20)],
        One,
    );
    let (after, _) = ok(&s, One, Action::Pass);
    assert_eq!(active(&after, One, 1).value, 2);
}

#[test]
fn an_attacking_ornery_die_rerolls_once() {
    let s = attack_state(
        vec![die(6, 5).with(DieSkill::Ornery)],
        vec![die(6, 3), die(20, 20)],
        One,
    );
    let (_, events) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
    let me = id(&s, One, 0);
    let count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::DieRerolled { die, .. } if *die == me))
        .count();
    assert_eq!(count, 1);
}

#[test]
fn time_and_space_gives_an_extra_turn_on_an_odd_reroll() {
    // R 6.19: the attacker plays again exactly when its new value is odd.
    let (mut odd, mut even) = (0, 0);
    for seed in 0..40 {
        let mut s = attack_state(
            vec![die(6, 5).with(DieSkill::TimeAndSpace), die(10, 1)],
            vec![die(6, 3), die(20, 20)],
            One,
        );
        s.rng = SeededRng::new(seed);
        let (after, events) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
        let value = active(&after, One, 0).value;
        let again = value % 2 == 1;
        let next = if again { One } else { Two };
        assert_eq!(
            after.waiting_for,
            WaitingFor::Attack { player: next },
            "value {value}"
        );
        assert_eq!(
            events
                .iter()
                .any(|e| matches!(e, GameEvent::ExtraTurn { player: One })),
            again
        );
        if again { odd += 1 } else { even += 1 }
    }
    assert!(odd > 0 && even > 0);
}

#[test]
fn a_konstant_time_and_space_die_gives_no_extra_turn() {
    // Interaction: Konstant does not reroll, so TimeAndSpace never triggers.
    let s = attack_state(
        vec![
            die(6, 3)
                .with(DieSkill::Konstant)
                .with(DieSkill::TimeAndSpace),
            die(4, 2),
        ],
        vec![die(10, 5), die(20, 20)],
        One,
    );
    let (after, _) = ok(&s, One, attack(&s, AttackKind::Skill, &[0, 1], &[0]));
    assert_eq!(after.waiting_for, WaitingFor::Attack { player: Two });
}

#[test]
fn extra_turn_never_prevents_the_end_of_the_round() {
    for seed in 0..20 {
        let mut s = attack_state(
            vec![die(6, 5).with(DieSkill::TimeAndSpace)],
            vec![die(6, 3)],
            One,
        );
        s.rng = SeededRng::new(seed);
        let (_, events) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
        assert!(
            events
                .iter()
                .any(|e| matches!(e, GameEvent::RoundEnded { .. }))
        );
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, GameEvent::ExtraTurn { .. }))
        );
    }
}
