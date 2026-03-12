use serde::{Deserialize, Serialize};

use crate::area::Area;
use crate::mob::Mob;
use crate::player::Player;
use crate::rng::{RngManager, RngSnapshot};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub player: Player,
    pub current_area: Area,
    pub current_mob: Option<Mob>,
    pub encounters_cleared: u32,
    pub rng_snapshot: RngSnapshot,
    pub is_boss_encounter: bool,
    pub in_town: bool,
    #[serde(default)]
    pub fruit_scene_active: bool,
    #[serde(default)]
    pub pending_fruit_id: Option<String>,
    pub version: u32,
}

pub const SAVE_VERSION: u32 = 3;

impl GameState {
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
            fruit_scene_active: false,
            pending_fruit_id: None,
            version: SAVE_VERSION,
        };
        (state, rng_manager)
    }

    pub fn serialize(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

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

    pub fn validate(data: &str) -> bool {
        Self::deserialize(data).is_ok()
    }

    pub fn restore_rng(&self) -> RngManager {
        RngManager::from_snapshot(&self.rng_snapshot)
    }

    pub fn sync_rng(&mut self, rng: &RngManager) {
        self.rng_snapshot = rng.snapshot();
    }

    pub fn execute_attack(&mut self) -> bool {
        if let Some(mob) = self.current_mob.as_mut() {
            mob.take_damage(2);
            true
        } else {
            false
        }
    }

    pub fn advance_encounter(&mut self) -> bool {
        if let Some(mob) = &self.current_mob {
            if mob.is_dead() {
                if self.is_boss_encounter {
                    self.handle_boss_defeat();
                    self.current_mob = None;
                    self.is_boss_encounter = false;
                } else {
                    self.encounters_cleared += 1;
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

    fn handle_boss_defeat(&mut self) {
        if self.current_area.id == "the_beach" && !self.player.has_auto_combat() {
            self.fruit_scene_active = true;
            self.pending_fruit_id = Some("fruit_of_instinct".to_string());
        } else {
            self.in_town = true;
        }
    }

    pub fn complete_fruit_scene(&mut self) {
        if !self.fruit_scene_active {
            return;
        }
        if let Some(fruit_id) = self.pending_fruit_id.take() {
            self.player.eat_fruit(&fruit_id);
        }
        self.fruit_scene_active = false;
    }

    pub fn enter_area(&mut self, area_id: &str) -> bool {
        if let Some(area) = Area::get_by_id(area_id) {
            self.current_area = area;
            self.encounters_cleared = 0;
            self.current_mob = Mob::get_by_id("rat");
            self.is_boss_encounter = false;
            self.in_town = false;
            true
        } else {
            false
        }
    }

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
        assert!(!state.fruit_scene_active);
        assert!(state.pending_fruit_id.is_none());
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
        let val = restored.gen_range("loot", 1, 100);
        assert!((1..=100).contains(&val));
    }

    #[test]
    fn sync_rng_updates_snapshot() {
        let (mut state, mut rng) = GameState::new_game();
        let original_snapshot = state.rng_snapshot.clone();
        for _ in 0..10 {
            rng.gen_range("loot", 0, 100);
        }
        assert_eq!(state.rng_snapshot, original_snapshot);
        state.sync_rng(&rng);
        assert_eq!(state.rng_snapshot.seeds, rng.snapshot().seeds);
    }

    #[test]
    fn execute_attack_reduces_hp_and_returns_true() {
        let (mut state, _) = GameState::new_game();
        if let Some(mob) = &mut state.current_mob {
            mob.health = 5;
        }
        let result = state.execute_attack();
        assert!(result);
        assert_eq!(state.current_mob.unwrap().health, 3);
        assert_eq!(state.encounters_cleared, 0);
    }

    #[test]
    fn execute_attack_kills_mob_and_increments_encounters() {
        let (mut state, _) = GameState::new_game();
        let result = state.execute_attack();
        assert!(result);
        assert!(state.current_mob.as_ref().unwrap().is_dead());
        assert_eq!(state.encounters_cleared, 0);

        let advanced = state.advance_encounter();
        assert!(advanced);
        assert_eq!(state.encounters_cleared, 1);
        assert!(state.current_mob.is_some());
        assert!(!state.current_mob.as_ref().unwrap().is_dead());
    }

    #[test]
    fn execute_attack_stops_spawning_when_cap_reached() {
        let (mut state, _) = GameState::new_game();
        state.current_area.base_encounter_amount = 2;

        state.execute_attack();
        state.advance_encounter();
        assert_eq!(state.encounters_cleared, 1);
        assert!(state.current_mob.is_some());

        state.execute_attack();
        state.advance_encounter();
        assert_eq!(state.encounters_cleared, 2);
        assert!(state.current_mob.is_none());
    }

    #[test]
    fn execute_attack_ignored_when_no_mob() {
        let (mut state, _) = GameState::new_game();
        state.current_mob = None;
        let result = state.execute_attack();
        assert!(!result);
        assert_eq!(state.encounters_cleared, 0);
    }

    #[test]
    fn overkill_damage_clamps_at_zero() {
        let (mut state, _) = GameState::new_game();
        if let Some(mob) = &mut state.current_mob {
            mob.health = 1;
        }
        state.execute_attack();
        assert_eq!(state.current_mob.as_ref().unwrap().health, 0);

        state.advance_encounter();
        assert_eq!(state.encounters_cleared, 1);
        assert!(state.current_mob.is_some());
    }

    #[test]
    fn beach_boss_triggers_fruit_scene_not_town() {
        let (mut state, _) = GameState::new_game();
        state.is_boss_encounter = true;
        state.current_area.id = "the_beach".to_string();
        if let Some(mob) = &mut state.current_mob {
            mob.health = 0;
        }

        let advanced = state.advance_encounter();
        assert!(advanced);
        assert!(state.fruit_scene_active);
        assert!(!state.in_town);
        assert_eq!(state.pending_fruit_id, Some("fruit_of_instinct".to_string()));
    }

    #[test]
    fn standard_mob_kill_on_beach_does_not_trigger_fruit_scene() {
        let (mut state, _) = GameState::new_game();
        state.is_boss_encounter = false;
        if let Some(mob) = &mut state.current_mob {
            mob.health = 0;
        }

        state.advance_encounter();
        assert!(!state.fruit_scene_active);
        assert!(state.pending_fruit_id.is_none());
    }

    #[test]
    fn complete_fruit_scene_eats_fruit_and_clears_flag() {
        let (mut state, _) = GameState::new_game();
        state.fruit_scene_active = true;
        state.pending_fruit_id = Some("fruit_of_instinct".to_string());

        state.complete_fruit_scene();
        assert!(!state.fruit_scene_active);
        assert!(state.pending_fruit_id.is_none());
        assert!(state.player.has_auto_combat());
        assert_eq!(state.player.actions.len(), 1);
    }

    #[test]
    fn complete_fruit_scene_noop_when_not_active() {
        let (mut state, _) = GameState::new_game();
        assert!(!state.fruit_scene_active);
        state.complete_fruit_scene();
        assert!(!state.player.has_auto_combat());
        assert!(state.player.actions.is_empty());
    }

    #[test]
    fn enter_area_transitions_correctly() {
        let (mut state, _) = GameState::new_game();
        state.encounters_cleared = 5;
        state.in_town = true;

        let success = state.enter_area("the_fringe");
        assert!(success);
        assert_eq!(state.current_area.id, "the_fringe");
        assert_eq!(state.current_area.name, "The Fringe");
        assert_eq!(state.encounters_cleared, 0);
        assert!(state.current_mob.is_some());
        assert!(!state.in_town);
    }

    #[test]
    fn enter_area_fails_for_invalid_id() {
        let (mut state, _) = GameState::new_game();
        let success = state.enter_area("nonexistent_area");
        assert!(!success);
        assert_eq!(state.current_area.id, "the_beach");
    }

    #[test]
    fn fringe_boss_triggers_town() {
        let (mut state, _) = GameState::new_game();
        state.current_area = Area::get_by_id("the_fringe").unwrap();
        state.is_boss_encounter = true;
        state.player.eat_fruit("fruit_of_instinct");
        if let Some(mob) = &mut state.current_mob {
            mob.health = 0;
        }

        let advanced = state.advance_encounter();
        assert!(advanced);
        assert!(state.in_town);
        assert!(!state.fruit_scene_active);
    }

    #[test]
    fn beach_boss_with_auto_combat_triggers_town() {
        let (mut state, _) = GameState::new_game();
        state.is_boss_encounter = true;
        state.current_area.id = "the_beach".to_string();
        state.player.eat_fruit("fruit_of_instinct");
        if let Some(mob) = &mut state.current_mob {
            mob.health = 0;
        }

        let advanced = state.advance_encounter();
        assert!(advanced);
        assert!(state.in_town);
        assert!(!state.fruit_scene_active);
    }

    #[test]
    fn enter_boss_portal_spawns_boss() {
        let (mut state, mut rng) = GameState::new_game();
        state.current_area.bosses = vec!["rat_lord".to_string()];
        state.current_area.base_encounter_amount = 0;
        state.encounters_cleared = 0;

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
            mob.health = 0;
        }

        state.advance_encounter();
        assert!(!state.in_town);
    }

    #[test]
    fn boss_portal_cannot_be_entered_early() {
        let (mut state, mut rng) = GameState::new_game();
        state.current_area.base_encounter_amount = 5;
        state.encounters_cleared = 2;

        let success = state.enter_boss_portal(&mut rng);
        assert!(!success);
        assert!(!state.is_boss_encounter);
    }
}
