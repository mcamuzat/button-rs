use crate::recipe::{RecipeError, parse_recipe};
use buttonmen_engine::Button;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum ButtonSource {
    Original,
    Buttonweavers,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ButtonRecipe {
    pub name: String,
    pub recipe: String,
    pub set: Option<String>,
    pub source: ButtonSource,
    /// The button has a button-level skill (buttonweavers `btn_special`), not supported in v1.
    pub special: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum Playability {
    Playable,
    Unplayable { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct CatalogEntry {
    pub name: String,
    pub recipe: String,
    pub set: Option<String>,
    pub source: ButtonSource,
    pub playability: Playability,
}

#[derive(Deserialize)]
struct OriginalFile {
    button: Vec<OriginalButton>,
}

#[derive(Deserialize)]
struct OriginalButton {
    name: String,
    recipe: String,
}

/// The original buttons shipped with the game (`data/buttons.toml`).
pub fn original_buttons() -> Vec<ButtonRecipe> {
    let file: OriginalFile = toml::from_str(include_str!("../../../data/buttons.toml"))
        .expect("data/buttons.toml is valid (checked by tests)");
    file.button
        .into_iter()
        .map(|b| ButtonRecipe {
            name: b.name,
            recipe: b.recipe,
            set: Some("Originaux".to_string()),
            source: ButtonSource::Original,
            special: false,
        })
        .collect()
}

#[derive(Deserialize)]
struct ImportedButton {
    name: String,
    recipe: String,
    set: Option<String>,
    #[serde(default)]
    special: bool,
}

/// Reads `data/buttonweavers/buttons.json`, written by `scripts/import-buttonweavers.sh`.
pub fn parse_buttonweavers_json(json: &str) -> Result<Vec<ButtonRecipe>, serde_json::Error> {
    let imported: Vec<ImportedButton> = serde_json::from_str(json)?;
    Ok(imported
        .into_iter()
        .map(|b| ButtonRecipe {
            name: b.name,
            recipe: b.recipe,
            set: b.set,
            source: ButtonSource::Buttonweavers,
            special: b.special,
        })
        .collect())
}

pub fn to_button(recipe: &ButtonRecipe) -> Result<Button, RecipeError> {
    Ok(Button {
        name: recipe.name.clone(),
        dice: parse_recipe(&recipe.recipe)?,
    })
}

/// Every button with whether v1 can play it, and why not.
pub fn catalog(recipes: &[ButtonRecipe]) -> Vec<CatalogEntry> {
    recipes
        .iter()
        .map(|r| CatalogEntry {
            name: r.name.clone(),
            recipe: r.recipe.clone(),
            set: r.set.clone(),
            source: r.source,
            playability: if r.special {
                Playability::Unplayable {
                    reason: "compétence de bouton non prise en charge".to_string(),
                }
            } else {
                match parse_recipe(&r.recipe) {
                    Ok(_) => Playability::Playable,
                    Err(e) => Playability::Unplayable {
                        reason: e.to_string(),
                    },
                }
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_original_button_is_playable_and_unique() {
        let originals = original_buttons();
        assert_eq!(originals.len(), 20);
        let entries = catalog(&originals);
        for entry in &entries {
            assert_eq!(
                entry.playability,
                Playability::Playable,
                "{} : {}",
                entry.name,
                entry.recipe
            );
            assert_eq!(entry.source, ButtonSource::Original);
        }
        let mut names: Vec<_> = originals.iter().map(|b| b.name.clone()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), 20);
    }

    #[test]
    fn originals_cover_every_v1_skill() {
        use buttonmen_engine::DieSkill;
        let originals = original_buttons();
        for skill in DieSkill::ALL {
            let covered = originals
                .iter()
                .filter_map(|b| to_button(b).ok())
                .any(|b| b.dice.iter().any(|d| d.skills.has(skill)));
            assert!(covered, "no original button uses {skill:?}");
        }
    }

    #[test]
    fn buttonweavers_json_is_read_and_classified() {
        let json = r#"[
            {"name": "Avis", "recipe": "(4) (4) (10) (12) (X)", "set": "Soldiers", "special": false},
            {"name": "Zeppo", "recipe": "(4) (12) (20) (X)!", "set": "1999 Rare / Promo", "special": false},
            {"name": "Echo", "recipe": "(4) (4) (10) (12) (X)", "set": "Promo", "special": true}
        ]"#;
        let recipes = parse_buttonweavers_json(json).unwrap();
        assert!(
            recipes
                .iter()
                .all(|r| r.source == ButtonSource::Buttonweavers)
        );
        let entries = catalog(&recipes);
        assert_eq!(entries[0].playability, Playability::Playable);
        assert!(
            matches!(&entries[1].playability, Playability::Unplayable { reason } if reason.contains('!'))
        );
        assert!(matches!(
            &entries[2].playability,
            Playability::Unplayable { .. }
        ));
    }

    #[test]
    fn local_buttonweavers_import_never_panics() {
        // Runs only when the optional import exists (scripts/import-buttonweavers.sh).
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../data/buttonweavers/buttons.json"
        );
        let Ok(json) = std::fs::read_to_string(path) else {
            return;
        };
        let recipes = parse_buttonweavers_json(&json).unwrap();
        let entries = catalog(&recipes);
        let playable = entries
            .iter()
            .filter(|e| e.playability == Playability::Playable)
            .count();
        println!(
            "{playable} / {} boutons buttonweavers jouables",
            entries.len()
        );
        assert!(playable > 0);
    }
}
