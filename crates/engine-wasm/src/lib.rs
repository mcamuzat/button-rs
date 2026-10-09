//! WASM bridge: exposes `bridge` to JavaScript. No game logic here.

pub mod bridge;

use buttonmen_engine::GameState;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

fn js(message: String) -> JsError {
    JsError::new(&message)
}

/// The button catalog: the originals, plus the local buttonweavers import when given.
#[wasm_bindgen]
pub fn catalog_json(imported: Option<String>) -> Result<String, JsError> {
    bridge::catalog(imported.as_deref()).map_err(js)
}

#[wasm_bindgen]
pub fn skills_json() -> String {
    bridge::skills()
}

/// A match held on the WASM side; JavaScript only sees views and outcomes.
#[wasm_bindgen]
pub struct WasmMatch {
    state: GameState,
}

#[wasm_bindgen]
impl WasmMatch {
    #[wasm_bindgen(constructor)]
    pub fn new(setup_json: &str) -> Result<WasmMatch, JsError> {
        Ok(WasmMatch {
            state: bridge::new_match(setup_json).map_err(js)?,
        })
    }

    pub fn view(&self, viewer_json: &str) -> Result<String, JsError> {
        bridge::view(&self.state, viewer_json).map_err(js)
    }

    pub fn apply(&mut self, player_json: &str, action_json: &str) -> Result<String, JsError> {
        bridge::apply(&mut self.state, player_json, action_json).map_err(js)
    }

    /// The full state (saving, debugging); never used for display.
    pub fn state_json(&self) -> String {
        bridge::state_json(&self.state)
    }
}

/// The AI's move, computed from a saved state (`WasmMatch::state_json`); runs in a Web Worker.
#[wasm_bindgen]
pub fn ai_choose_action(
    state_json: &str,
    player_json: &str,
    level_json: &str,
    seed: u32,
) -> Result<String, JsError> {
    bridge::ai_action(state_json, player_json, level_json, seed).map_err(js)
}
