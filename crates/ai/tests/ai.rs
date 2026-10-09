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
