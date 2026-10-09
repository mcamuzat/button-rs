//! Skill hooks: the only places where skills change the rules (spec §5.4).
//! Each hook matches every `DieSkill` explicitly, so a new skill must decide each hook.

use crate::die::Die;
use crate::rng::SeededRng;
use crate::skill::DieSkill;

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
