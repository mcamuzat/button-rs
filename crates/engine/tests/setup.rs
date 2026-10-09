mod common;

use PlayerId::{One, Two};
use buttonmen_engine::testing::{button, fixed};
use buttonmen_engine::{
    Action, DieShape, DieSkill, DieSpec, MatchConfig, PlayerId, RuleError, SpecChoice, SwingLetter,
    TwinPart, WaitingFor, new_game,
};
use common::{err, ok};

fn swing(l: SwingLetter) -> DieSpec {
    DieSpec::new(DieShape::Swing(l))
}

fn x_choice(size: u8) -> Action {
    Action::SpecifyDice {
        choice: SpecChoice {
            swing: vec![(SwingLetter::X, size)],
            options: vec![],
        },
    }
}

fn swing_match() -> buttonmen_engine::GameState {
    new_game(
        [
            button("Avis", vec![fixed(4), fixed(10), swing(SwingLetter::X)]),
            button("Hammer", vec![fixed(6), fixed(20), swing(SwingLetter::X)]),
        ],
        11,
        MatchConfig::default(),
    )
}

#[test]
fn swing_buttons_wait_for_both_players() {
    let state = swing_match();
    assert_eq!(
        state.waiting_for,
        WaitingFor::SpecifyDice {
            pending: vec![One, Two]
        }
    );
    assert_eq!(state.round, 1);
}

#[test]
fn swing_size_must_be_in_range_and_present() {
    let state = swing_match();
    assert_eq!(
        err(&state, One, x_choice(21)),
        RuleError::SwingOutOfRange {
            letter: SwingLetter::X,
            size: 21
        }
    );
    assert_eq!(
        err(
            &state,
            One,
            Action::SpecifyDice {
                choice: SpecChoice::default()
            }
        ),
        RuleError::SwingMismatch {
            letter: SwingLetter::X
        }
    );
    let extra = Action::SpecifyDice {
        choice: SpecChoice {
            swing: vec![(SwingLetter::X, 8), (SwingLetter::Y, 3)],
            options: vec![],
        },
    };
    assert_eq!(
        err(&state, One, extra),
        RuleError::SwingMismatch {
            letter: SwingLetter::Y
        }
    );
}

#[test]
fn a_player_cannot_specify_twice() {
    let (state, _) = ok(&swing_match(), One, x_choice(8));
    assert_eq!(
        state.waiting_for,
        WaitingFor::SpecifyDice { pending: vec![Two] }
    );
    assert_eq!(err(&state, One, x_choice(9)), RuleError::NotYourTurn);
}

#[test]
fn after_both_choices_dice_are_rolled_with_chosen_sizes() {
    let (state, _) = ok(&swing_match(), One, x_choice(8));
    let (state, events) = ok(&state, Two, x_choice(20));
    assert!(matches!(state.waiting_for, WaitingFor::Attack { .. }));
    let one_x = &state.player(One).active[2];
    assert_eq!(one_x.max(), 8);
    assert!((1..=8).contains(&one_x.value));
    assert_eq!(state.player(Two).active[2].max(), 20);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, buttonmen_engine::GameEvent::InitiativeDecided { .. }))
    );
}

#[test]
fn same_letter_twin_halves_share_one_size() {
    let twin_x = DieSpec::new(DieShape::Twin(
        TwinPart::Swing(SwingLetter::X),
        TwinPart::Swing(SwingLetter::X),
    ));
    let state = new_game(
        [
            button("A", vec![twin_x, swing(SwingLetter::X)]),
            button("B", vec![fixed(6)]),
        ],
        5,
        MatchConfig::default(),
    );
    assert_eq!(
        state.waiting_for,
        WaitingFor::SpecifyDice { pending: vec![One] }
    );
    let (state, _) = ok(&state, One, x_choice(7));
    assert_eq!(state.player(One).active[0].max(), 14);
    assert_eq!(state.player(One).active[1].max(), 7);
}

#[test]
fn option_dice_take_one_of_their_two_sizes() {
    let state = new_game(
        [
            button("A", vec![DieSpec::new(DieShape::Option(6, 10))]),
            button("B", vec![fixed(6)]),
        ],
        5,
        MatchConfig::default(),
    );
    let die = state.player(One).slots[0].id;
    let bad = Action::SpecifyDice {
        choice: SpecChoice {
            swing: vec![],
            options: vec![(die, 8)],
        },
    };
    assert_eq!(err(&state, One, bad), RuleError::InvalidOption { die });
    let good = Action::SpecifyDice {
        choice: SpecChoice {
            swing: vec![],
            options: vec![(die, 10)],
        },
    };
    let (state, _) = ok(&state, One, good);
    assert_eq!(state.player(One).active[0].max(), 10);
}

#[test]
fn fixed_buttons_start_attacking_immediately() {
    let state = new_game(
        [
            button("A", vec![fixed(4), fixed(6)]),
            button("B", vec![fixed(8)]),
        ],
        3,
        MatchConfig::default(),
    );
    assert!(matches!(state.waiting_for, WaitingFor::Attack { .. }));
}

#[test]
fn reserve_dice_start_out_of_play() {
    let state = new_game(
        [
            button("A", vec![fixed(4), fixed(20).with(DieSkill::Reserve)]),
            button("B", vec![fixed(8)]),
        ],
        3,
        MatchConfig::default(),
    );
    assert_eq!(state.player(One).active.len(), 1);
    assert!(!state.player(One).slots[1].in_play);
}

#[test]
fn initiative_goes_to_the_lowest_die() {
    // One-sided dice always show 1, (1,1) twins always show 2.
    let state = new_game(
        [
            button("A", vec![fixed(1), fixed(1)]),
            button(
                "B",
                vec![
                    fixed(1),
                    DieSpec::new(DieShape::Twin(TwinPart::Fixed(1), TwinPart::Fixed(1))),
                ],
            ),
        ],
        3,
        MatchConfig::default(),
    );
    // A = [1, 1], B = [1, 2]: equal first die, A's second die is lower.
    assert_eq!(state.initiative, Some(One));
}

#[test]
fn initiative_with_no_eligible_dice_still_decides() {
    // Review Focus 1: player One only has Trip dice (excluded from initiative).
    let state = new_game(
        [
            button("A", vec![fixed(6).with(DieSkill::Trip)]),
            button("B", vec![fixed(6)]),
        ],
        3,
        MatchConfig::default(),
    );
    assert_eq!(state.initiative, Some(Two));
    assert_eq!(state.waiting_for, WaitingFor::Attack { player: Two });
}

#[test]
fn surrender_ends_the_match() {
    let state = new_game(
        [button("A", vec![fixed(6)]), button("B", vec![fixed(6)])],
        3,
        MatchConfig::default(),
    );
    let (state, _) = ok(&state, One, Action::Surrender);
    assert!(matches!(
        state.waiting_for,
        WaitingFor::GameOver {
            result: buttonmen_engine::MatchResult { winner: Two, .. }
        }
    ));
    assert_eq!(err(&state, Two, Action::Pass), RuleError::GameOver);
}
