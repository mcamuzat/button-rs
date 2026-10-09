mod common;

use PlayerId::{One, Two};
use buttonmen_engine::testing::{attack_state, button, die, fixed, id, opening_state};
use buttonmen_engine::{
    Action, AttackKind, DieShape, DieSkill, DieSpec, LegalActions, MatchConfig, PlayerId,
    SwingLetter, apply, candidate_actions, legal_actions, new_game,
};
use common::ok;

#[test]
fn only_actors_get_actions() {
    let s = attack_state(vec![die(6, 4)], vec![die(6, 3)], One);
    assert_eq!(legal_actions(&s, Two), LegalActions::Wait);
    assert!(candidate_actions(&s, Two).is_empty());
}

#[test]
fn specify_dice_lists_ranges() {
    let s = new_game(
        [
            button("A", vec![DieSpec::new(DieShape::Swing(SwingLetter::X))]),
            button("B", vec![fixed(6)]),
        ],
        1,
        MatchConfig::default(),
    );
    let LegalActions::SpecifyDice { swing, options } = legal_actions(&s, One) else {
        panic!("expected SpecifyDice")
    };
    assert_eq!(
        (
            swing[0].letter,
            swing[0].min,
            swing[0].max,
            swing[0].current
        ),
        (SwingLetter::X, 4, 20, None)
    );
    assert!(options.is_empty());
    for action in candidate_actions(&s, One) {
        assert!(apply(&s, One, action).is_ok());
    }
}

#[test]
fn attack_lists_every_legal_attack_and_forbids_pass() {
    let s = attack_state(vec![die(6, 4), die(6, 2)], vec![die(6, 3), die(6, 6)], One);
    let LegalActions::Attack { attacks, can_pass } = legal_actions(&s, One) else {
        panic!("expected Attack")
    };
    assert!(!can_pass);
    // Power 4→3 and Skill 4+2→6 are both possible.
    let kinds: Vec<_> = attacks
        .iter()
        .map(|a| (a.attack.kind, a.attack.attackers.len()))
        .collect();
    assert!(kinds.contains(&(AttackKind::Power, 1)));
    assert!(kinds.contains(&(AttackKind::Skill, 2)));
    assert!(attacks.iter().all(|a| !a.needs_fire));
    for option in attacks {
        assert!(
            apply(
                &s,
                One,
                Action::Attack {
                    attack: option.attack
                }
            )
            .is_ok()
        );
    }
}

#[test]
fn stuck_players_can_pass() {
    let s = attack_state(vec![die(6, 1)], vec![die(6, 6)], One);
    assert_eq!(
        legal_actions(&s, One),
        LegalActions::Attack {
            attacks: vec![],
            can_pass: true
        }
    );
    assert_eq!(candidate_actions(&s, One), vec![Action::Pass]);
}

#[test]
fn fire_assisted_attacks_are_flagged() {
    let s = attack_state(
        vec![die(6, 3), die(10, 8).with(DieSkill::Fire)],
        vec![die(6, 5)],
        One,
    );
    let LegalActions::Attack { attacks, can_pass } = legal_actions(&s, One) else {
        panic!("expected Attack")
    };
    assert!(!can_pass);
    assert!(
        attacks
            .iter()
            .any(|a| a.needs_fire && a.attack.kind == AttackKind::Power)
    );
}

#[test]
fn adjust_fire_offers_capacities_and_working_candidates() {
    let s = attack_state(
        vec![die(6, 3), die(10, 8).with(DieSkill::Fire)],
        vec![die(6, 5), die(20, 20)],
        One,
    );
    let attack = buttonmen_engine::DeclaredAttack {
        kind: AttackKind::Power,
        attackers: vec![id(&s, One, 0)],
        targets: vec![id(&s, Two, 0)],
    };
    let (s, _) = ok(&s, One, Action::Attack { attack });
    let LegalActions::AdjustFire { fire, boosts, .. } = legal_actions(&s, One) else {
        panic!("expected AdjustFire")
    };
    assert_eq!(fire[0].max_amount, 7);
    assert_eq!(boosts[0].max_amount, 3);
    let candidates = candidate_actions(&s, One);
    assert!(candidates.contains(&Action::CancelFire));
    assert!(
        candidates
            .iter()
            .any(|a| matches!(a, Action::FireTurnDown { .. }))
    );
}

#[test]
fn reactions_offer_pass_chance_and_winning_focus() {
    let s = opening_state(vec![die(10, 3)], vec![die(10, 5).with(DieSkill::Focus)]);
    let LegalActions::ReactToInitiative { focus, chance } = legal_actions(&s, Two) else {
        panic!("expected ReactToInitiative")
    };
    assert_eq!((focus[0].min, focus[0].current), (1, 5));
    assert!(chance.is_empty());
    let candidates = candidate_actions(&s, Two);
    assert!(candidates.contains(&Action::PassInitiative));
    assert!(
        candidates
            .iter()
            .any(|a| matches!(a, Action::FocusTurnDown { .. }))
    );
}
