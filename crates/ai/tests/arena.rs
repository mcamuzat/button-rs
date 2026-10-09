use std::time::{Duration, Instant};

use buttonmen_ai::{Level, choose_action};
use buttonmen_engine::testing::{attack_state, button, die, fixed};
use buttonmen_engine::{
    DieShape, DieSkill, DieSpec, GameState, MatchConfig, PlayerId, SwingLetter, WaitingFor, apply,
    new_game,
};

fn buttons() -> Vec<Vec<DieSpec>> {
    let x = DieSpec::new(DieShape::Swing(SwingLetter::X));
    vec![
        vec![fixed(4), fixed(4), fixed(10), fixed(12), x.clone()],
        vec![fixed(6), fixed(8), fixed(12), fixed(20), x.clone()],
        vec![
            fixed(4).with(DieSkill::Poison),
            fixed(8),
            fixed(10),
            fixed(12).with(DieSkill::Speed),
            x,
        ],
    ]
}

fn winner(one: Level, two: Level, a: usize, b: usize, seed: u64) -> PlayerId {
    let pool = buttons();
    let mut state: GameState = new_game(
        [button("A", pool[a].clone()), button("B", pool[b].clone())],
        seed,
        MatchConfig::default(),
    );
    for step in 0..3000u64 {
        if let WaitingFor::GameOver { result } = state.waiting_for {
            return result.winner;
        }
        let actor = state.actors()[0];
        let level = if actor == PlayerId::One { one } else { two };
        let action = choose_action(&state, actor, level, seed * 7919 + step).expect("action");
        state = apply(&state, actor, action).expect("legal").0;
    }
    panic!("no end");
}

#[test]
#[ignore = "slow in debug: cargo test -p buttonmen-ai --release -- --ignored"]
fn hard_beats_easy() {
    let mut hard_wins = 0;
    let mut games = 0;
    for a in 0..3 {
        for b in 0..3 {
            for seed in 0..2u64 {
                // Alternate seats so that neither side keeps a seat advantage.
                let hard_is_one = (a + b + seed as usize).is_multiple_of(2);
                let (one, two) = if hard_is_one {
                    (Level::Hard, Level::Easy)
                } else {
                    (Level::Easy, Level::Hard)
                };
                let w = winner(one, two, a, b, seed);
                if (w == PlayerId::One) == hard_is_one {
                    hard_wins += 1;
                }
                games += 1;
            }
        }
    }
    println!("Difficile bat Facile : {hard_wins} / {games}");
    assert!(
        hard_wins * 3 >= games * 2,
        "Hard won only {hard_wins} / {games}"
    );
}

#[test]
#[ignore = "timing: cargo test -p buttonmen-ai --release -- --ignored"]
fn hard_decides_quickly_on_large_buttons() {
    // Review Focus 5: nine dice each, the largest playable buttonweavers size.
    let nine = |v: u8| {
        (0..9)
            .map(|i| die(4 + 2 * i, v.min(4 + 2 * i)))
            .collect::<Vec<_>>()
    };
    let s = attack_state(nine(3), nine(5), PlayerId::One);
    let start = Instant::now();
    let action = choose_action(&s, PlayerId::One, Level::Hard, 1);
    let spent = start.elapsed();
    println!("Difficile, 9 dés contre 9 : {spent:?}");
    assert!(action.is_some());
    assert!(spent < Duration::from_secs(3), "{spent:?}");
}
