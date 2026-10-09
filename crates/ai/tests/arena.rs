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

/// Review Focus 5: the AI must answer quickly even in the heaviest positions. Budget for
/// a native release build; WASM in a browser is about three times slower.
const BUDGET: Duration = Duration::from_millis(400);

fn timed(state: &GameState, me: PlayerId) -> Duration {
    let start = Instant::now();
    assert!(choose_action(state, me, Level::Hard, 1).is_some());
    start.elapsed()
}

#[test]
#[ignore = "timing: cargo test -p buttonmen-ai --release -- --ignored"]
fn hard_decides_quickly_on_large_buttons() {
    // Eight Poison dice each, like buttonweavers' Super Germ against Max: many attacks.
    let poison = |sides: u8, value: u8| die(sides, value).with(DieSkill::Poison);
    let one = [
        (10, 3),
        (12, 7),
        (16, 9),
        (20, 14),
        (24, 5),
        (30, 22),
        (30, 17),
        (20, 11),
    ];
    let two = [
        (12, 6),
        (12, 2),
        (20, 13),
        (20, 8),
        (30, 25),
        (30, 4),
        (20, 19),
        (20, 10),
    ];
    let s = attack_state(
        one.iter().map(|&(n, v)| poison(n, v)).collect(),
        two.iter().map(|&(n, v)| poison(n, v)).collect(),
        PlayerId::One,
    );
    let candidates = buttonmen_engine::candidate_actions(&s, PlayerId::One).len();
    assert!(
        candidates >= 30,
        "only {candidates} candidates: not a heavy position"
    );
    let spent = timed(&s, PlayerId::One);
    println!("Difficile, attaque à {candidates} coups : {spent:?}");
    assert!(spent < BUDGET, "{spent:?}");
}

#[test]
#[ignore = "timing: cargo test -p buttonmen-ai --release -- --ignored"]
fn hard_chooses_many_swing_dice_quickly() {
    // buttonweavers' CactusJack against schwa: options and five swing letters.
    let recipe = |r: &str| buttonmen_data::parse_recipe(r).expect("valid recipe");
    let s = new_game(
        [
            button("CactusJack", recipe("z(8/12) (4/16) s(6/10) z(X) s(U)")),
            button("schwa", recipe("f(V) t(W) d(X) v(Y) B(Z)")),
        ],
        1,
        MatchConfig::default(),
    );
    for me in [PlayerId::One, PlayerId::Two] {
        let spent = timed(&s, me);
        println!("Difficile, choix des Swing ({me:?}) : {spent:?}");
        assert!(spent < BUDGET, "{spent:?}");
    }
}
