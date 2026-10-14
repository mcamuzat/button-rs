//! Scenario builders for tests and for other crates' tests (feature "testing").

use crate::die::{Die, DieShape, DieSpec, Sides, TwinPart};
use crate::ids::{DieId, PlayerId};
use crate::setup::new_game;
use crate::skill::DieSkill;
use crate::state::{Button, GameState, MatchConfig, WaitingFor};

pub fn fixed(n: u8) -> DieSpec {
    DieSpec::new(DieShape::Fixed(n))
}

pub fn button(name: &str, dice: Vec<DieSpec>) -> Button {
    Button {
        name: name.to_string(),
        dice,
    }
}

/// A die of a hand-made scenario, with a forced value.
#[derive(Debug, Clone)]
pub struct Placed {
    pub spec: DieSpec,
    pub value: u8,
}

impl Placed {
    pub fn with(mut self, skill: DieSkill) -> Self {
        self.spec.skills.insert(skill);
        self
    }
}

/// An `n`-sided die showing `value`.
pub fn die(n: u8, value: u8) -> Placed {
    Placed {
        spec: fixed(n),
        value,
    }
}

/// An `(a,b)` twin die showing `value`.
pub fn twin(a: u8, b: u8, value: u8) -> Placed {
    Placed {
        spec: DieSpec::new(DieShape::Twin(TwinPart::Fixed(a), TwinPart::Fixed(b))),
        value,
    }
}

fn sides_of(spec: &DieSpec) -> Sides {
    match spec.shape {
        DieShape::Fixed(n) => Sides::Single(n),
        DieShape::Twin(TwinPart::Fixed(a), TwinPart::Fixed(b)) => Sides::Twin(a, b),
        DieShape::Swing(_) | DieShape::Option(..) | DieShape::Twin(..) => {
            panic!("scenario dice must have fixed sizes")
        }
    }
}

/// A round in the attack phase, `to_move` to play, with exactly these dice and values.
/// Die ids: player One's dice are 0.., player Two's follow.
pub fn attack_state(one: Vec<Placed>, two: Vec<Placed>, to_move: PlayerId) -> GameState {
    let specs = |v: &[Placed]| v.iter().map(|p| p.spec.clone()).collect();
    let mut state = new_game(
        [button("Un", specs(&one)), button("Deux", specs(&two))],
        1,
        MatchConfig::default(),
    );
    for (p, placed) in [(PlayerId::One, &one), (PlayerId::Two, &two)] {
        let ps = state.player_mut(p);
        for slot in &mut ps.slots {
            slot.in_play = true;
        }
        ps.active = ps
            .slots
            .iter()
            .zip(placed.iter())
            .map(|(slot, pl)| {
                let sides = sides_of(&pl.spec);
                assert!(
                    (sides.min()..=sides.max()).contains(&pl.value),
                    "value out of range"
                );
                Die {
                    id: slot.id,
                    skills: slot.spec.skills.clone(),
                    sides,
                    value: pl.value,
                    dizzy: false,
                }
            })
            .collect();
        ps.captured.clear();
        ps.out_of_play.clear();
    }
    state.initiative = Some(to_move);
    state.consecutive_passes = 0;
    state.waiting_for = WaitingFor::Attack { player: to_move };
    state
}

/// The id of the `index`-th die of `player`'s button.
pub fn id(state: &GameState, player: PlayerId, index: usize) -> DieId {
    state.player(player).slots[index].id
}

/// Like `attack_state`, but the initiative is then decided from the forced values
/// (R 3.3–3.5), so the state may wait for an initiative reaction.
pub fn opening_state(one: Vec<Placed>, two: Vec<Placed>) -> GameState {
    let mut state = attack_state(one, two, PlayerId::One);
    state.initiative = None;
    crate::initiative::determine(&mut state, &mut Vec::new());
    state
}
