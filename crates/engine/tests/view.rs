mod common;

use PlayerId::{One, Two};
use buttonmen_engine::testing::{attack_state, button, die, fixed};
use buttonmen_engine::{
    Action, DieShape, DieSkill, DieSpec, LegalActions, MatchConfig, PlayerId, SpecChoice,
    SwingLetter, Viewer, new_game, view_for,
};
use common::ok;

#[test]
fn pending_swing_hidden_from_opponent() {
    // Review Focus 4.
    let x = || DieSpec::new(DieShape::Swing(SwingLetter::X));
    let s = new_game(
        [
            button("A", vec![fixed(6), x()]),
            button("B", vec![fixed(6), x()]),
        ],
        1,
        MatchConfig::default(),
    );
    let choice = SpecChoice {
        swing: vec![(SwingLetter::X, 17)],
        options: vec![],
    };
    let (s, _) = ok(
        &s,
        One,
        Action::SpecifyDice {
            choice: choice.clone(),
        },
    );
    let theirs = view_for(&s, Viewer::Seat { player: Two });
    assert!(theirs.players[0].has_chosen);
    assert_eq!(theirs.players[0].pending_choice, None);
    assert_eq!(theirs.players[0].swing[0].size, None);
    let mine = view_for(&s, Viewer::Seat { player: One });
    assert_eq!(mine.players[0].pending_choice, Some(choice));
    let json = serde_json::to_string(&theirs).unwrap();
    assert!(!json.contains("17"), "the hidden size leaked: {json}");
}

#[test]
fn dice_carry_their_notation_and_scores_are_in_half_points() {
    let s = attack_state(
        vec![die(6, 4).with(DieSkill::Poison), die(7, 1)],
        vec![die(6, 3)],
        One,
    );
    let v = view_for(&s, Viewer::Seat { player: One });
    assert_eq!(v.players[0].active[0].notation, "p(6)");
    assert_eq!(v.players[0].recipe, "p(6) (7)");
    // Poison 6 kept: -6 points (-12); d7 kept: 3.5 points (7).
    assert_eq!(v.players[0].round_score, -5);
    assert_eq!(v.actors, vec![One]);
    assert!(matches!(v.legal, LegalActions::Attack { .. }));
}

#[test]
fn spectators_and_waiting_seats_get_no_actions() {
    let s = attack_state(vec![die(6, 4)], vec![die(6, 3)], One);
    assert_eq!(view_for(&s, Viewer::Spectator).legal, LegalActions::Wait);
    assert_eq!(
        view_for(&s, Viewer::Seat { player: Two }).legal,
        LegalActions::Wait
    );
}

#[test]
fn captured_dice_show_their_points() {
    let s = attack_state(vec![die(6, 4)], vec![die(6, 3), die(20, 20)], One);
    let attack = buttonmen_engine::DeclaredAttack {
        kind: buttonmen_engine::AttackKind::Power,
        attackers: vec![s.player(One).active[0].id],
        targets: vec![s.player(Two).active[0].id],
    };
    let (s, _) = ok(&s, One, Action::Attack { attack });
    let v = view_for(&s, Viewer::Spectator);
    assert_eq!(v.players[0].captured[0].points, 12);
    assert_eq!(v.players[0].captured[0].die.notation, "(6)");
}
