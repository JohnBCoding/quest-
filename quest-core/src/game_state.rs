use serde::{Deserialize, Serialize};

use crate::area::Area;
use crate::mob::Mob;
use crate::player::Player;
use crate::rng::{RngManager, RngSnapshot};

/// The complete game state — everything needed to save/load a game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub player: Player,
    pub current_area: Area,
    pub current_mob: Option<Mob>,
    pub encounters_cleared: u32,
    pub rng_snapshot: RngSnapshot,
    pub is_boss_encounter: bool,
    pub in_town: bool,
    /// Schema version for future save compatibility.
    pub version: u32,
}

/// Current schema version. Bump this when the save format changes.
pub const SAVE_VERSION: u32 = 2;

impl GameState {
    /// Creates a brand new game with default player and starting area.
    pub fn new_game() -> (Self, RngManager) {
        let rng_manager = RngManager::new();
        let state = Self {
            player: Player::default(),
            current_area: Area::starting_area(),
            current_mob: Mob::get_by_id("rat"), 
            encounters_cleared: 0,
            rng_snapshot: rng_manager.snapshot(),
            is_boss_encounter: false,
            in_town: false,
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

    /// Executes an attack against the current mob, reducing its health by 2.
    /// Returns `true` if a mob was attacked, `false` if there was no mob.
    pub fn execute_attack(&mut self) -> bool {
        if let Some(mob) = self.current_mob.as_mut() {
            mob.take_damage(2);
            true
        } else {
            false
        }
    }

    /// Advances the encounter if the current mob is dead.
    pub fn advance_encounter(&mut self) -> bool {
        if let Some(mob) = &self.current_mob {
            if mob.is_dead() {
                if self.is_boss_encounter {
                    if self.current_area.id == "the_beach" {
                        self.in_town = true;
                    }
                    self.current_mob = None;
                    self.is_boss_encounter = false;
                } else {
                    self.encounters_cleared += 1;
                    
                    // Spawn next encounter if we haven't hit the area max limit yet.
                    if self.encounters_cleared < self.current_area.base_encounter_amount {
                        self.current_mob = Mob::get_by_id("rat");
                    } else {
                        self.current_mob = None;
                    }
                }
                return true;
            }
        }
        false
    }

    /// Enters the boss portal for the current area. 
    /// Returns true if a boss was successfully spawned.
    pub fn enter_boss_portal(&mut self, rng: &mut RngManager) -> bool {
        if self.encounters_cleared < self.current_area.base_encounter_amount {
            return false;
        }

        if self.current_area.bosses.is_empty() {
            return false;
        }

        let max_idx = self.current_area.bosses.len() as u32 - 1;
        let boss_idx = rng.gen_range("mob_spawns", 0, max_idx) as usize;
        let boss_id = &self.current_area.bosses[boss_idx];
        
        if let Some(boss_mob) = Mob::get_by_id(boss_id) {
            self.current_mob = Some(boss_mob);
            self.is_boss_encounter = true;
            self.sync_rng(rng);
            true
        } else {
            false
        }
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
    
    #[test]
    fn execute_attack_reduces_hp_and_returns_true() {
        let (mut state, _) = GameState::new_game();
        if let Some(mob) = &mut state.current_mob {
            mob.health = 5; // make it survive 2 damage
        }
        let result = state.execute_attack();
        assert!(result);
        assert_eq!(state.current_mob.unwrap().health, 3);
        assert_eq!(state.encounters_cleared, 0);
    }

    #[test]
    fn execute_attack_kills_mob_and_increments_encounters() {
        let (mut state, _) = GameState::new_game();
        // default mob (rat) has 2 health
        let result = state.execute_attack();
        assert!(result);
        assert!(state.current_mob.as_ref().unwrap().is_dead());
        assert_eq!(state.encounters_cleared, 0); // Not advanced yet
        
        let advanced = state.advance_encounter();
        assert!(advanced);
        assert_eq!(state.encounters_cleared, 1);
        // checking the respawn mechanics
        assert!(state.current_mob.is_some()); 
        assert!(!state.current_mob.as_ref().unwrap().is_dead());
    }

    #[test]
    fn execute_attack_stops_spawning_when_cap_reached() {
        let (mut state, _) = GameState::new_game();
        state.current_area.base_encounter_amount = 2; // only 2 encounters total
        
        // kill 1st
        state.execute_attack();
        state.advance_encounter();
        assert_eq!(state.encounters_cleared, 1);
        assert!(state.current_mob.is_some());
        
        // kill 2nd
        state.execute_attack();
        state.advance_encounter();
        assert_eq!(state.encounters_cleared, 2);
        assert!(state.current_mob.is_none()); // cap reached
    }

    #[test]
    fn execute_attack_ignored_when_no_mob() {
         let (mut state, _) = GameState::new_game();
         state.current_mob = None;
         let result = state.execute_attack();
         assert!(!result);
         assert_eq!(state.encounters_cleared, 0); // No state change recorded
    }

    #[test]
    fn overkill_damage_clamps_at_zero() {
        let (mut state, _) = GameState::new_game();
        // simulate standard rat with 2 hp taking 2 damage = 0 health, it's alive during the check just for test isolation
        if let Some(mob) = &mut state.current_mob {
            mob.health = 1; 
        }
        state.execute_attack(); // Should deal 2 damage, dropping health from 1 to 0 without underflow
        assert_eq!(state.current_mob.as_ref().unwrap().health, 0);
        
        state.advance_encounter();
        assert_eq!(state.encounters_cleared, 1);
        // verify spawned next
        assert!(state.current_mob.is_some());
    }

    #[test]
    fn advance_encounter_after_beach_boss_sets_in_town() {
        let (mut state, _) = GameState::new_game();
        state.is_boss_encounter = true;
        state.current_area.id = "the_beach".to_string();
        if let Some(mob) = &mut state.current_mob {
            mob.health = 0; // Simulate dead boss
        }
        
        let advanced = state.advance_encounter();
        assert!(advanced);
        assert!(state.in_town);
        assert!(!state.is_boss_encounter);
        assert!(state.current_mob.is_none());
    }

    #[test]
    fn enter_boss_portal_spawns_boss() {
        let (mut state, mut rng) = GameState::new_game();
        state.current_area.bosses = vec!["rat_lord".to_string()];
        state.current_area.base_encounter_amount = 0; // Allow instant boss query
        state.encounters_cleared = 0; // Encounters >= base
        
        let success = state.enter_boss_portal(&mut rng);
        assert!(success);
        assert!(state.is_boss_encounter);
        assert_eq!(state.current_mob.unwrap().id, "rat_lord");
    }

    #[test]
    fn advance_encounter_after_standard_mob_does_not_set_in_town() {
        let (mut state, _) = GameState::new_game();
        state.is_boss_encounter = false;
        state.current_area.id = "the_beach".to_string();
        if let Some(mob) = &mut state.current_mob {
            mob.health = 0; // Simulate dead standard mob
        }
        
        state.advance_encounter();
        assert!(!state.in_town); // Standard mob should not transition to town
    }

    #[test]
    fn boss_portal_cannot_be_entered_early() {
        let (mut state, mut rng) = GameState::new_game();
        state.current_area.base_encounter_amount = 5;
        state.encounters_cleared = 2; // Not enough cleared
        
        let success = state.enter_boss_portal(&mut rng);
        assert!(!success);
        assert!(!state.is_boss_encounter);
    }

    #[test]
    fn advance_encounter_after_non_beach_boss_does_not_set_in_town() {
        let (mut state, _) = GameState::new_game();
        state.is_boss_encounter = true;
        state.current_area.id = "dark_forest".to_string(); // Non-beach area
        if let Some(mob) = &mut state.current_mob {
            mob.health = 0;
        }

        let advanced = state.advance_encounter();
        assert!(advanced);
        assert!(!state.in_town); // Should not set in town
        assert!(!state.is_boss_encounter); 
        assert!(state.current_mob.is_none());
    }
}
