use std::collections::BTreeSet;

use crate::action::SpecChoice;
use crate::die::{Die, DieShape, Sides, SwingLetter, TwinPart};
use crate::error::RuleError;
use crate::event::GameEvent;
use crate::ids::{DieId, PlayerId};
use crate::initiative;
use crate::rng::SeededRng;
use crate::skill::DieSkill;
use crate::state::{Button, DieSlot, GameState, MatchConfig, PlayerState, Record, WaitingFor};

/// Creates a match and plays it forward until the first decision.
pub fn new_game(buttons: [Button; 2], seed: u64, config: MatchConfig) -> GameState {
    let mut next_id = 0u16;
    let players = buttons.map(|button| {
        let slots = button
            .dice
            .into_iter()
            .map(|spec| {
                let id = DieId(next_id);
                next_id += 1;
                // R 6.11: reserve dice start out of play.
                let in_play = !spec.skills.has(DieSkill::Reserve);
                DieSlot { id, spec, in_play }
            })
            .collect();
        PlayerState {
            button: button.name,
            slots,
            choice: SpecChoice::default(),
            pending_choice: None,
            active: Vec::new(),
            captured: Vec::new(),
            record: Record::default(),
        }
    });
    let mut state = GameState {
        config,
        players,
        round: 1,
        initiative: None,
        consecutive_passes: 0,
        last_round_loser: None,
        waiting_for: WaitingFor::SpecifyDice {
            pending: Vec::new(),
        },
        rng: SeededRng::new(seed),
    };
    begin_round(&mut state, &mut Vec::new());
    state
}

/// R 1.2: swing letters used by `player`'s dice in play.
pub(crate) fn required_letters(player: &PlayerState) -> BTreeSet<SwingLetter> {
    let mut letters = BTreeSet::new();
    for slot in player.slots.iter().filter(|s| s.in_play) {
        match slot.spec.shape {
            DieShape::Swing(l) => {
                letters.insert(l);
            }
            DieShape::Twin(a, b) => {
                for part in [a, b] {
                    if let TwinPart::Swing(l) = part {
                        letters.insert(l);
                    }
                }
            }
            DieShape::Fixed(_) | DieShape::Option(..) => {}
        }
    }
    letters
}

/// R 1.3: option dice in play, with their two sizes.
pub(crate) fn option_slots(player: &PlayerState) -> Vec<(DieId, u8, u8)> {
    player
        .slots
        .iter()
        .filter(|s| s.in_play)
        .filter_map(|s| match s.spec.shape {
            DieShape::Option(a, b) => Some((s.id, a, b)),
            DieShape::Fixed(_) | DieShape::Swing(_) | DieShape::Twin(..) => None,
        })
        .collect()
}

/// R 1.2 + R 1.3: `choice` gives exactly one valid size per letter and per option die.
pub(crate) fn validate_choice(player: &PlayerState, choice: &SpecChoice) -> Result<(), RuleError> {
    let letters = required_letters(player);
    for &letter in &letters {
        let sizes: Vec<u8> = choice
            .swing
            .iter()
            .filter(|(l, _)| *l == letter)
            .map(|(_, s)| *s)
            .collect();
        match sizes.as_slice() {
            [size] if letter.range().contains(size) => {}
            [size] => {
                return Err(RuleError::SwingOutOfRange {
                    letter,
                    size: *size,
                });
            }
            _ => return Err(RuleError::SwingMismatch { letter }),
        }
    }
    if let Some((letter, _)) = choice.swing.iter().find(|(l, _)| !letters.contains(l)) {
        return Err(RuleError::SwingMismatch { letter: *letter });
    }
    let options = option_slots(player);
    for &(die, a, b) in &options {
        let picks: Vec<u8> = choice
            .options
            .iter()
            .filter(|(d, _)| *d == die)
            .map(|(_, s)| *s)
            .collect();
        match picks.as_slice() {
            [size] if *size == a || *size == b => {}
            _ => return Err(RuleError::InvalidOption { die }),
        }
    }
    if let Some((die, _)) = choice
        .options
        .iter()
        .find(|(d, _)| !options.iter().any(|(o, _, _)| o == d))
    {
        return Err(RuleError::InvalidOption { die: *die });
    }
    Ok(())
}

fn needs_choice(player: &PlayerState) -> bool {
    !required_letters(player).is_empty() || !option_slots(player).is_empty()
}

/// R 2.3 + R 6.11: what happens before the dice are rolled.
pub(crate) fn begin_round(state: &mut GameState, events: &mut Vec<GameEvent>) {
    if let Some(loser) = state.last_round_loser {
        let has_reserve = state
            .player(loser)
            .slots
            .iter()
            .any(|s| !s.in_play && s.spec.skills.has(DieSkill::Reserve));
        if has_reserve {
            state.waiting_for = WaitingFor::ChooseReserve { player: loser };
            return;
        }
    }
    ask_for_choices(state, events);
}

/// R 3.1 + R 2.3: round 1, both players choose; later, the loser may change, and anyone
/// whose dice are not all sized (a reserve swing die just added) must choose.
pub(crate) fn ask_for_choices(state: &mut GameState, events: &mut Vec<GameEvent>) {
    let pending: Vec<PlayerId> = PlayerId::ALL
        .into_iter()
        .filter(|&p| {
            let player = state.player(p);
            needs_choice(player)
                && (validate_choice(player, &player.choice).is_err()
                    || state.last_round_loser == Some(p))
        })
        .collect();
    if pending.is_empty() {
        start_round(state, events);
    } else {
        state.waiting_for = WaitingFor::SpecifyDice { pending };
    }
}

/// Handles `Action::SpecifyDice`.
pub(crate) fn specify_dice(
    state: &mut GameState,
    player: PlayerId,
    choice: SpecChoice,
    events: &mut Vec<GameEvent>,
) -> Result<(), RuleError> {
    let WaitingFor::SpecifyDice { pending } = &state.waiting_for else {
        return Err(RuleError::WrongPhase);
    };
    if !pending.contains(&player) {
        return Err(RuleError::NotYourTurn);
    }
    let remaining: Vec<PlayerId> = pending.iter().copied().filter(|&p| p != player).collect();
    validate_choice(state.player(player), &choice)?;
    state.player_mut(player).pending_choice = Some(choice);
    if !remaining.is_empty() {
        state.waiting_for = WaitingFor::SpecifyDice { pending: remaining };
        return Ok(());
    }
    // R 3.1: choices are revealed together.
    for p in PlayerId::ALL {
        if let Some(choice) = state.player_mut(p).pending_choice.take() {
            events.push(GameEvent::DiceSpecified {
                player: p,
                choice: choice.clone(),
            });
            state.player_mut(p).choice = choice;
        }
    }
    start_round(state, events);
    Ok(())
}

fn resolve_sides(shape: DieShape, id: DieId, choice: &SpecChoice) -> Sides {
    let swing = |l: SwingLetter| {
        choice
            .swing_size(l)
            .expect("choices are validated before rolling")
    };
    let part = |p: TwinPart| match p {
        TwinPart::Fixed(n) => n,
        TwinPart::Swing(l) => swing(l),
    };
    match shape {
        DieShape::Fixed(n) => Sides::Single(n),
        DieShape::Swing(l) => Sides::Single(swing(l)),
        DieShape::Option(..) => Sides::Single(
            choice
                .option_size(id)
                .expect("choices are validated before rolling"),
        ),
        DieShape::Twin(a, b) => Sides::Twin(part(a), part(b)),
    }
}

/// R 3.2: every die in play is rebuilt from its recipe and rolled.
pub(crate) fn start_round(state: &mut GameState, events: &mut Vec<GameEvent>) {
    events.push(GameEvent::RoundStarted { round: state.round });
    state.consecutive_passes = 0;
    for p in PlayerId::ALL {
        let mut dice: Vec<Die> = {
            let player = state.player(p);
            player
                .slots
                .iter()
                .filter(|s| s.in_play)
                .map(|s| Die {
                    id: s.id,
                    skills: s.spec.skills.clone(),
                    sides: resolve_sides(s.spec.shape, s.id, &player.choice),
                    value: 0,
                    dizzy: false,
                })
                .collect()
        };
        for die in &mut dice {
            die.value = die.sides.roll(&mut state.rng);
        }
        events.push(GameEvent::DiceRolled {
            player: p,
            dice: dice.iter().map(|d| (d.id, d.value)).collect(),
        });
        let player = state.player_mut(p);
        player.active = dice;
        player.captured.clear();
    }
    initiative::determine(state, events);
}
