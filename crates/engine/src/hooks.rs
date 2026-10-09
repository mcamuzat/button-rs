//! Skill hooks: the only places where skills change the rules (spec §5.4).
//! Each hook matches every `DieSkill` explicitly, so a new skill must decide each hook.

use crate::die::Die;
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
