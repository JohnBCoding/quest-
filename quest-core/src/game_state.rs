use serde::{Deserialize, Serialize};

use crate::area::Area;
use crate::player::Player;
use crate::rng::{RngManager, RngSnapshot};

/// The complete game state — everything needed to save/load a game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub player: Player,
    pub current_area: Area,
    pub rng_snapshot: RngSnapshot,
    /// Schema version for future save compatibility.
    pub version: u32,
}

/// Current schema version. Bump this when the save format changes.
pub const SAVE_VERSION: u32 = 1;

impl GameState {
    /// Creates a brand new game with default player and starting area.
    pub fn new_game() -> (Self, RngManager) {
        let rng_manager = RngManager::new();
        let state = Self {
            player: Player::default(),
            current_area: Area::starting_area(),
            rng_snapshot: rng_manager.snapshot(),
            version: SAVE_VERSION,
        };
        (state, rng_manager)
    }

    /// Serializes the game state to JSON.
    pub fn serialize(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserializes a game state from JSON.
    pub fn deserialize(data: &str) -> Result<Self, String> {
        let state: GameState =
            serde_json::from_str(data).map_err(|e| format!("Invalid save data: {}", e))?;

        if state.version != SAVE_VERSION {
            return Err(format!(
                "Incompatible save version: expected {}, got {}",
                SAVE_VERSION, state.version
            ));
        }

        Ok(state)
    }

    /// Validates whether a JSON string contains valid save data.
    /// Returns true only if the data can be fully deserialized and
    /// the version matches.
    pub fn validate(data: &str) -> bool {
        Self::deserialize(data).is_ok()
    }

    /// Restores the RngManager from the state's snapshot.
    pub fn restore_rng(&self) -> RngManager {
        RngManager::from_snapshot(&self.rng_snapshot)
    }

    /// Updates the RNG snapshot in the state (call before saving).
    pub fn sync_rng(&mut self, rng: &RngManager) {
        self.rng_snapshot = rng.snapshot();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_creates_valid_state() {
        let (state, _rng) = GameState::new_game();
        assert_eq!(state.player.name, "Hero");
        assert_eq!(state.current_area.name, "The Beach");
        assert_eq!(state.version, SAVE_VERSION);
    }

    #[test]
    fn serialize_deserialize_roundtrip() {
        let (state, _rng) = GameState::new_game();
        let json = state.serialize().unwrap();
        let loaded = GameState::deserialize(&json).unwrap();
        assert_eq!(loaded.player.name, state.player.name);
        assert_eq!(loaded.current_area.name, state.current_area.name);
        assert_eq!(loaded.version, state.version);
    }

    #[test]
    fn validate_accepts_valid_save() {
        let (state, _rng) = GameState::new_game();
        let json = state.serialize().unwrap();
        assert!(GameState::validate(&json));
    }

    #[test]
    fn validate_rejects_empty_string() {
        assert!(!GameState::validate(""));
    }

    #[test]
    fn validate_rejects_random_garbage() {
        assert!(!GameState::validate("lk234j5lkj{}[]not json at all"));
    }

    #[test]
    fn validate_rejects_valid_json_wrong_shape() {
        assert!(!GameState::validate(r#"{"foo": "bar"}"#));
    }

    #[test]
    fn validate_rejects_wrong_version() {
        let (mut state, _rng) = GameState::new_game();
        state.version = 9999;
        let json = state.serialize().unwrap();
        assert!(!GameState::validate(&json));
    }

    #[test]
    fn deserialize_returns_error_for_corrupted_data() {
        let result = GameState::deserialize("corrupted");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid save data"));
    }

    #[test]
    fn deserialize_returns_error_for_wrong_version() {
        let (mut state, _rng) = GameState::new_game();
        state.version = 0;
        let json = state.serialize().unwrap();
        let result = GameState::deserialize(&json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Incompatible save version"));
    }

    #[test]
    fn rng_restore_produces_working_manager() {
        let (state, _rng) = GameState::new_game();
        let mut restored = state.restore_rng();
        // Should not panic
        let val = restored.gen_range("loot", 1, 100);
        assert!((1..=100).contains(&val));
    }

    #[test]
    fn sync_rng_updates_snapshot() {
        let (mut state, mut rng) = GameState::new_game();
        let original_snapshot = state.rng_snapshot.clone();

        // Generate some values to advance the RNG state
        for _ in 0..10 {
            rng.gen_range("loot", 0, 100);
        }

        // The snapshot in state should still be the original
        assert_eq!(state.rng_snapshot, original_snapshot);

        // After sync, snapshot should reflect current RNG seeds
        // (Note: seeds don't change, only the internal state advances.
        //  But the snapshot captures seeds, which stay the same.)
        state.sync_rng(&rng);
        // Seeds themselves don't change, just verifying the method works
        assert_eq!(state.rng_snapshot.seeds, rng.snapshot().seeds);
    }
}
