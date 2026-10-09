mod common;

use PlayerId::{One, Two};
use buttonmen_engine::testing::{attack_state, die, id};
use buttonmen_engine::{
    Action, AttackError, AttackKind, DeclaredAttack, DieSkill, FireError, GameState, PlayerId,
    RuleError, WaitingFor,
};
use common::{err, ok};

fn declared(
    state: &GameState,
    kind: AttackKind,
    attackers: &[usize],
    targets: &[usize],
) -> DeclaredAttack {
    DeclaredAttack {
        kind,
        attackers: attackers.iter().map(|&i| id(state, One, i)).collect(),
        targets: targets.iter().map(|&i| id(state, Two, i)).collect(),
    }
}

/// One: a d6 showing 3 and a Fire d10 showing 8. Two: a d6 showing 5 and a d20 showing 20.
fn power_needing_two() -> (GameState, DeclaredAttack) {
    let s = attack_state(
        vec![die(6, 3), die(10, 8).with(DieSkill::Fire)],
        vec![die(6, 5), die(20, 20)],
        One,
    );
    let attack = declared(&s, AttackKind::Power, &[0], &[0]);
    let (s, _) = ok(
        &s,
        One,
        Action::Attack {
            attack: attack.clone(),
        },
    );
    (s, attack)
}

#[test]
fn an_attack_fire_can_rescue_waits_for_adjustment() {
    let (s, attack) = power_needing_two();
    assert_eq!(
        s.waiting_for,
        WaitingFor::AdjustFire {
            player: One,
            attack
        }
    );
}

#[test]
fn fire_turn_down_completes_the_attack() {
    let (s, _) = power_needing_two();
    let (attacker, fire) = (id(&s, One, 0), id(&s, One, 1));
    let (after, _) = ok(
        &s,
        One,
        Action::FireTurnDown {
            fire: vec![(fire, 2)],
            boosts: vec![(attacker, 2)],
        },
    );
    assert_eq!(after.player(One).captured.len(), 1);
    assert_eq!(after.player(One).die(fire).unwrap().value, 6);
    assert_eq!(after.waiting_for, WaitingFor::Attack { player: Two });
}

#[test]
fn fire_adjustments_are_validated() {
    let (s, _) = power_needing_two();
    let (attacker, fire) = (id(&s, One, 0), id(&s, One, 1));
    let target = id(&s, Two, 0);
    let refuse = |fire_list: Vec<_>, boosts: Vec<_>| {
        err(
            &s,
            One,
            Action::FireTurnDown {
                fire: fire_list,
                boosts,
            },
        )
    };
    assert_eq!(
        refuse(vec![(fire, 3)], vec![(attacker, 2)]),
        RuleError::InvalidFire {
            reason: FireError::AmountsDiffer
        }
    );
    assert_eq!(
        refuse(vec![(fire, 1)], vec![(attacker, 1)]),
        RuleError::InvalidFire {
            reason: FireError::AttackStillInvalid
        }
    );
    assert_eq!(
        refuse(vec![(fire, 4)], vec![(attacker, 4)]),
        RuleError::InvalidFire {
            reason: FireError::OutOfRange { die: attacker }
        }
    );
    assert_eq!(
        refuse(vec![(attacker, 2)], vec![(attacker, 2)]),
        RuleError::InvalidFire {
            reason: FireError::NotFire { die: attacker }
        }
    );
    assert_eq!(
        refuse(vec![(fire, 2)], vec![(target, 2)]),
        RuleError::InvalidFire {
            reason: FireError::NotAttacker { die: target }
        }
    );
}

#[test]
fn cancel_fire_returns_to_the_attack_choice() {
    let (s, _) = power_needing_two();
    let (after, _) = ok(&s, One, Action::CancelFire);
    assert_eq!(after.waiting_for, WaitingFor::Attack { player: One });
    assert_eq!(after.player(One).active[1].value, 8);
}

#[test]
fn fire_helps_skill_attacks_across_several_attackers() {
    let s = attack_state(
        vec![die(6, 2), die(6, 2), die(6, 6).with(DieSkill::Fire)],
        vec![die(10, 7), die(20, 20)],
        One,
    );
    let attack = declared(&s, AttackKind::Skill, &[0, 1], &[0]);
    let (s, _) = ok(&s, One, Action::Attack { attack });
    let (a, b, f) = (id(&s, One, 0), id(&s, One, 1), id(&s, One, 2));
    let (after, _) = ok(
        &s,
        One,
        Action::FireTurnDown {
            fire: vec![(f, 3)],
            boosts: vec![(a, 1), (b, 2)],
        },
    );
    assert_eq!(after.player(One).captured.len(), 1);
}

#[test]
fn fire_cannot_rescue_beyond_its_capacity_or_other_attack_kinds() {
    let s = attack_state(
        vec![die(6, 1), die(4, 2).with(DieSkill::Fire)],
        vec![die(6, 6), die(20, 20)],
        One,
    );
    // Need 5, the Fire die can give only 1.
    let attack = declared(&s, AttackKind::Power, &[0], &[0]);
    assert_eq!(
        err(&s, One, Action::Attack { attack }),
        RuleError::IllegalAttack {
            reason: AttackError::ValuesDoNotMatch
        }
    );
    let s = attack_state(
        vec![
            die(10, 5).with(DieSkill::Shadow),
            die(10, 10).with(DieSkill::Fire),
        ],
        vec![die(4, 3), die(20, 20)],
        One,
    );
    let attack = declared(&s, AttackKind::Shadow, &[0], &[0]);
    assert_eq!(
        err(&s, One, Action::Attack { attack }),
        RuleError::IllegalAttack {
            reason: AttackError::ValuesDoNotMatch
        }
    );
}

#[test]
fn pass_refused_when_only_fire_assisted_attack_exists() {
    // Review Focus 2.
    let s = attack_state(
        vec![die(6, 3), die(10, 8).with(DieSkill::Fire)],
        vec![die(6, 5)],
        One,
    );
    assert_eq!(err(&s, One, Action::Pass), RuleError::PassNotAllowed);
}
