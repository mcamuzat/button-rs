use crate::recipe::{RecipeError, parse_recipe};
use buttonmen_engine::Button;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum ButtonSource {
    Original,
    Buttonweavers,
}

/// Family of an edition, in display order: the official Cheapass editions come first.
/// The buttonweavers families are the sections of its `buttonset` table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum SetCategory {
    /// Published by Cheapass Games (buttonweavers « Official Sets »).
    Official,
    /// The buttons shipped with buttonmen-rs.
    Original,
    /// Made for Button Men Online or by its players.
    Online,
    /// Fan and vanity editions.
    Unofficial,
    Fanatics,
    /// RandomBM, CustomBM.
    Special,
    /// An import written before categories were recorded.
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ButtonRecipe {
    pub name: String,
    pub recipe: String,
    pub set: Option<String>,
    pub category: SetCategory,
    /// Release order of the edition within its category.
    pub set_order: u32,
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
    pub category: SetCategory,
    pub source: ButtonSource,
    pub playability: Playability,
}

/// One edition of the catalog, with how many of its buttons can be played.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Edition {
    pub name: String,
    pub category: SetCategory,
    pub playable: u32,
    pub total: u32,
}

/// The catalog as the client shows it: buttons and editions, both in display order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Catalog {
    pub entries: Vec<CatalogEntry>,
    pub editions: Vec<Edition>,
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

const ORIGINAL_SET: &str = "Originaux";

/// The original buttons shipped with the game (`data/buttons.toml`).
pub fn original_buttons() -> Vec<ButtonRecipe> {
    let file: OriginalFile = toml::from_str(include_str!("../../../data/buttons.toml"))
        .expect("data/buttons.toml is valid (checked by tests)");
    file.button
        .into_iter()
        .map(|b| ButtonRecipe {
            name: b.name,
            recipe: b.recipe,
            set: Some(ORIGINAL_SET.to_string()),
            category: SetCategory::Original,
            set_order: 0,
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
    #[serde(default = "other_category")]
    category: SetCategory,
    #[serde(default = "last_order")]
    set_order: u32,
    #[serde(default)]
    special: bool,
}

fn other_category() -> SetCategory {
    SetCategory::Other
}

fn last_order() -> u32 {
    u32::MAX
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
            category: b.category,
            set_order: b.set_order,
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

/// Every button with whether v1 can play it, and why not, grouped by edition: category,
/// then release order; buttons keep their source order within an edition.
pub fn catalog(recipes: &[ButtonRecipe]) -> Vec<CatalogEntry> {
    let mut sorted: Vec<&ButtonRecipe> = recipes.iter().collect();
    sorted
        .sort_by(|a, b| (a.category, a.set_order, &a.set).cmp(&(b.category, b.set_order, &b.set)));
    sorted
        .into_iter()
        .map(|r| CatalogEntry {
            name: r.name.clone(),
            recipe: r.recipe.clone(),
            set: r.set.clone(),
            category: r.category,
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

/// The editions of `entries` (as ordered by [`catalog`]), with their playable counts.
pub fn editions(entries: &[CatalogEntry]) -> Vec<Edition> {
    let mut editions: Vec<Edition> = Vec::new();
    for entry in entries {
        let name = entry.set.as_deref().unwrap_or("Sans édition");
        let same_edition = editions
            .last()
            .is_some_and(|last| last.name == name && last.category == entry.category);
        if !same_edition {
            editions.push(Edition {
                name: name.to_string(),
                category: entry.category,
                playable: 0,
                total: 0,
            });
        }
        let edition = editions.last_mut().expect("an edition was just ensured");
        edition.total += 1;
        if entry.playability == Playability::Playable {
            edition.playable += 1;
        }
    }
    editions
}

/// Buttons and editions for the client.
pub fn build_catalog(recipes: &[ButtonRecipe]) -> Catalog {
    let entries = catalog(recipes);
    let editions = editions(&entries);
    Catalog { entries, editions }
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
        let playability = |name: &str| {
            &entries
                .iter()
                .find(|e| e.name == name)
                .expect("button in catalog")
                .playability
        };
        assert_eq!(playability("Avis"), &Playability::Playable);
        assert!(
            matches!(playability("Zeppo"), Playability::Unplayable { reason } if reason.contains('!'))
        );
        assert!(matches!(
            playability("Echo"),
            Playability::Unplayable { .. }
        ));
    }

    #[test]
    fn catalog_groups_buttons_by_category_then_release_order() {
        let json = r#"[
            {"name": "Ricky", "recipe": "(6) (8) (20) (X)", "set": "Uptown", "category": "Official", "set_order": 3},
            {"name": "Iago", "recipe": "(4) (8) (20) (X)", "set": "Fanatic", "category": "Fanatics", "set_order": 9},
            {"name": "Avis", "recipe": "(4) (4) (10) (12) (X)", "set": "Soldiers", "category": "Official", "set_order": 0},
            {"name": "Hammer", "recipe": "(6) (12) (20) (X)!", "set": "Soldiers", "category": "Official", "set_order": 0}
        ]"#;
        let mut recipes = original_buttons();
        recipes.extend(parse_buttonweavers_json(json).unwrap());
        let Catalog { entries, editions } = build_catalog(&recipes);

        let names: Vec<&str> = entries.iter().take(3).map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["Avis", "Hammer", "Ricky"]);
        assert_eq!(entries.last().unwrap().name, "Iago");

        let summary: Vec<(&str, SetCategory, u32, u32)> = editions
            .iter()
            .map(|e| (e.name.as_str(), e.category, e.playable, e.total))
            .collect();
        assert_eq!(
            summary,
            [
                ("Soldiers", SetCategory::Official, 1, 2),
                ("Uptown", SetCategory::Official, 1, 1),
                (ORIGINAL_SET, SetCategory::Original, 20, 20),
                ("Fanatic", SetCategory::Fanatics, 1, 1),
            ]
        );
    }

    #[test]
    fn an_import_without_categories_lands_in_other() {
        let json = r#"[{"name": "Avis", "recipe": "(4) (4) (10) (12) (X)", "set": "Soldiers"}]"#;
        let recipes = parse_buttonweavers_json(json).unwrap();
        assert_eq!(recipes[0].category, SetCategory::Other);
        let editions = build_catalog(&recipes).editions;
        assert_eq!(editions[0].category, SetCategory::Other);
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
