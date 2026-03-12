use gloo_storage::{LocalStorage, Storage};
use quest_core::game_state::GameState;

const SAVE_KEY: &str = "quest_save_data";

/// Saves the game state to local storage.
pub fn save_game(state: &GameState) {
    if let Ok(json) = state.serialize() {
        let _ = LocalStorage::set(SAVE_KEY, json);
    }
}

/// Loads the game state from local storage.
/// Returns None if no save exists or if the data is invalid.
pub fn load_game() -> Option<GameState> {
    let json: String = LocalStorage::get(SAVE_KEY).ok()?;
    GameState::deserialize(&json).ok()
}

/// Checks whether a valid save exists in local storage.
pub fn has_valid_save() -> bool {
    if let Ok(json) = LocalStorage::get::<String>(SAVE_KEY) {
        GameState::validate(&json)
    } else {
        false
    }
}
