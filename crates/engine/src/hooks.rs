//! Skill hooks: the only places where skills change the rules (spec §5.4).
//! Each hook matches every `DieSkill` explicitly, so a new skill must decide each hook.

use crate::action::AttackKind;
use crate::die::Die;
use crate::rng::SeededRng;
use crate::skill::DieSkill;
use crate::state::CaptureScoring;
use std::collections::BTreeSet;

/// R 3.3 + R 6.13 + R 6.14: does this die count when comparing initiative?
pub(crate) fn counts_for_initiative(die: &Die) -> bool {
    die.skills.iter().all(|skill| match skill {
        // R 6.13 / R 6.14: Stinger and Trip dice are ignored for initiative.
        DieSkill::Stinger | DieSkill::Trip => false,
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
        | DieSkill::Value => true,
    })
}

/// Rerolls a die, honouring `rerolls`.
pub(crate) fn reroll(die: &mut Die, rng: &mut SeededRng) {
    if rerolls(die) {
        die.value = die.sides.roll(rng);
    }
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
