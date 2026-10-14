use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// Die skills supported in v1 (R 6). The order is the display order of notations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum DieSkill {
    Berserk,
    Chance,
    Fire,
    Focus,
    Konstant,
    Mighty,
    Null,
    Ornery,
    Poison,
    Queer,
    Reserve,
    Shadow,
    Slow,
    Speed,
    Stealth,
    Stinger,
    TimeAndSpace,
    Trip,
    Value,
}

impl DieSkill {
    pub const ALL: [DieSkill; 19] = [
        DieSkill::Berserk,
        DieSkill::Chance,
        DieSkill::Fire,
        DieSkill::Focus,
        DieSkill::Konstant,
        DieSkill::Mighty,
        DieSkill::Null,
        DieSkill::Ornery,
        DieSkill::Poison,
        DieSkill::Queer,
        DieSkill::Reserve,
        DieSkill::Shadow,
        DieSkill::Slow,
        DieSkill::Speed,
        DieSkill::Stealth,
        DieSkill::Stinger,
        DieSkill::TimeAndSpace,
        DieSkill::Trip,
        DieSkill::Value,
    ];

    /// R 1.1: recipe letter, as in buttonweavers' skill abbreviation table.
    pub fn letter(self) -> char {
        match self {
            DieSkill::Berserk => 'B',
            DieSkill::Chance => 'c',
            DieSkill::Fire => 'F',
            DieSkill::Focus => 'f',
            DieSkill::Konstant => 'k',
            DieSkill::Mighty => 'H',
            DieSkill::Null => 'n',
            DieSkill::Ornery => 'o',
            DieSkill::Poison => 'p',
            DieSkill::Queer => 'q',
            DieSkill::Reserve => 'r',
            DieSkill::Shadow => 's',
            DieSkill::Slow => 'w',
            DieSkill::Speed => 'z',
            DieSkill::Stealth => 'd',
            DieSkill::Stinger => 'g',
            DieSkill::TimeAndSpace => '^',
            DieSkill::Trip => 't',
            DieSkill::Value => 'v',
        }
    }

    pub fn from_letter(c: char) -> Option<DieSkill> {
        DieSkill::ALL.into_iter().find(|s| s.letter() == c)
    }
}

/// The skills of one die.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct SkillSet(BTreeSet<DieSkill>);

impl SkillSet {
    pub fn has(&self, skill: DieSkill) -> bool {
        self.0.contains(&skill)
    }

    pub fn insert(&mut self, skill: DieSkill) {
        self.0.insert(skill);
    }

    pub fn remove(&mut self, skill: DieSkill) {
        self.0.remove(&skill);
    }

    pub fn iter(&self) -> impl Iterator<Item = DieSkill> + '_ {
        self.0.iter().copied()
    }
}

impl FromIterator<DieSkill> for SkillSet {
    fn from_iter<I: IntoIterator<Item = DieSkill>>(iter: I) -> Self {
        SkillSet(iter.into_iter().collect())
    }
}

/// Display information for a skill, shown by the client as a tooltip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct SkillInfo {
    pub skill: DieSkill,
    pub letter: char,
    pub name: String,
    pub rule: String,
    pub description: String,
}

pub fn skill_info(skill: DieSkill) -> SkillInfo {
    let (name, rule, description) = match skill {
        DieSkill::Berserk => (
            "Berserk",
            "R 6.1",
            "Pas d'attaque Skill. Attaque Berserk : capture des dés dont la somme égale sa valeur, puis perd Berserk et passe à la moitié de ses faces.",
        ),
        DieSkill::Chance => (
            "Chance",
            "R 6.2",
            "Sans l'initiative en début de manche, peut relancer un dé Chance pour tenter de la prendre.",
        ),
        DieSkill::Fire => (
            "Fire",
            "R 6.3",
            "Pas d'attaque Power. Peut baisser sa valeur pour augmenter d'autant des dés qui font une attaque Power ou Skill.",
        ),
        DieSkill::Focus => (
            "Focus",
            "R 6.4",
            "Sans l'initiative en début de manche, peut baisser sa valeur pour la prendre ; il ne peut alors pas servir à la première attaque.",
        ),
        DieSkill::Konstant => (
            "Konstant",
            "R 6.5",
            "Ne se relance jamais. Pas d'attaque Power ni Skill seul ; dans une attaque Skill à plusieurs dés, ajoute ou soustrait sa valeur.",
        ),
        DieSkill::Mighty => (
            "Mighty",
            "R 6.17",
            "Grandit d'un cran à chaque relance : 1, 2, 4, 6, 8, 10, 12, 16, 20, 30.",
        ),
        DieSkill::Null => (
            "Null",
            "R 6.6",
            "Vaut zéro point, et les dés qu'il aide à capturer valent zéro point.",
        ),
        DieSkill::Ornery => (
            "Ornery",
            "R 6.18",
            "Se relance après chaque attaque de son propriétaire, même s'il n'y participe pas ; pas quand il passe.",
        ),
        DieSkill::Poison => (
            "Poison",
            "R 6.7",
            "Points négatifs : gardé, retire sa taille ; capturé, retire la moitié de sa taille.",
        ),
        DieSkill::Queer => (
            "Queer",
            "R 6.8",
            "Normal sur une valeur paire, Shadow sur une valeur impaire.",
        ),
        DieSkill::Reserve => (
            "Reserve",
            "R 6.11",
            "Hors jeu au départ ; après une manche perdue, on peut le faire entrer en jeu pour la suite du match.",
        ),
        DieSkill::Shadow => (
            "Shadow",
            "R 6.10",
            "Pas d'attaque Power. Capture un dé de valeur supérieure ou égale à la sienne, sans dépasser sa taille.",
        ),
        DieSkill::Slow => ("Slow", "R 6.16", "Ne compte pas pour l'initiative."),
        DieSkill::Speed => (
            "Speed",
            "R 6.9",
            "Peut capturer plusieurs dés dont la somme égale exactement sa valeur.",
        ),
        DieSkill::Stealth => (
            "Stealth",
            "R 6.12",
            "N'attaque et ne peut être capturé que par des attaques Skill d'au moins deux dés.",
        ),
        DieSkill::Stinger => (
            "Stinger",
            "R 6.13",
            "En attaque Skill, compte pour toute valeur de son minimum à sa valeur actuelle. Ignoré pour l'initiative.",
        ),
        DieSkill::TimeAndSpace => (
            "TimeAndSpace",
            "R 6.19",
            "S'il attaque et se relance sur une valeur impaire, son propriétaire rejoue.",
        ),
        DieSkill::Trip => (
            "Trip",
            "R 6.14",
            "Attaque Trip : relance les deux dés, capture si le Trip fait au moins autant. Ignoré pour l'initiative.",
        ),
        DieSkill::Value => (
            "Value",
            "R 6.15",
            "Compte pour sa valeur au lieu de sa taille ; les dés qu'il capture comptent pour leur valeur au moment de la capture.",
        ),
    };
    SkillInfo {
        skill,
        letter: skill.letter(),
        name: name.to_string(),
        rule: rule.to_string(),
        description: description.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_round_trip_and_are_unique() {
        let mut letters: Vec<char> = DieSkill::ALL.iter().map(|s| s.letter()).collect();
        for s in DieSkill::ALL {
            assert_eq!(DieSkill::from_letter(s.letter()), Some(s));
        }
        letters.sort();
        letters.dedup();
        assert_eq!(letters.len(), DieSkill::ALL.len());
        assert_eq!(DieSkill::from_letter('D'), None); // Doppelganger : hors jalon 4
    }

    #[test]
    fn skill_set_basics() {
        let mut set: SkillSet = [DieSkill::Poison, DieSkill::Shadow].into_iter().collect();
        assert!(set.has(DieSkill::Poison));
        set.remove(DieSkill::Poison);
        assert!(!set.has(DieSkill::Poison));
        set.insert(DieSkill::Berserk);
        assert_eq!(
            set.iter().collect::<Vec<_>>(),
            vec![DieSkill::Berserk, DieSkill::Shadow]
        );
    }

    #[test]
    fn every_skill_has_french_info_and_a_rule() {
        for s in DieSkill::ALL {
            let info = skill_info(s);
            assert_eq!(info.letter, s.letter());
            assert!(info.rule.starts_with("R 6."));
            assert!(!info.description.is_empty());
        }
    }
}
