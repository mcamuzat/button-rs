mod common;

use PlayerId::{One, Two};
use buttonmen_engine::testing::{attack_state, die, id, opening_state};
use buttonmen_engine::{
    Action, AttackError, AttackKind, DeclaredAttack, DieSkill, GameEvent, GameState, PlayerId,
    ResizeCause, RuleError, SeededRng, Sides, Viewer, WaitingFor, attack_swing, capture_swing,
    check_attack, view_for,
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

#[test]
fn a_captured_rage_die_is_replaced_by_a_clone_without_rage() {
    // R 6.20.
    let s = attack_state(
        vec![die(10, 9)],
        vec![
            die(8, 4).with(DieSkill::Rage).with(DieSkill::Poison),
            die(20, 20),
        ],
        One,
    );
    let (after, events) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
    let two = after.player(Two);
    assert_eq!(two.active.len(), 2);
    let clone = &two.active[0];
    assert_ne!(clone.id, id(&s, Two, 0));
    assert_eq!(clone.sides, Sides::Single(8));
    assert!(!clone.has(DieSkill::Rage) && clone.has(DieSkill::Poison));
    assert!((1..=8).contains(&clone.value));
    assert_eq!(after.player(One).captured.len(), 1);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::RageReplaced { player: Two, .. }))
    );
}

#[test]
fn a_rage_die_that_attacks_loses_rage() {
    let s = attack_state(
        vec![die(10, 9).with(DieSkill::Rage)],
        vec![die(8, 4), die(20, 20)],
        One,
    );
    let (after, _) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
    assert!(!active(&after, One, 0).has(DieSkill::Rage));
}

#[test]
fn rage_dice_do_not_count_for_initiative() {
    let s = opening_state(
        vec![die(6, 1).with(DieSkill::Rage), die(10, 9)],
        vec![die(6, 3), die(10, 8)],
    );
    assert_eq!(s.initiative, Some(Two));
}

#[test]
fn a_failed_trip_on_a_rage_die_makes_no_clone() {
    for seed in 0..30 {
        let mut s = attack_state(
            vec![die(4, 1).with(DieSkill::Trip)],
            vec![die(20, 20).with(DieSkill::Rage), die(30, 30)],
            One,
        );
        s.rng = SeededRng::new(seed);
        let (after, events) = ok(&s, One, attack(&s, AttackKind::Trip, &[0], &[0]));
        let captured = !after.player(One).captured.is_empty();
        assert_eq!(
            events
                .iter()
                .any(|e| matches!(e, GameEvent::RageReplaced { .. })),
            captured
        );
        assert_eq!(after.player(Two).active.len(), 2);
    }
}

#[test]
fn capturing_rage_only_gains_the_captured_points() {
    let s = attack_state(vec![die(10, 9)], vec![die(8, 4).with(DieSkill::Rage)], One);
    let attacker = &s.player(One).active[0];
    let rage = &s.player(Two).active[0];
    // The owner keeps a clone worth the same 4 kept points: only the 16 captured half-points swing.
    assert_eq!(capture_swing(&[attacker], rage), 16);
}

#[test]
fn rage_clone_does_not_outlive_the_round() {
    let s = attack_state(vec![die(10, 9)], vec![die(8, 4).with(DieSkill::Rage)], One);
    // Two now holds only the clone. Force values: Two cannot attack and passes, then One
    // captures the clone, which ends the round.
    let (mut s2, _) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
    s2.player_mut(Two).active[0].value = 1;
    s2.player_mut(One).active[0].value = 10;
    let (s3, _) = ok(&s2, Two, Action::Pass);
    let clone = s3.player(Two).active[0].id;
    let capture_clone = Action::Attack {
        attack: DeclaredAttack {
            kind: AttackKind::Power,
            attackers: vec![id(&s, One, 0)],
            targets: vec![clone],
        },
    };
    let (next, events) = ok(&s3, One, capture_clone);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::RoundEnded { .. }))
    );
    // The next round starts from the recipe: only the slot die, with Rage again.
    let two = &next.player(Two).active;
    assert_eq!(two.len(), 1);
    assert_eq!(two[0].id, id(&s, Two, 0));
    assert!(two[0].has(DieSkill::Rage));
}

#[test]
fn rage_clone_has_a_notation_in_the_view() {
    let s = attack_state(
        vec![die(10, 9)],
        vec![
            die(8, 4).with(DieSkill::Rage).with(DieSkill::Poison),
            die(20, 20),
        ],
        One,
    );
    let (after, _) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
    let view = view_for(&after, Viewer::Spectator);
    assert_eq!(view.players[1].active[0].notation, "p(8)");
}

fn check(
    state: &GameState,
    kind: AttackKind,
    attackers: &[usize],
    targets: &[usize],
) -> Result<(), AttackError> {
    let Action::Attack { attack } = attack(state, kind, attackers, targets) else {
        unreachable!()
    };
    check_attack(state, One, &attack)
}

#[test]
fn a_rush_die_captures_exactly_two_dice_summing_to_its_value() {
    // R 6.21.
    let s = attack_state(
        vec![die(10, 7).with(DieSkill::Rush)],
        vec![die(6, 3), die(6, 4), die(6, 2)],
        One,
    );
    assert_eq!(check(&s, AttackKind::Rush, &[0], &[0, 1]), Ok(()));
    assert_eq!(
        check(&s, AttackKind::Rush, &[0], &[0, 2]),
        Err(AttackError::ValuesDoNotMatch)
    );
    assert_eq!(
        check(&s, AttackKind::Rush, &[0], &[0]),
        Err(AttackError::WrongDiceCount)
    );
    assert_eq!(
        check(&s, AttackKind::Rush, &[0], &[0, 1, 2]),
        Err(AttackError::WrongDiceCount)
    );
}

#[test]
fn any_die_can_rush_two_targets_when_one_of_them_is_rush() {
    let s = attack_state(
        vec![die(10, 7)],
        vec![die(6, 3).with(DieSkill::Rush), die(6, 4), die(6, 3)],
        One,
    );
    assert_eq!(check(&s, AttackKind::Rush, &[0], &[0, 1]), Ok(()));
    let plain = attack_state(vec![die(10, 7)], vec![die(6, 3), die(6, 4)], One);
    assert!(matches!(
        check(&plain, AttackKind::Rush, &[0], &[0, 1]),
        Err(AttackError::KindUnavailable { .. })
    ));
}

#[test]
fn rush_cannot_capture_stealth() {
    let s = attack_state(
        vec![die(10, 7).with(DieSkill::Rush)],
        vec![
            die(6, 3).with(DieSkill::Stealth),
            die(6, 4).with(DieSkill::Rush),
        ],
        One,
    );
    assert!(matches!(
        check(&s, AttackKind::Rush, &[0], &[0, 1]),
        Err(AttackError::TargetProtected { .. })
    ));
}

#[test]
fn stealth_dice_cannot_rush() {
    let s = attack_state(
        vec![die(10, 7).with(DieSkill::Stealth).with(DieSkill::Rush)],
        vec![die(6, 3), die(6, 4)],
        One,
    );
    assert!(matches!(
        check(&s, AttackKind::Rush, &[0], &[0, 1]),
        Err(AttackError::KindUnavailable { .. })
    ));
}

#[test]
fn a_rush_attack_captures_both_dice_and_is_listed() {
    let s = attack_state(
        vec![die(10, 7).with(DieSkill::Rush)],
        vec![die(6, 3), die(6, 4), die(20, 20)],
        One,
    );
    let listed = buttonmen_engine::candidate_actions(&s, One);
    assert!(listed.contains(&attack(&s, AttackKind::Rush, &[0], &[0, 1])));
    let (after, _) = ok(&s, One, attack(&s, AttackKind::Rush, &[0], &[0, 1]));
    assert_eq!(after.player(One).captured.len(), 2);
    assert_eq!(after.player(Two).active.len(), 1);
}

#[test]
fn a_boom_die_leaves_play_and_rerolls_its_target() {
    // R 6.22.
    let s = attack_state(
        vec![die(6, 2).with(DieSkill::Boom), die(10, 1)],
        vec![die(20, 20), die(4, 4)],
        One,
    );
    let (after, events) = ok(&s, One, attack(&s, AttackKind::Boom, &[0], &[0]));
    let one = after.player(One);
    assert_eq!(one.active.len(), 1);
    assert_eq!(one.out_of_play.len(), 1);
    assert!(one.captured.is_empty());
    assert_eq!(after.player(Two).active.len(), 2);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::BoomRerolled { .. }))
    );
    assert_eq!(after.waiting_for, WaitingFor::Attack { player: Two });
}

#[test]
fn a_boomed_die_scores_for_nobody() {
    let s = attack_state(
        vec![die(20, 2).with(DieSkill::Boom), die(10, 1)],
        vec![die(20, 20), die(4, 4)],
        One,
    );
    let before = buttonmen_engine::round_score(&s, One);
    let (after, _) = ok(&s, One, attack(&s, AttackKind::Boom, &[0], &[0]));
    assert_eq!(buttonmen_engine::round_score(&after, One), before - 20);
}

#[test]
fn boom_may_target_stealth_but_stealth_cannot_boom() {
    let s = attack_state(
        vec![die(6, 2).with(DieSkill::Boom)],
        vec![die(6, 3).with(DieSkill::Stealth)],
        One,
    );
    assert_eq!(check(&s, AttackKind::Boom, &[0], &[0]), Ok(()));
    let stealthy = attack_state(
        vec![die(6, 2).with(DieSkill::Boom).with(DieSkill::Stealth)],
        vec![die(6, 3)],
        One,
    );
    assert!(matches!(
        check(&stealthy, AttackKind::Boom, &[0], &[0]),
        Err(AttackError::KindUnavailable { .. })
    ));
}

#[test]
fn a_possible_boom_forbids_passing() {
    // Buttonweavers treats Boom as an obligatory attack: it is not optional like Warrior.
    let s = attack_state(vec![die(6, 1).with(DieSkill::Boom)], vec![die(6, 6)], One);
    assert_eq!(
        buttonmen_engine::apply(&s, One, Action::Pass).unwrap_err(),
        RuleError::PassNotAllowed
    );
}

#[test]
fn boomed_dice_come_back_next_round() {
    let s = attack_state(vec![die(6, 1).with(DieSkill::Boom)], vec![die(6, 6)], One);
    let (after, events) = ok(&s, One, attack(&s, AttackKind::Boom, &[0], &[0]));
    // One has no die left: the round ends and the next one starts from the recipe.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::RoundEnded { .. }))
    );
    assert!(after.player(One).out_of_play.is_empty());
}

#[test]
fn attack_swing_counts_captures_rage_and_boom() {
    let s = attack_state(
        vec![die(10, 9), die(8, 2).with(DieSkill::Boom)],
        vec![die(6, 4), die(8, 4).with(DieSkill::Rage)],
        One,
    );
    let swing = |kind, a: &[usize], t: &[usize]| {
        let Action::Attack { attack } = attack(&s, kind, a, t) else {
            unreachable!()
        };
        attack_swing(&s, One, &attack)
    };
    assert_eq!(swing(AttackKind::Power, &[0], &[0]), 18);
    assert_eq!(swing(AttackKind::Power, &[0], &[1]), 16);
    // The Boom die leaves One's kept dice (4 points) and captures nothing.
    assert_eq!(swing(AttackKind::Boom, &[1], &[0]), -8);
}
