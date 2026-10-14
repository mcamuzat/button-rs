//! Skill hooks: the only places where skills change the rules (spec §5.4).
//! Each hook matches every `DieSkill` explicitly, so a new skill must decide each hook.

use crate::action::AttackKind;
use crate::die::{Die, Sides};
use crate::rng::SeededRng;
use crate::skill::DieSkill;
use crate::state::CaptureScoring;
use std::collections::BTreeSet;

/// R 3.3 + R 6.13 + R 6.14 + R 6.16: does this die count when comparing initiative?
pub(crate) fn counts_for_initiative(die: &Die) -> bool {
    die.skills.iter().all(|skill| match skill {
        // R 6.13 / R 6.14 / R 6.16: Stinger, Trip and Slow dice are ignored for initiative.
        DieSkill::Slow | DieSkill::Stinger | DieSkill::Trip => false,
        DieSkill::Berserk
        | DieSkill::Chance
        | DieSkill::Fire
        | DieSkill::Focus
        | DieSkill::Konstant
        | DieSkill::Null
        | DieSkill::Poison
        | DieSkill::Queer
        | DieSkill::Reserve
        | DieSkill::Shadow
        | DieSkill::Speed
        | DieSkill::Stealth
        | DieSkill::Mighty
        | DieSkill::Ornery
        | DieSkill::TimeAndSpace
        | DieSkill::Value => true,
    })
}

/// R 4.4 + R 6.5: does this die take a new value when it is rerolled?
pub(crate) fn rerolls(die: &Die) -> bool {
    die.skills.iter().all(|skill| match skill {
        // R 6.5: Konstant dice keep their value.
        DieSkill::Konstant => false,
        DieSkill::Berserk
        | DieSkill::Chance
        | DieSkill::Fire
        | DieSkill::Focus
        | DieSkill::Null
        | DieSkill::Poison
        | DieSkill::Queer
        | DieSkill::Reserve
        | DieSkill::Shadow
        | DieSkill::Speed
        | DieSkill::Stealth
        | DieSkill::Stinger
        | DieSkill::Trip
        | DieSkill::Mighty
        | DieSkill::Slow
        | DieSkill::Ornery
        | DieSkill::TimeAndSpace
        | DieSkill::Value => true,
    })
}

/// Why a die is rerolled. Mighty (and Mood, in milestone 4B) skip the die that has just
/// made a Trip attack (BMSkillMighty::pre_roll, flag `JustPerformedTripAttack`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RerollCause {
    Normal,
    TripAttacker,
}

/// R 6.17: does this die grow before each reroll?
fn grows_on_reroll(die: &Die) -> bool {
    die.skills.iter().any(|skill| match skill {
        DieSkill::Mighty => true,
        DieSkill::Berserk
        | DieSkill::Chance
        | DieSkill::Fire
        | DieSkill::Focus
        | DieSkill::Konstant
        | DieSkill::Null
        | DieSkill::Poison
        | DieSkill::Queer
        | DieSkill::Reserve
        | DieSkill::Shadow
        | DieSkill::Slow
        | DieSkill::Speed
        | DieSkill::Stealth
        | DieSkill::Stinger
        | DieSkill::Trip
        | DieSkill::Ornery
        | DieSkill::TimeAndSpace
        | DieSkill::Value => false,
    })
}

/// Rerolls a die, honouring `rerolls` (R 6.5) and Mighty (R 6.17). Returns the sides
/// before a Mighty growth when the die grew.
pub(crate) fn reroll(die: &mut Die, rng: &mut SeededRng, cause: RerollCause) -> Option<Sides> {
    if !rerolls(die) {
        return None;
    }
    let before = die.sides;
    if grows_on_reroll(die) && cause != RerollCause::TripAttacker {
        die.sides = die.sides.grown();
    }
    die.value = die.sides.roll(rng);
    (die.sides != before).then_some(before)
}

/// R 6.18: does this die reroll after every attack of its owner?
pub(crate) fn rerolls_after_any_attack(die: &Die) -> bool {
    die.skills.iter().any(|skill| match skill {
        DieSkill::Ornery => true,
        DieSkill::Berserk
        | DieSkill::Chance
        | DieSkill::Fire
        | DieSkill::Focus
        | DieSkill::Konstant
        | DieSkill::Mighty
        | DieSkill::Null
        | DieSkill::Poison
        | DieSkill::Queer
        | DieSkill::Reserve
        | DieSkill::Shadow
        | DieSkill::Slow
        | DieSkill::Speed
        | DieSkill::Stealth
        | DieSkill::Stinger
        | DieSkill::TimeAndSpace
        | DieSkill::Trip
        | DieSkill::Value => false,
    })
}

/// R 6.19: does this attacker, rerolled to an odd value, give an extra turn?
pub(crate) fn grants_extra_turn(die: &Die) -> bool {
    die.value % 2 == 1
        && rerolls(die)
        && die.skills.iter().any(|skill| match skill {
            DieSkill::TimeAndSpace => true,
            DieSkill::Berserk
            | DieSkill::Chance
            | DieSkill::Fire
            | DieSkill::Focus
            | DieSkill::Konstant
            | DieSkill::Mighty
            | DieSkill::Null
            | DieSkill::Ornery
            | DieSkill::Poison
            | DieSkill::Queer
            | DieSkill::Reserve
            | DieSkill::Shadow
            | DieSkill::Slow
            | DieSkill::Speed
            | DieSkill::Stealth
            | DieSkill::Stinger
            | DieSkill::Trip
            | DieSkill::Value => false,
        })
}

/// Which attacks this die can make (R 4.2, R 4.3, R 6.1, R 6.3, R 6.5, R 6.8–6.10,
/// R 6.12, R 6.14). Grants are applied first, then restrictions.
pub(crate) fn attack_kinds(die: &Die) -> BTreeSet<AttackKind> {
    let odd = die.value % 2 == 1;
    let mut kinds = BTreeSet::from([AttackKind::Power, AttackKind::Skill]);
    for skill in die.skills.iter() {
        match skill {
            // R 6.10.
            DieSkill::Shadow => {
                kinds.insert(AttackKind::Shadow);
            }
            // R 6.8: Shadow on odd values.
            DieSkill::Queer if odd => {
                kinds.insert(AttackKind::Shadow);
            }
            // R 6.9.
            DieSkill::Speed => {
                kinds.insert(AttackKind::Speed);
            }
            // R 6.14.
            DieSkill::Trip => {
                kinds.insert(AttackKind::Trip);
            }
            // R 6.1.
            DieSkill::Berserk => {
                kinds.insert(AttackKind::Berserk);
            }
            DieSkill::Queer
            | DieSkill::Chance
            | DieSkill::Fire
            | DieSkill::Focus
            | DieSkill::Konstant
            | DieSkill::Null
            | DieSkill::Poison
            | DieSkill::Reserve
            | DieSkill::Stealth
            | DieSkill::Stinger
            | DieSkill::Mighty
            | DieSkill::Slow
            | DieSkill::Ornery
            | DieSkill::TimeAndSpace
            | DieSkill::Value => {}
        }
    }
    for skill in die.skills.iter() {
        match skill {
            // R 6.10 / R 6.8: Shadow replaces Power.
            DieSkill::Shadow => {
                kinds.remove(&AttackKind::Power);
            }
            DieSkill::Queer if odd => {
                kinds.remove(&AttackKind::Power);
            }
            // R 6.1: no Skill attacks.
            DieSkill::Berserk => {
                kinds.remove(&AttackKind::Skill);
            }
            // R 6.5 / R 6.3: no Power attacks.
            DieSkill::Konstant | DieSkill::Fire => {
                kinds.remove(&AttackKind::Power);
            }
            // R 6.12: multi-die Skill attacks only (the "multi-die" part is `needs_partner`).
            DieSkill::Stealth => kinds.retain(|k| *k == AttackKind::Skill),
            DieSkill::Queer
            | DieSkill::Chance
            | DieSkill::Focus
            | DieSkill::Null
            | DieSkill::Poison
            | DieSkill::Reserve
            | DieSkill::Speed
            | DieSkill::Stinger
            | DieSkill::Trip
            | DieSkill::Mighty
            | DieSkill::Slow
            | DieSkill::Ornery
            | DieSkill::TimeAndSpace
            | DieSkill::Value => {}
        }
    }
    kinds
}

/// R 6.5 + R 6.12: this die only makes Skill attacks together with at least one other die.
pub(crate) fn needs_partner(die: &Die) -> bool {
    die.skills.iter().any(|skill| match skill {
        DieSkill::Konstant | DieSkill::Stealth => true,
        DieSkill::Berserk
        | DieSkill::Chance
        | DieSkill::Fire
        | DieSkill::Focus
        | DieSkill::Null
        | DieSkill::Poison
        | DieSkill::Queer
        | DieSkill::Reserve
        | DieSkill::Shadow
        | DieSkill::Speed
        | DieSkill::Stinger
        | DieSkill::Trip
        | DieSkill::Mighty
        | DieSkill::Slow
        | DieSkill::Ornery
        | DieSkill::TimeAndSpace
        | DieSkill::Value => false,
    })
}

/// R 6.12: can `target` be captured by a `kind` attack made by `attackers` dice?
pub(crate) fn can_be_captured(target: &Die, kind: AttackKind, attackers: usize) -> bool {
    target.skills.iter().all(|skill| match skill {
        DieSkill::Stealth => kind == AttackKind::Skill && attackers >= 2,
        DieSkill::Berserk
        | DieSkill::Chance
        | DieSkill::Fire
        | DieSkill::Focus
        | DieSkill::Konstant
        | DieSkill::Null
        | DieSkill::Poison
        | DieSkill::Queer
        | DieSkill::Reserve
        | DieSkill::Shadow
        | DieSkill::Speed
        | DieSkill::Stinger
        | DieSkill::Trip
        | DieSkill::Mighty
        | DieSkill::Slow
        | DieSkill::Ornery
        | DieSkill::TimeAndSpace
        | DieSkill::Value => true,
    })
}

/// R 4.3 + R 6.5 + R 6.13 + R 7.3: the values this die may stand for in a Skill attack.
pub(crate) fn skill_values(die: &Die) -> Vec<i32> {
    let mut stinger = false;
    let mut konstant = false;
    for skill in die.skills.iter() {
        match skill {
            DieSkill::Stinger => stinger = true,
            DieSkill::Konstant => konstant = true,
            DieSkill::Berserk
            | DieSkill::Chance
            | DieSkill::Fire
            | DieSkill::Focus
            | DieSkill::Null
            | DieSkill::Poison
            | DieSkill::Queer
            | DieSkill::Reserve
            | DieSkill::Shadow
            | DieSkill::Speed
            | DieSkill::Stealth
            | DieSkill::Trip
            | DieSkill::Mighty
            | DieSkill::Slow
            | DieSkill::Ornery
            | DieSkill::TimeAndSpace
            | DieSkill::Value => {}
        }
    }
    let low = if stinger { die.min() } else { die.value };
    let mut values: Vec<i32> = (low..=die.value).map(i32::from).collect();
    if konstant {
        let negatives: Vec<i32> = values.iter().map(|v| -v).collect();
        values.extend(negatives);
    }
    values
}

/// R 6.6 + R 6.15 + R 7.1: how `target`, captured by these attackers, will score.
pub(crate) fn capture_scoring(attackers: &[&Die], target: &Die) -> CaptureScoring {
    let mut null = false;
    let mut value = false;
    for die in attackers {
        for skill in die.skills.iter() {
            match skill {
                DieSkill::Null => null = true,
                DieSkill::Value => value = true,
                DieSkill::Berserk
                | DieSkill::Chance
                | DieSkill::Fire
                | DieSkill::Focus
                | DieSkill::Konstant
                | DieSkill::Poison
                | DieSkill::Queer
                | DieSkill::Reserve
                | DieSkill::Shadow
                | DieSkill::Speed
                | DieSkill::Stealth
                | DieSkill::Stinger
                | DieSkill::Mighty
                | DieSkill::Slow
                | DieSkill::Ornery
                | DieSkill::TimeAndSpace
                | DieSkill::Trip => {}
            }
        }
    }
    if null {
        CaptureScoring::Zero
    } else if value {
        // R 6.15: valued at the face the target shows when captured.
        CaptureScoring::ByValue(target.value)
    } else {
        CaptureScoring::BySize
    }
}
