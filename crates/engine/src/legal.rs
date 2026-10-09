use serde::{Deserialize, Serialize};

use crate::action::{Action, DeclaredAttack, SpecChoice};
use crate::apply::apply;
use crate::attack;
use crate::die::SwingLetter;
use crate::fire;
use crate::ids::{DieId, PlayerId};
use crate::setup::{option_slots, required_letters};
use crate::skill::DieSkill;
use crate::state::{GameState, WaitingFor};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct SwingRequest {
    pub letter: SwingLetter,
    pub min: u8,
    pub max: u8,
    pub current: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct OptionRequest {
    pub die: DieId,
    pub a: u8,
    pub b: u8,
    pub current: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct FocusRange {
    pub die: DieId,
    pub min: u8,
    pub current: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct AttackOption {
    pub attack: DeclaredAttack,
    /// R 6.3: legal only once Fire dice are turned down.
    pub needs_fire: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct FireCapacity {
    pub die: DieId,
    pub max_amount: u8,
}

/// What `player` may do now, described for an interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum LegalActions {
    Wait,
    SpecifyDice {
        swing: Vec<SwingRequest>,
        options: Vec<OptionRequest>,
    },
    ChooseReserve {
        dice: Vec<DieId>,
    },
    ReactToInitiative {
        focus: Vec<FocusRange>,
        chance: Vec<DieId>,
    },
    Attack {
        attacks: Vec<AttackOption>,
        can_pass: bool,
    },
    AdjustFire {
        attack: DeclaredAttack,
        fire: Vec<FireCapacity>,
        boosts: Vec<FireCapacity>,
    },
    GameOver,
}

pub fn legal_actions(state: &GameState, player: PlayerId) -> LegalActions {
    let me = state.player(player);
    match &state.waiting_for {
        WaitingFor::GameOver { .. } => LegalActions::GameOver,
        WaitingFor::SpecifyDice { pending } if pending.contains(&player) => {
            LegalActions::SpecifyDice {
                swing: required_letters(me)
                    .into_iter()
                    .map(|letter| SwingRequest {
                        letter,
                        min: *letter.range().start(),
                        max: *letter.range().end(),
                        current: me.choice.swing_size(letter),
                    })
                    .collect(),
                options: option_slots(me)
                    .into_iter()
                    .map(|(die, a, b)| OptionRequest {
                        die,
                        a,
                        b,
                        current: me.choice.option_size(die),
                    })
                    .collect(),
            }
        }
        WaitingFor::ChooseReserve { player: p } if *p == player => LegalActions::ChooseReserve {
            dice: me
                .slots
                .iter()
                .filter(|s| !s.in_play && s.spec.skills.has(DieSkill::Reserve))
                .map(|s| s.id)
                .collect(),
        },
        WaitingFor::ReactToInitiative { player: p } if *p == player => {
            LegalActions::ReactToInitiative {
                focus: me
                    .active
                    .iter()
                    .filter(|d| d.has(DieSkill::Focus) && d.value > d.min())
                    .map(|d| FocusRange {
                        die: d.id,
                        min: d.min(),
                        current: d.value,
                    })
                    .collect(),
                chance: me
                    .active
                    .iter()
                    .filter(|d| d.has(DieSkill::Chance))
                    .map(|d| d.id)
                    .collect(),
            }
        }
        WaitingFor::Attack { player: p } if *p == player => {
            let mut attacks: Vec<AttackOption> = attack::possible_attacks(state, player)
                .into_iter()
                .map(|attack| AttackOption {
                    attack,
                    needs_fire: false,
                })
                .collect();
            attacks.extend(
                fire::assisted_attacks(state, player)
                    .into_iter()
                    .map(|attack| AttackOption {
                        attack,
                        needs_fire: true,
                    }),
            );
            let can_pass = attacks.is_empty();
            LegalActions::Attack { attacks, can_pass }
        }
        WaitingFor::AdjustFire { player: p, attack } if *p == player => LegalActions::AdjustFire {
            attack: attack.clone(),
            fire: fire::fire_dice(state, player, attack)
                .into_iter()
                .map(|(die, max_amount)| FireCapacity { die, max_amount })
                .collect(),
            boosts: fire::attacker_room(state, player, attack)
                .into_iter()
                .filter(|(_, room)| *room > 0)
                .map(|(die, max_amount)| FireCapacity { die, max_amount })
                .collect(),
        },
        WaitingFor::SpecifyDice { .. }
        | WaitingFor::ChooseReserve { .. }
        | WaitingFor::ReactToInitiative { .. }
        | WaitingFor::Attack { .. }
        | WaitingFor::AdjustFire { .. } => LegalActions::Wait,
    }
}

const MAX_SPEC_CANDIDATES: usize = 64;

fn spec_candidates(swing: &[SwingRequest], options: &[OptionRequest]) -> Vec<Action> {
    let mut choices = vec![SpecChoice::default()];
    for req in swing {
        let mut sizes = vec![req.min, req.min + (req.max - req.min) / 2, req.max];
        sizes.dedup();
        let mut next = Vec::new();
        for c in &choices {
            for &size in &sizes {
                let mut c = c.clone();
                c.swing.push((req.letter, size));
                next.push(c);
            }
        }
        next.truncate(MAX_SPEC_CANDIDATES);
        choices = next;
    }
    for req in options {
        let mut sizes = vec![req.a, req.b];
        sizes.dedup();
        let mut next = Vec::new();
        for c in &choices {
            for &size in &sizes {
                let mut c = c.clone();
                c.options.push((req.die, size));
                next.push(c);
            }
        }
        next.truncate(MAX_SPEC_CANDIDATES);
        choices = next;
    }
    choices
        .into_iter()
        .map(|choice| Action::SpecifyDice { choice })
        .collect()
}

fn focus_candidates(focus: &[FocusRange]) -> Vec<Action> {
    let mut actions: Vec<Action> = focus
        .iter()
        .flat_map(|f| {
            (f.min..f.current).map(move |v| Action::FocusTurnDown {
                dice: vec![(f.die, v)],
            })
        })
        .collect();
    if focus.len() > 1 {
        actions.push(Action::FocusTurnDown {
            dice: focus.iter().map(|f| (f.die, f.min)).collect(),
        });
    }
    actions
}

/// Spreads `total` greedily over `capacities`, in order.
fn spread(total: u8, capacities: &[FireCapacity]) -> Vec<(DieId, u8)> {
    let mut left = total;
    let mut out = Vec::new();
    for c in capacities {
        if left == 0 {
            break;
        }
        let take = left.min(c.max_amount);
        if take > 0 {
            out.push((c.die, take));
            left -= take;
        }
    }
    out
}

fn fire_candidates(fire: &[FireCapacity], boosts: &[FireCapacity]) -> Vec<Action> {
    let sum = |list: &[FireCapacity]| list.iter().map(|c| u32::from(c.max_amount)).sum::<u32>();
    let limit = u8::try_from(sum(fire).min(sum(boosts))).unwrap_or(u8::MAX);
    (1..=limit)
        .map(|total| Action::FireTurnDown {
            fire: spread(total, fire),
            boosts: spread(total, boosts),
        })
        .collect()
}

/// Concrete actions for `player`, all accepted by `apply` (for the AI and for tests).
/// Never empty for a player who must act. Surrender is left out.
pub fn candidate_actions(state: &GameState, player: PlayerId) -> Vec<Action> {
    let raw = match legal_actions(state, player) {
        LegalActions::Wait | LegalActions::GameOver => Vec::new(),
        LegalActions::SpecifyDice { swing, options } => spec_candidates(&swing, &options),
        LegalActions::ChooseReserve { dice } => std::iter::once(None)
            .chain(dice.into_iter().map(Some))
            .map(|die| Action::ChooseReserve { die })
            .collect(),
        LegalActions::ReactToInitiative { focus, chance } => {
            let mut actions = vec![Action::PassInitiative];
            actions.extend(chance.into_iter().map(|die| Action::ChanceReroll { die }));
            actions.extend(focus_candidates(&focus));
            actions
        }
        LegalActions::Attack { attacks, can_pass } => {
            let mut actions: Vec<Action> = attacks
                .into_iter()
                .map(|o| Action::Attack { attack: o.attack })
                .collect();
            if can_pass {
                actions.push(Action::Pass);
            }
            actions
        }
        LegalActions::AdjustFire { fire, boosts, .. } => {
            let mut actions = vec![Action::CancelFire];
            actions.extend(fire_candidates(&fire, &boosts));
            actions
        }
    };
    raw.into_iter()
        .filter(|a| apply(state, player, a.clone()).is_ok())
        .collect()
}
