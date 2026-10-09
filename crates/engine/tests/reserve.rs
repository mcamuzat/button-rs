mod common;

use PlayerId::{One, Two};
use buttonmen_engine::testing::{button, fixed};
use buttonmen_engine::{
    Action, AttackKind, DeclaredAttack, DieShape, DieSkill, DieSpec, GameState, MatchConfig,
    PlayerId, RuleError, SpecChoice, SwingLetter, WaitingFor, new_game,
};
use common::{err, ok};

/// One has two d1 against Two's single d1 (plus Two's reserve dice, out of play):
/// One always starts (longer list, R 3.3), captures Two's only die, and Two loses round 1.
fn after_first_round(two_dice: Vec<DieSpec>) -> GameState {
    let state = new_game(
        [button("A", vec![fixed(1), fixed(1)]), button("B", two_dice)],
        4,
        MatchConfig::default(),
    );
    assert_eq!(state.waiting_for, WaitingFor::Attack { player: One });
    let attack = DeclaredAttack {
        kind: AttackKind::Power,
        attackers: vec![state.player(One).active[0].id],
        targets: vec![state.player(Two).active[0].id],
    };
    let (state, _) = ok(&state, One, Action::Attack { attack });
    assert_eq!(state.last_round_loser, Some(Two));
    state
}

#[test]
fn the_loser_may_add_a_reserve_die() {
    let state = after_first_round(vec![fixed(1), fixed(20).with(DieSkill::Reserve)]);
    assert_eq!(state.waiting_for, WaitingFor::ChooseReserve { player: Two });
    let reserve = state.player(Two).slots[1].id;
    let (state, _) = ok(&state, Two, Action::ChooseReserve { die: Some(reserve) });
    assert!(state.player(Two).slots[1].in_play);
    assert!(
        !state.player(Two).slots[1]
            .spec
            .skills
            .has(DieSkill::Reserve)
    );
    assert_eq!(state.player(Two).active.len(), 2);
}

#[test]
fn reserve_choice_is_validated_and_may_be_declined() {
    let state = after_first_round(vec![fixed(1), fixed(20).with(DieSkill::Reserve)]);
    let not_reserve = state.player(Two).slots[0].id;
    assert_eq!(
        err(
            &state,
            Two,
            Action::ChooseReserve {
                die: Some(not_reserve)
            }
        ),
        RuleError::InvalidReserve { die: not_reserve }
    );
    assert_eq!(
        err(&state, One, Action::ChooseReserve { die: None }),
        RuleError::NotYourTurn
    );
    let (state, _) = ok(&state, Two, Action::ChooseReserve { die: None });
    assert_eq!(state.player(Two).active.len(), 1);
}

#[test]
fn a_reserve_swing_die_must_be_sized() {
    let reserve_x = DieSpec::new(DieShape::Swing(SwingLetter::X)).with(DieSkill::Reserve);
    let state = after_first_round(vec![fixed(1), reserve_x]);
    let reserve = state.player(Two).slots[1].id;
    let (state, _) = ok(&state, Two, Action::ChooseReserve { die: Some(reserve) });
    assert_eq!(
        state.waiting_for,
        WaitingFor::SpecifyDice { pending: vec![Two] }
    );
    let x = Action::SpecifyDice {
        choice: SpecChoice {
            swing: vec![(SwingLetter::X, 12)],
            options: vec![],
        },
    };
    let (state, _) = ok(&state, Two, x);
    assert_eq!(state.player(Two).active[1].max(), 12);
}
