use AttackKind::{Berserk, Power, Shadow, Skill, Speed, Trip};
use PlayerId::{One, Two};
use buttonmen_engine::testing::{Placed, attack_state, die, id, twin};
use buttonmen_engine::{
    AttackError, AttackKind, DeclaredAttack, DieSkill, GameState, PlayerId, check_attack,
};

/// One's dice `mine` against Two's dice `theirs`; One to move.
fn table(mine: Vec<Placed>, theirs: Vec<Placed>) -> GameState {
    attack_state(mine, theirs, One)
}

/// Checks an attack by One, with attackers and targets given by index.
fn check(
    state: &GameState,
    kind: AttackKind,
    attackers: &[usize],
    targets: &[usize],
) -> Result<(), AttackError> {
    let attack = DeclaredAttack {
        kind,
        attackers: attackers.iter().map(|&i| id(state, One, i)).collect(),
        targets: targets.iter().map(|&i| id(state, Two, i)).collect(),
    };
    check_attack(state, One, &attack)
}

#[test]
fn power_captures_an_equal_or_lower_die() {
    let s = table(vec![die(6, 4)], vec![die(6, 4), die(6, 5)]);
    assert_eq!(check(&s, Power, &[0], &[0]), Ok(()));
    assert_eq!(
        check(&s, Power, &[0], &[1]),
        Err(AttackError::ValuesDoNotMatch)
    );
    assert_eq!(
        check(&s, Power, &[0], &[0, 1]),
        Err(AttackError::WrongDiceCount)
    );
}

#[test]
fn skill_needs_an_exact_sum() {
    let s = table(
        vec![die(6, 2), die(6, 3)],
        vec![die(10, 5), die(10, 6), die(10, 2)],
    );
    assert_eq!(check(&s, Skill, &[0, 1], &[0]), Ok(()));
    assert_eq!(
        check(&s, Skill, &[0, 1], &[1]),
        Err(AttackError::ValuesDoNotMatch)
    );
    assert_eq!(check(&s, Skill, &[0], &[2]), Ok(()));
}

#[test]
fn dice_must_belong_to_the_right_player_and_appear_once() {
    let s = table(vec![die(6, 4)], vec![die(6, 3)]);
    let theirs = id(&s, Two, 0);
    let mine = id(&s, One, 0);
    let wrong_side = DeclaredAttack {
        kind: Power,
        attackers: vec![theirs],
        targets: vec![theirs],
    };
    assert_eq!(
        check_attack(&s, One, &wrong_side),
        Err(AttackError::DuplicateDie { die: theirs })
    );
    let own_target = DeclaredAttack {
        kind: Power,
        attackers: vec![mine],
        targets: vec![mine],
    };
    assert_eq!(
        check_attack(&s, One, &own_target),
        Err(AttackError::DuplicateDie { die: mine })
    );
    let not_mine = DeclaredAttack {
        kind: Skill,
        attackers: vec![theirs],
        targets: vec![],
    };
    assert_eq!(
        check_attack(&s, One, &not_mine),
        Err(AttackError::NotYourDie { die: theirs })
    );
}

#[test]
fn shadow_captures_between_its_value_and_its_size() {
    let s = table(
        vec![die(10, 2).with(DieSkill::Shadow)],
        vec![die(4, 2), die(12, 10), die(12, 11), die(4, 1)],
    );
    assert_eq!(check(&s, Shadow, &[0], &[0]), Ok(()));
    assert_eq!(check(&s, Shadow, &[0], &[1]), Ok(()));
    assert_eq!(
        check(&s, Shadow, &[0], &[2]),
        Err(AttackError::ValuesDoNotMatch)
    );
    assert_eq!(
        check(&s, Shadow, &[0], &[3]),
        Err(AttackError::ValuesDoNotMatch)
    );
    let shadow = id(&s, One, 0);
    assert_eq!(
        check(&s, Power, &[0], &[3]),
        Err(AttackError::KindUnavailable {
            die: shadow,
            kind: Power
        })
    );
}

#[test]
fn queer_is_shadow_on_odd_values_only() {
    let odd = table(
        vec![die(10, 3).with(DieSkill::Queer)],
        vec![die(10, 5), die(10, 1)],
    );
    assert_eq!(check(&odd, Shadow, &[0], &[0]), Ok(()));
    assert!(matches!(
        check(&odd, Power, &[0], &[1]),
        Err(AttackError::KindUnavailable { .. })
    ));
    let even = table(
        vec![die(10, 4).with(DieSkill::Queer)],
        vec![die(10, 5), die(10, 1)],
    );
    assert_eq!(check(&even, Power, &[0], &[1]), Ok(()));
    assert!(matches!(
        check(&even, Shadow, &[0], &[0]),
        Err(AttackError::KindUnavailable { .. })
    ));
}

#[test]
fn speed_captures_several_dice_summing_to_its_value() {
    let s = table(
        vec![die(10, 7).with(DieSkill::Speed), die(10, 7)],
        vec![die(6, 3), die(6, 4), die(6, 3)],
    );
    assert_eq!(check(&s, Speed, &[0], &[0, 1]), Ok(()));
    assert_eq!(
        check(&s, Speed, &[0], &[0, 2]),
        Err(AttackError::ValuesDoNotMatch)
    );
    assert!(matches!(
        check(&s, Speed, &[1], &[0, 1]),
        Err(AttackError::KindUnavailable { .. })
    ));
    // A Speed die can still make ordinary attacks.
    assert_eq!(check(&s, Power, &[0], &[0]), Ok(()));
}

#[test]
fn berserk_attacks_like_speed_but_never_skill() {
    let s = table(
        vec![die(10, 7).with(DieSkill::Berserk)],
        vec![die(6, 3), die(6, 4), die(10, 7)],
    );
    assert_eq!(check(&s, Berserk, &[0], &[0, 1]), Ok(()));
    assert!(matches!(
        check(&s, Skill, &[0], &[2]),
        Err(AttackError::KindUnavailable { .. })
    ));
}

#[test]
fn trip_is_illegal_only_without_any_chance() {
    let s = table(
        vec![
            die(4, 1).with(DieSkill::Trip),
            die(1, 1).with(DieSkill::Trip),
        ],
        vec![die(20, 20), twin(1, 1, 2)],
    );
    assert_eq!(check(&s, Trip, &[0], &[0]), Ok(()));
    assert_eq!(check(&s, Trip, &[0], &[1]), Ok(()));
    // R 6.14: a 1-sided Trip die can never beat a twin (minimum 2).
    assert_eq!(
        check(&s, Trip, &[1], &[1]),
        Err(AttackError::ValuesDoNotMatch)
    );
}

#[test]
fn stealth_only_fights_in_multi_die_skill_attacks() {
    let s = table(
        vec![die(6, 3).with(DieSkill::Stealth), die(6, 2), die(10, 5)],
        vec![die(6, 5).with(DieSkill::Stealth), die(6, 3)],
    );
    assert!(matches!(
        check(&s, Skill, &[0], &[1]),
        Err(AttackError::KindUnavailable { .. })
    ));
    assert!(matches!(
        check(&s, Power, &[0], &[1]),
        Err(AttackError::KindUnavailable { .. })
    ));
    let stealth_target = id(&s, Two, 0);
    assert_eq!(
        check(&s, Power, &[2], &[0]),
        Err(AttackError::TargetProtected {
            die: stealth_target
        })
    );
    assert_eq!(
        check(&s, Skill, &[2], &[0]),
        Err(AttackError::TargetProtected {
            die: stealth_target
        })
    );
    assert_eq!(check(&s, Skill, &[0, 1], &[0]), Ok(()));
}

#[test]
fn konstant_adds_or_subtracts_and_never_attacks_alone() {
    let s = table(
        vec![die(6, 5), die(4, 2).with(DieSkill::Konstant)],
        vec![die(6, 3), die(10, 7), die(4, 2)],
    );
    assert_eq!(check(&s, Skill, &[0, 1], &[0]), Ok(()));
    assert_eq!(check(&s, Skill, &[0, 1], &[1]), Ok(()));
    assert!(matches!(
        check(&s, Skill, &[1], &[2]),
        Err(AttackError::KindUnavailable { .. })
    ));
    assert!(matches!(
        check(&s, Power, &[1], &[2]),
        Err(AttackError::KindUnavailable { .. })
    ));
}

#[test]
fn stinger_counts_for_any_value_up_to_its_face() {
    // R 6.13 example: a die showing 4 and a Stinger showing 6 reach 5 through 10.
    let s = table(
        vec![die(6, 4), die(10, 6).with(DieSkill::Stinger)],
        vec![die(4, 4), die(6, 5), die(10, 10), die(12, 11)],
    );
    assert_eq!(
        check(&s, Skill, &[0, 1], &[0]),
        Err(AttackError::ValuesDoNotMatch)
    );
    assert_eq!(check(&s, Skill, &[0, 1], &[1]), Ok(()));
    assert_eq!(check(&s, Skill, &[0, 1], &[2]), Ok(()));
    assert_eq!(
        check(&s, Skill, &[0, 1], &[3]),
        Err(AttackError::ValuesDoNotMatch)
    );
}

#[test]
fn fire_dice_cannot_power_attack() {
    let s = table(vec![die(6, 6).with(DieSkill::Fire)], vec![die(6, 1)]);
    assert!(matches!(
        check(&s, Power, &[0], &[0]),
        Err(AttackError::KindUnavailable { .. })
    ));
    assert_eq!(
        check(&s, Skill, &[0], &[0]),
        Err(AttackError::ValuesDoNotMatch)
    );
}

#[test]
fn dizzy_dice_cannot_attack() {
    let mut s = table(vec![die(6, 4).with(DieSkill::Focus)], vec![die(6, 3)]);
    let focus = id(&s, One, 0);
    s.player_mut(One).die_mut(focus).unwrap().dizzy = true;
    assert_eq!(
        check(&s, Power, &[0], &[0]),
        Err(AttackError::Dizzy { die: focus })
    );
}

#[test]
fn trip_against_konstant_needs_a_real_chance() {
    // R 6.14 (BMAttackTrip::is_disabled_by_konstant): Konstant dice keep their value.
    let s = table(
        vec![
            die(4, 2).with(DieSkill::Trip),
            die(4, 1).with(DieSkill::Trip).with(DieSkill::Konstant),
        ],
        vec![
            die(10, 8).with(DieSkill::Konstant),
            die(10, 4).with(DieSkill::Konstant),
            twin(4, 4, 5),
            die(6, 6),
        ],
    );
    // Konstant target showing 8 can never be beaten by a d4.
    assert_eq!(
        check(&s, Trip, &[0], &[0]),
        Err(AttackError::ValuesDoNotMatch)
    );
    assert_eq!(check(&s, Trip, &[0], &[1]), Ok(()));
    // A Konstant Trip die stays at 1: it can never beat a twin (minimum 2)…
    assert_eq!(
        check(&s, Trip, &[1], &[2]),
        Err(AttackError::ValuesDoNotMatch)
    );
    // …but a d6 can roll a 1.
    assert_eq!(check(&s, Trip, &[1], &[3]), Ok(()));
}
