mod common;

use PlayerId::{One, Two};
use buttonmen_engine::testing::{die, id, opening_state};
use buttonmen_engine::{
    Action, DieSkill, PlayerId, ReactionError, RuleError, SeededRng, WaitingFor,
};
use common::{err, ok};

#[test]
fn player_without_initiative_may_react_with_focus() {
    let state = opening_state(vec![die(10, 3)], vec![die(10, 5).with(DieSkill::Focus)]);
    assert_eq!(state.initiative, Some(One));
    assert_eq!(
        state.waiting_for,
        WaitingFor::ReactToInitiative { player: Two }
    );
    let focus = id(&state, Two, 0);
    let (state, _) = ok(
        &state,
        Two,
        Action::FocusTurnDown {
            dice: vec![(focus, 2)],
        },
    );
    assert_eq!(state.initiative, Some(Two));
    // One has no Focus/Chance dice: Two attacks first, with a dizzy Focus die (R 6.4).
    assert_eq!(state.waiting_for, WaitingFor::Attack { player: Two });
    let d = state.player(Two).die(focus).unwrap();
    assert_eq!(d.value, 2);
    assert!(d.dizzy);
}

#[test]
fn focus_must_lower_the_die_and_gain_initiative() {
    let state = opening_state(vec![die(10, 3)], vec![die(10, 5).with(DieSkill::Focus)]);
    let focus = id(&state, Two, 0);
    let refused = |value| {
        err(
            &state,
            Two,
            Action::FocusTurnDown {
                dice: vec![(focus, value)],
            },
        )
    };
    assert_eq!(
        refused(4),
        RuleError::InvalidReaction {
            reason: ReactionError::DoesNotGainInitiative
        }
    );
    // A tie is not a gain.
    assert_eq!(
        refused(3),
        RuleError::InvalidReaction {
            reason: ReactionError::DoesNotGainInitiative
        }
    );
    assert_eq!(
        refused(6),
        RuleError::InvalidReaction {
            reason: ReactionError::ValueNotLowered { die: focus }
        }
    );
    assert_eq!(
        refused(0),
        RuleError::InvalidReaction {
            reason: ReactionError::ValueNotLowered { die: focus }
        }
    );
    assert_eq!(
        err(&state, Two, Action::FocusTurnDown { dice: vec![] }),
        RuleError::InvalidReaction {
            reason: ReactionError::NoDice
        }
    );
}

#[test]
fn only_focus_dice_can_be_turned_down() {
    let state = opening_state(
        vec![die(10, 3)],
        vec![die(10, 5).with(DieSkill::Focus), die(10, 9)],
    );
    let plain = id(&state, Two, 1);
    assert_eq!(
        err(
            &state,
            Two,
            Action::FocusTurnDown {
                dice: vec![(plain, 1)]
            }
        ),
        RuleError::InvalidReaction {
            reason: ReactionError::NotFocus { die: plain }
        }
    );
}

#[test]
fn the_holder_cannot_react() {
    let state = opening_state(vec![die(10, 3)], vec![die(10, 5).with(DieSkill::Focus)]);
    assert_eq!(
        err(&state, One, Action::PassInitiative),
        RuleError::NotYourTurn
    );
}

#[test]
fn reactions_alternate_until_someone_passes() {
    let state = opening_state(
        vec![die(10, 3).with(DieSkill::Focus)],
        vec![die(10, 5).with(DieSkill::Focus)],
    );
    let (a, b) = (id(&state, One, 0), id(&state, Two, 0));
    let (state, _) = ok(&state, Two, Action::FocusTurnDown { dice: vec![(b, 2)] });
    assert_eq!(
        state.waiting_for,
        WaitingFor::ReactToInitiative { player: One }
    );
    let (state, _) = ok(&state, One, Action::FocusTurnDown { dice: vec![(a, 1)] });
    assert_eq!(
        state.waiting_for,
        WaitingFor::ReactToInitiative { player: Two }
    );
    let (state, _) = ok(&state, Two, Action::PassInitiative);
    assert_eq!(state.waiting_for, WaitingFor::Attack { player: One });
    // R 7.4: only the starting player's lowered dice stay dizzy.
    assert!(state.player(One).die(a).unwrap().dizzy);
    assert!(!state.player(Two).die(b).unwrap().dizzy);
}

#[test]
fn failed_chance_reroll_ends_reactions() {
    // [1, 1] against [1]: One keeps the longer list (R 3.3). A d1 always rerolls 1.
    let state = opening_state(
        vec![die(1, 1), die(1, 1)],
        vec![die(1, 1).with(DieSkill::Chance)],
    );
    assert_eq!(
        state.waiting_for,
        WaitingFor::ReactToInitiative { player: Two }
    );
    let chance = id(&state, Two, 0);
    let (state, _) = ok(&state, Two, Action::ChanceReroll { die: chance });
    assert_eq!(state.waiting_for, WaitingFor::Attack { player: One });
}

#[test]
fn chance_reroll_gives_initiative_only_when_lower() {
    let mut gained = 0;
    let mut kept = 0;
    for seed in 0..60 {
        let mut state = opening_state(vec![die(6, 2)], vec![die(6, 6).with(DieSkill::Chance)]);
        state.rng = SeededRng::new(seed);
        let chance = id(&state, Two, 0);
        let (after, _) = ok(&state, Two, Action::ChanceReroll { die: chance });
        let value = after.player(Two).die(chance).unwrap().value;
        assert!((1..=6).contains(&value));
        match value {
            1 => {
                assert_eq!(after.initiative, Some(Two));
                gained += 1;
            }
            3..=6 => {
                assert_eq!(after.initiative, Some(One));
                kept += 1;
            }
            _ => {} // 2: tie, decided by a coin (R 3.4)
        }
    }
    assert!(gained > 0 && kept > 0);
}

#[test]
fn chance_on_a_konstant_die_keeps_its_value() {
    let state = opening_state(
        vec![die(1, 1), die(1, 1)],
        vec![die(20, 17).with(DieSkill::Chance).with(DieSkill::Konstant)],
    );
    let chance = id(&state, Two, 0);
    let (state, _) = ok(&state, Two, Action::ChanceReroll { die: chance });
    assert_eq!(state.player(Two).die(chance).unwrap().value, 17);
}
