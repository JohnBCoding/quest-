use serde::{Deserialize, Serialize};

use crate::area::Area;
use crate::equipment::EquipmentSlot;
use crate::mob::Mob;
use crate::player::Player;
use crate::rng::{RngManager, RngSnapshot};

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutedPlayerAction {
    Attack,
    HealthPotion { healed: u32 },
}

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
    #[serde(default)]
    pub equipment_scene_active: bool,
    #[serde(default)]
    pub pending_equipment_id: Option<String>,
    #[serde(default)]
    pub pending_town_after_inventory: bool,
    #[serde(default)]
    pub split_hilt_scene_seen: bool,
    #[serde(default)]
    pub action_counter: u32,
    #[serde(default)]
    pub portals_unlocked: bool,
    pub version: u32,
}

pub const SAVE_VERSION: u32 = 3;

impl GameState {
    fn next_standard_mob_for_area(area: &Area, encounter_index: u32) -> Option<Mob> {
        if area.mobs.is_empty() {
            return Mob::get_by_id("rat");
        }

        let idx = (encounter_index as usize) % area.mobs.len();
        Mob::get_by_id(&area.mobs[idx])
    }

    pub fn new_game() -> (Self, RngManager) {
        let rng_manager = RngManager::new();
        let current_area = Area::starting_area();
        let state = Self {
            player: Player::default(),
            current_mob: Self::next_standard_mob_for_area(&current_area, 0),
            current_area,
            encounters_cleared: 0,
            rng_snapshot: rng_manager.snapshot(),
            is_boss_encounter: false,
            in_town: false,
            fruit_scene_active: false,
            pending_fruit_id: None,
            equipment_scene_active: false,
            pending_equipment_id: None,
            pending_town_after_inventory: false,
            split_hilt_scene_seen: false,
            action_counter: 0,
            portals_unlocked: false,
            version: SAVE_VERSION,
        };
        (state, rng_manager)
    }

    pub fn serialize(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn deserialize(data: &str) -> Result<Self, String> {
        let mut state: GameState =
            serde_json::from_str(data).map_err(|e| format!("Invalid save data: {}", e))?;

        if state.version != SAVE_VERSION {
            return Err(format!(
                "Incompatible save version: expected {}, got {}",
                SAVE_VERSION, state.version
            ));
        }

        state.player.ensure_auto_combat_actions();
        if state.in_town {
            state.player.refill_health_potions();
            state.portals_unlocked = true;
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

    fn execute_attack_with_damage(&mut self, damage: u32) -> bool {
        if let Some(mob) = self.current_mob.as_mut() {
            let was_alive = !mob.is_dead();
            mob.take_damage(damage);
            if was_alive && mob.is_dead() {
                self.player.gain_experience(mob.base_xp);
            }
            true
        } else {
            false
        }
    }

    pub fn execute_attack(&mut self) -> bool {
        let (min_damage, _max_damage) = self.player.attack_damage_range();
        self.execute_attack_with_damage(min_damage)
    }

    pub fn execute_attack_with_rng(&mut self, rng: &mut RngManager) -> bool {
        let (min_damage, max_damage) = self.player.attack_damage_range();
        let damage = rng.gen_range("combat", min_damage, max_damage);
        let did_attack = self.execute_attack_with_damage(damage);
        self.sync_rng(rng);
        did_attack
    }

    pub fn execute_prioritized_action(&mut self) -> Option<ExecutedPlayerAction> {
        let mob = self.current_mob.as_ref()?;
        if mob.is_dead() || !self.player.is_alive() {
            return None;
        }

        let next_action_number = self.action_counter.saturating_add(1);
        let actions = self.player.actions.clone();

        for action in actions {
            if !action.trigger_matches(next_action_number) {
                continue;
            }

            match action.id.as_str() {
                "health_potion" => {
                    let threshold = action.health_threshold_percent().unwrap_or(50);
                    if let Some(healed) = self.player.use_health_potion(threshold) {
                        self.action_counter = next_action_number;
                        return Some(ExecutedPlayerAction::HealthPotion { healed });
                    }
                }
                "attack" => {
                    if self.execute_attack() {
                        self.action_counter = next_action_number;
                        return Some(ExecutedPlayerAction::Attack);
                    }
                }
                _ => {}
            }
        }

        None
    }

    pub fn execute_prioritized_action_with_rng(
        &mut self,
        rng: &mut RngManager,
    ) -> Option<ExecutedPlayerAction> {
        let mob = self.current_mob.as_ref()?;
        if mob.is_dead() || !self.player.is_alive() {
            return None;
        }

        let next_action_number = self.action_counter.saturating_add(1);
        let actions = self.player.actions.clone();

        for action in actions {
            if !action.trigger_matches(next_action_number) {
                continue;
            }

            match action.id.as_str() {
                "health_potion" => {
                    let threshold = action.health_threshold_percent().unwrap_or(50);
                    if let Some(healed) = self.player.use_health_potion(threshold) {
                        self.action_counter = next_action_number;
                        return Some(ExecutedPlayerAction::HealthPotion { healed });
                    }
                }
                "attack" => {
                    if self.execute_attack_with_rng(rng) {
                        self.action_counter = next_action_number;
                        return Some(ExecutedPlayerAction::Attack);
                    }
                }
                _ => {}
            }
        }

        None
    }

    fn execute_mob_attack_with_damage(&mut self, damage: u32) -> Option<u32> {
        let mob = self.current_mob.as_ref()?;
        if mob.is_dead() || !self.player.is_alive() {
            return None;
        }
        self.player.take_damage(damage);
        Some(damage)
    }

    pub fn execute_mob_attack(&mut self) -> Option<u32> {
        let mob = self.current_mob.as_ref()?;
        let (min_damage, _max_damage) = mob.damage_range();
        self.execute_mob_attack_with_damage(min_damage)
    }

    pub fn execute_mob_attack_with_rng(&mut self, rng: &mut RngManager) -> Option<u32> {
        let mob = self.current_mob.as_ref()?;
        let (min_damage, max_damage) = mob.damage_range();
        let damage = rng.gen_range("combat", min_damage, max_damage);
        let hit = self.execute_mob_attack_with_damage(damage);
        self.sync_rng(rng);
        hit
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
                        self.current_mob = Self::next_standard_mob_for_area(
                            &self.current_area,
                            self.encounters_cleared,
                        );
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
        let boss_id = self
            .current_mob
            .as_ref()
            .map(|mob| mob.id.as_str())
            .unwrap_or("");
        if boss_id == "rat_face" {
            if !self.split_hilt_scene_seen {
                self.equipment_scene_active = true;
                self.pending_equipment_id = Some("split_hilt_blade".to_string());
                self.split_hilt_scene_seen = true;
                return;
            }
            self.player.add_equipment_item("split_hilt_blade");
        }

        if self.current_area.id == "the_beach" && !self.player.has_auto_combat() {
            self.fruit_scene_active = true;
            self.pending_fruit_id = Some("fruit_of_instinct".to_string());
        } else {
            self.enter_town();
        }
    }

    fn enter_town(&mut self) {
        self.in_town = true;
        self.portals_unlocked = true;
        self.player.refill_health_potions();
    }

    pub fn portal_to_town(&mut self) -> bool {
        if self.in_town {
            return false;
        }
        self.current_mob = None;
        self.is_boss_encounter = false;
        self.enter_town();
        true
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

    pub fn complete_equipment_scene(&mut self) {
        if !self.equipment_scene_active {
            return;
        }

        if let Some(item_id) = self.pending_equipment_id.take() {
            self.player.add_equipment_item(&item_id);
            let _ = self
                .player
                .equip_item_to_slot(&item_id, EquipmentSlot::MainHand);
        }

        self.equipment_scene_active = false;
        self.pending_town_after_inventory = true;
    }

    pub fn finish_first_inventory_visit(&mut self) -> bool {
        if !self.pending_town_after_inventory {
            return false;
        }

        self.pending_town_after_inventory = false;
        self.enter_town();
        true
    }

    pub fn enter_area(&mut self, area_id: &str) -> bool {
        if let Some(area) = Area::get_by_id(area_id) {
            self.current_area = area;
            self.encounters_cleared = 0;
            self.current_mob = Self::next_standard_mob_for_area(&self.current_area, 0);
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
    use crate::action::{Action, ActionCondition, ActionTrigger};

    #[test]
    fn new_game_creates_valid_state() {
        let (state, _rng) = GameState::new_game();
        assert_eq!(state.player.name, "Hero");
        assert_eq!(state.current_area.name, "The Beach");
        assert_eq!(state.version, SAVE_VERSION);
        assert!(!state.fruit_scene_active);
        assert!(state.pending_fruit_id.is_none());
        assert!(!state.equipment_scene_active);
        assert!(state.pending_equipment_id.is_none());
        assert!(!state.pending_town_after_inventory);
        assert!(!state.split_hilt_scene_seen);
        assert_eq!(state.action_counter, 0);
        assert!(!state.portals_unlocked);
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
        assert_eq!(state.current_mob.unwrap().health, 4);
        assert_eq!(state.encounters_cleared, 0);
        assert_eq!(state.player.experience, 0);
    }

    #[test]
    fn execute_attack_kills_mob_and_increments_encounters() {
        let (mut state, _) = GameState::new_game();
        state.current_area.base_encounter_amount = 10;
        if let Some(mob) = &mut state.current_mob {
            mob.health = 1;
        }
        let expected_xp = state.current_mob.as_ref().map(|m| m.base_xp).unwrap_or(0);
        let result = state.execute_attack();
        assert!(result);
        assert!(state.current_mob.as_ref().unwrap().is_dead());
        assert_eq!(state.encounters_cleared, 0);
        assert_eq!(state.player.experience, expected_xp);

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

        if let Some(mob) = &mut state.current_mob {
            mob.health = 1;
        }
        state.execute_attack();
        state.advance_encounter();
        assert_eq!(state.encounters_cleared, 1);
        assert!(state.current_mob.is_some());

        if let Some(mob) = &mut state.current_mob {
            mob.health = 1;
        }
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
    fn execute_mob_attack_reduces_player_health() {
        let (mut state, _) = GameState::new_game();
        state.current_mob = Mob::get_by_id("rat_lord");
        state.player.health = 10;
        let expected_damage = state
            .current_mob
            .as_ref()
            .map(|mob| mob.damage_range().0)
            .unwrap_or(0);
        let damage = state.execute_mob_attack();
        assert_eq!(damage, Some(expected_damage));
        assert_eq!(state.player.health, 10u32.saturating_sub(expected_damage));
    }

    #[test]
    fn execute_mob_attack_ignored_when_no_mob() {
        let (mut state, _) = GameState::new_game();
        state.current_mob = None;
        let damage = state.execute_mob_attack();
        assert_eq!(damage, None);
    }

    #[test]
    fn execute_attack_with_rng_rolls_within_player_range() {
        let (mut state, mut rng) = GameState::new_game();
        if let Some(mob) = &mut state.current_mob {
            mob.health = 10;
        }
        let hp_before = state.current_mob.as_ref().unwrap().health;

        let attacked = state.execute_attack_with_rng(&mut rng);
        assert!(attacked);

        let hp_after = state.current_mob.as_ref().unwrap().health;
        let dealt = hp_before.saturating_sub(hp_after);
        assert!((1..=2).contains(&dealt));
    }

    #[test]
    fn execute_mob_attack_with_rng_rolls_within_mob_range() {
        let (mut state, mut rng) = GameState::new_game();
        state.current_mob = Mob::get_by_id("rat_face");
        state.player.health = 50;
        let health_before = state.player.health;

        let dealt = state.execute_mob_attack_with_rng(&mut rng).unwrap();
        assert!((2..=5).contains(&dealt));
        assert_eq!(state.player.health, health_before.saturating_sub(dealt));
    }

    #[test]
    fn overkill_damage_clamps_at_zero() {
        let (mut state, _) = GameState::new_game();
        state.current_area.base_encounter_amount = 10;
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
    fn boss_kill_grants_base_experience() {
        let (mut state, _) = GameState::new_game();
        state.current_mob = Mob::get_by_id("rat_lord");
        state.is_boss_encounter = true;
        state.player.max_experience = 1_000_000;
        let expected_xp = state.current_mob.as_ref().map(|m| m.base_xp).unwrap_or(0);

        if let Some(mob) = &mut state.current_mob {
            mob.health = 1;
        }

        state.execute_attack();
        let advanced = state.advance_encounter();
        assert!(advanced);
        assert_eq!(state.player.experience, expected_xp);
    }

    #[test]
    fn player_levels_up_when_xp_threshold_is_met_on_mob_death() {
        let (mut state, _) = GameState::new_game();
        state.player.experience = 240;
        state.current_area.base_encounter_amount = 10;
        if let Some(mob) = &mut state.current_mob {
            mob.health = 1;
            mob.base_xp = 10;
        }

        state.execute_attack();
        let advanced = state.advance_encounter();
        assert!(advanced);
        assert_eq!(state.player.level, 2);
        assert_eq!(state.player.experience, 0);
        assert_eq!(state.player.max_experience, 400);
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
        assert_eq!(
            state.pending_fruit_id,
            Some("fruit_of_instinct".to_string())
        );
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
        assert_eq!(state.player.actions.len(), 2);
        assert_eq!(state.player.actions[0].id, "health_potion");
        assert_eq!(state.player.actions[1].id, "attack");
        assert_eq!(state.player.health_potion_uses, 5);
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
        assert!(state.portals_unlocked);
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
    fn rat_face_first_boss_kill_triggers_equipment_scene() {
        let (mut state, _) = GameState::new_game();
        state.current_area = Area::get_by_id("the_fringe").unwrap();
        state.current_mob = Mob::get_by_id("rat_face");
        state.is_boss_encounter = true;
        if let Some(mob) = &mut state.current_mob {
            mob.health = 0;
        }

        let advanced = state.advance_encounter();
        assert!(advanced);
        assert!(state.equipment_scene_active);
        assert_eq!(
            state.pending_equipment_id,
            Some("split_hilt_blade".to_string())
        );
        assert!(!state.in_town);
    }

    #[test]
    fn complete_equipment_scene_equips_item_and_sets_inventory_flag() {
        let (mut state, _) = GameState::new_game();
        state.equipment_scene_active = true;
        state.pending_equipment_id = Some("split_hilt_blade".to_string());

        state.complete_equipment_scene();
        assert!(!state.equipment_scene_active);
        assert!(state.pending_equipment_id.is_none());
        assert_eq!(
            state.player.equipped_main_hand,
            Some("split_hilt_blade".to_string())
        );
        assert!(state.pending_town_after_inventory);
    }

    #[test]
    fn finish_first_inventory_visit_enters_town_once() {
        let (mut state, _) = GameState::new_game();
        state.pending_town_after_inventory = true;

        let first = state.finish_first_inventory_visit();
        let second = state.finish_first_inventory_visit();

        assert!(first);
        assert!(!second);
        assert!(state.in_town);
        assert!(!state.pending_town_after_inventory);
    }

    #[test]
    fn rat_face_subsequent_kill_drops_item_without_scene() {
        let (mut state, _) = GameState::new_game();
        state.current_area = Area::get_by_id("the_fringe").unwrap();
        state.current_mob = Mob::get_by_id("rat_face");
        state.is_boss_encounter = true;
        state.split_hilt_scene_seen = true;
        if let Some(mob) = &mut state.current_mob {
            mob.health = 0;
        }

        state.advance_encounter();
        assert!(!state.equipment_scene_active);
        assert!(state.pending_equipment_id.is_none());
        assert!(state.in_town);
        assert_eq!(
            state.player.equipment_inventory,
            vec!["split_hilt_blade".to_string()]
        );
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

    #[test]
    fn prioritized_action_uses_potion_before_attack_when_low_health() {
        let (mut state, _) = GameState::new_game();
        state.player.eat_fruit("fruit_of_instinct");
        state.player.health = 20;
        state.current_mob = Mob::get_by_id("rat_lord");
        let mob_hp_before = state.current_mob.as_ref().unwrap().health;

        let result = state.execute_prioritized_action();
        assert_eq!(
            result,
            Some(ExecutedPlayerAction::HealthPotion { healed: 25 })
        );
        assert_eq!(state.player.health, 45);
        assert_eq!(state.player.health_potion_uses, 4);
        assert_eq!(state.current_mob.as_ref().unwrap().health, mob_hp_before);
        assert_eq!(state.action_counter, 1);
    }

    #[test]
    fn prioritized_action_falls_through_to_attack_when_potion_not_eligible() {
        let (mut state, _) = GameState::new_game();
        state.player.eat_fruit("fruit_of_instinct");
        state.player.health = 30;
        state.current_mob = Mob::get_by_id("rat_lord");
        let mob_hp_before = state.current_mob.as_ref().unwrap().health;

        let result = state.execute_prioritized_action();
        assert_eq!(result, Some(ExecutedPlayerAction::Attack));
        assert_eq!(
            state.current_mob.as_ref().unwrap().health,
            mob_hp_before - 1
        );
        assert_eq!(state.player.health_potion_uses, 5);
        assert_eq!(state.action_counter, 1);
    }

    #[test]
    fn prioritized_action_falls_through_when_potion_empty() {
        let (mut state, _) = GameState::new_game();
        state.player.eat_fruit("fruit_of_instinct");
        state.player.health = 10;
        state.player.health_potion_uses = 0;
        state.current_mob = Mob::get_by_id("rat_lord");
        let mob_hp_before = state.current_mob.as_ref().unwrap().health;

        let result = state.execute_prioritized_action();
        assert_eq!(result, Some(ExecutedPlayerAction::Attack));
        assert_eq!(
            state.current_mob.as_ref().unwrap().health,
            mob_hp_before - 1
        );
    }

    #[test]
    fn town_entry_refills_potions() {
        let (mut state, _) = GameState::new_game();
        state.current_area = Area::get_by_id("the_fringe").unwrap();
        state.is_boss_encounter = true;
        state.player.eat_fruit("fruit_of_instinct");
        state.player.health_potion_uses = 1;
        if let Some(mob) = &mut state.current_mob {
            mob.health = 0;
        }

        state.advance_encounter();
        assert!(state.in_town);
        assert_eq!(state.player.health_potion_uses, 5);
    }

    #[test]
    fn portal_to_town_unlocks_portals_and_clears_encounter_state() {
        let (mut state, _) = GameState::new_game();
        state.current_area = Area::get_by_id("the_fringe").unwrap();
        state.current_mob = Mob::get_by_id("rat_lord");
        state.is_boss_encounter = true;
        state.player.eat_fruit("fruit_of_instinct");
        state.player.health_potion_uses = 2;

        let success = state.portal_to_town();
        assert!(success);
        assert!(state.in_town);
        assert!(state.portals_unlocked);
        assert!(state.current_mob.is_none());
        assert!(!state.is_boss_encounter);
        assert_eq!(state.player.health_potion_uses, 5);
    }

    #[test]
    fn deserialize_old_town_save_unlocks_portals() {
        let old_town_save = r#"{
            "player": {
                "name":"Hero",
                "level":1,
                "health":50,
                "max_health":50,
                "experience":0,
                "max_experience":250,
                "eaten_fruits":["fruit_of_instinct"],
                "actions":[{"id":"attack","name":"Attack","trigger":"EveryAction"}],
                "action_speed_ms":1000
            },
            "current_area": {
                "id":"the_fringe",
                "name":"The Fringe",
                "description":"x",
                "required_level":1,
                "base_encounter_amount":1,
                "bosses":["rat_lord"]
            },
            "current_mob": null,
            "encounters_cleared":0,
            "rng_snapshot":{"seeds":{"mob_spawns":1}},
            "is_boss_encounter":false,
            "in_town":true,
            "fruit_scene_active":false,
            "pending_fruit_id":null,
            "version":3
        }"#;

        let state = GameState::deserialize(old_town_save).unwrap();
        assert!(state.in_town);
        assert!(state.portals_unlocked);
    }

    #[test]
    fn deserialize_migrates_old_auto_combat_layout() {
        let old_save = r#"{
            "player": {
                "name":"Hero",
                "level":1,
                "health":50,
                "max_health":50,
                "experience":0,
                "max_experience":250,
                "eaten_fruits":["fruit_of_instinct"],
                "actions":[{"id":"attack","name":"Attack","trigger":"EveryAction"}],
                "action_speed_ms":1000
            },
            "current_area": {
                "id":"the_beach",
                "name":"The Beach",
                "description":"x",
                "required_level":1,
                "base_encounter_amount":1,
                "bosses":["rat_lord"]
            },
            "current_mob": null,
            "encounters_cleared":0,
            "rng_snapshot":{"seeds":{"mob_spawns":1}},
            "is_boss_encounter":false,
            "in_town":false,
            "fruit_scene_active":false,
            "pending_fruit_id":null,
            "version":3
        }"#;

        let state = GameState::deserialize(old_save).unwrap();
        assert_eq!(state.player.actions.len(), 2);
        assert_eq!(state.player.actions[0].id, "health_potion");
        assert_eq!(state.player.actions[1].id, "attack");
        assert_eq!(
            state.player.actions[0].condition,
            ActionCondition::HealthBelowPercent(50)
        );
        assert_eq!(state.player.health_potion_uses, 5);
        assert_eq!(state.player.health_potion_capacity, 5);
    }

    #[test]
    fn prioritized_action_respects_trigger_frequency() {
        let (mut state, _) = GameState::new_game();
        state.current_mob = Mob::get_by_id("rat_lord");
        state.player.actions = vec![
            Action {
                id: "health_potion".to_string(),
                name: "Health Potion".to_string(),
                trigger: ActionTrigger::EveryNActions(2),
                condition: ActionCondition::HealthBelowPercent(90),
            },
            Action::default_attack(),
        ];
        state.player.health_potion_uses = 5;
        state.player.health = 10;

        let first = state.execute_prioritized_action();
        assert_eq!(first, Some(ExecutedPlayerAction::Attack));
        let second = state.execute_prioritized_action();
        assert_eq!(
            second,
            Some(ExecutedPlayerAction::HealthPotion { healed: 25 })
        );
    }
}
