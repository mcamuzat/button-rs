use buttonmen_ai::{Level, choose_action};
use buttonmen_data::{original_buttons, parse_buttonweavers_json, parse_recipe};
use buttonmen_engine::{
    Action, Button, DieSkill, GameEvent, GameState, MatchConfig, PlayerId, RuleError, Viewer,
    new_game, skill_info, view_for,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct SeatButton {
    pub name: String,
    pub recipe: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct MatchSetup {
    pub buttons: [SeatButton; 2],
    pub seed: u32,
    pub max_wins: u8,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum ApplyOutcome {
    Applied { events: Vec<GameEvent> },
    Rejected { error: RuleError, message: String },
}

fn to_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("engine types always serialize")
}

fn parse<'a, T: Deserialize<'a>>(json: &'a str) -> Result<T, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}

pub fn new_match(setup_json: &str) -> Result<GameState, String> {
    let setup: MatchSetup = parse(setup_json)?;
    let button = |b: &SeatButton| -> Result<Button, String> {
        Ok(Button {
            name: b.name.clone(),
            dice: parse_recipe(&b.recipe).map_err(|e| format!("{} : {e}", b.name))?,
        })
    };
    let buttons = [button(&setup.buttons[0])?, button(&setup.buttons[1])?];
    Ok(new_game(
        buttons,
        u64::from(setup.seed),
        MatchConfig {
            max_wins: setup.max_wins,
        },
    ))
}

pub fn view(state: &GameState, viewer_json: &str) -> Result<String, String> {
    let viewer: Viewer = parse(viewer_json)?;
    Ok(to_json(&view_for(state, viewer)))
}

pub fn apply(
    state: &mut GameState,
    player_json: &str,
    action_json: &str,
) -> Result<String, String> {
    let player: PlayerId = parse(player_json)?;
    let action: Action = parse(action_json)?;
    let outcome = match buttonmen_engine::apply(state, player, action) {
        Ok((next, events)) => {
            *state = next;
            ApplyOutcome::Applied { events }
        }
        Err(error) => ApplyOutcome::Rejected {
            message: error.to_string(),
            error,
        },
    };
    Ok(to_json(&outcome))
}

pub fn catalog(imported_json: Option<&str>) -> Result<String, String> {
    let mut recipes = original_buttons();
    if let Some(json) = imported_json {
        recipes.extend(parse_buttonweavers_json(json).map_err(|e| e.to_string())?);
    }
    Ok(to_json(&buttonmen_data::build_catalog(&recipes)))
}

pub fn skills() -> String {
    to_json(&DieSkill::ALL.map(skill_info))
}

pub fn state_json(state: &GameState) -> String {
    to_json(state)
}

/// The AI's action for `player` in a state saved with `state_json` (JSON `null` when
/// that player has nothing to do).
pub fn ai_action(
    state_json: &str,
    player_json: &str,
    level_json: &str,
    seed: u32,
) -> Result<String, String> {
    let state: GameState = parse(state_json)?;
    let player: PlayerId = parse(player_json)?;
    let level: Level = parse(level_json)?;
    Ok(to_json(&choose_action(
        &state,
        player,
        level,
        u64::from(seed),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SETUP: &str = r#"{
        "buttons": [{"name": "A", "recipe": "(4) (6) (X)"}, {"name": "B", "recipe": "(8) (X)"}],
        "seed": 7,
        "max_wins": 3
    }"#;

    #[test]
    fn a_match_starts_with_both_players_choosing_swing() {
        let state = new_match(SETUP).unwrap();
        let view: serde_json::Value =
            serde_json::from_str(&view(&state, r#"{"type":"Spectator"}"#).unwrap()).unwrap();
        assert_eq!(view["waiting_for"]["type"], "SpecifyDice");
        assert_eq!(view["actors"], serde_json::json!(["One", "Two"]));
    }

    #[test]
    fn apply_reports_events_or_a_rejection() {
        let mut state = new_match(SETUP).unwrap();
        let bad = r#"{"type":"SpecifyDice","choice":{"swing":[["X",99]],"options":[]}}"#;
        let out: serde_json::Value =
            serde_json::from_str(&apply(&mut state, r#""One""#, bad).unwrap()).unwrap();
        assert_eq!(out["type"], "Rejected");
        assert_eq!(out["error"]["type"], "SwingOutOfRange");
        assert!(
            out["message"]
                .as_str()
                .unwrap()
                .contains("hors de la plage")
        );
        let good = r#"{"type":"SpecifyDice","choice":{"swing":[["X",10]],"options":[]}}"#;
        let out: serde_json::Value =
            serde_json::from_str(&apply(&mut state, r#""One""#, good).unwrap()).unwrap();
        assert_eq!(out["type"], "Applied");
    }

    #[test]
    fn malformed_input_is_an_error_not_a_panic() {
        assert!(new_match("{}").is_err());
        assert!(new_match(&SETUP.replace("(8) (X)", "o(8)")).is_err());
        let mut state = new_match(SETUP).unwrap();
        assert!(apply(&mut state, r#""Three""#, r#"{"type":"Pass"}"#).is_err());
        assert!(view(&state, "nope").is_err());
    }

    #[test]
    fn catalog_and_skills_serialize() {
        let catalog: serde_json::Value = serde_json::from_str(&catalog(None).unwrap()).unwrap();
        assert_eq!(catalog["entries"].as_array().unwrap().len(), 20);
        assert_eq!(catalog["editions"][0]["category"], "Original");
        let skills: serde_json::Value = serde_json::from_str(&skills()).unwrap();
        assert_eq!(skills.as_array().unwrap().len(), 15);
    }

    #[test]
    fn the_ai_answers_from_a_saved_state() {
        let state = new_match(SETUP).unwrap();
        let saved = state_json(&state);
        let action = ai_action(&saved, r#""Two""#, r#""Normal""#, 5).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&action).unwrap();
        assert_eq!(parsed["type"], "SpecifyDice");
        // The answer is a legal action for that state.
        let mut state = state;
        let out: serde_json::Value =
            serde_json::from_str(&apply(&mut state, r#""Two""#, &action).unwrap()).unwrap();
        assert_eq!(out["type"], "Applied");
        assert!(ai_action(&saved, r#""Two""#, r#""Godlike""#, 5).is_err());
    }
}
