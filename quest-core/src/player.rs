use serde::{Deserialize, Serialize};

use crate::action::Action;
use crate::equipment::{EquipmentItem, EquipmentSlot};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Player {
    pub name: String,
    pub level: u32,
    pub health: u32,
    pub max_health: u32,
    pub experience: u64,
    #[serde(default = "default_max_experience")]
    pub max_experience: u64,
    #[serde(default)]
    pub eaten_fruits: Vec<String>,
    #[serde(default)]
    pub actions: Vec<Action>,
    #[serde(default = "default_action_speed")]
    pub action_speed_ms: u32,
    #[serde(default)]
    pub health_potion_uses: u32,
    #[serde(default = "default_health_potion_capacity")]
    pub health_potion_capacity: u32,
    #[serde(default)]
    pub equipment_inventory: Vec<String>,
    #[serde(default)]
    pub equipped_main_hand: Option<String>,
    #[serde(default)]
    pub equipped_off_hand: Option<String>,
}

fn default_action_speed() -> u32 {
    1000
}

fn default_max_experience() -> u64 {
    250
}

fn default_health_potion_capacity() -> u32 {
    5
}

fn default_fist_damage_min() -> u32 {
    1
}

fn default_fist_damage_max() -> u32 {
    2
}

impl Player {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            level: 1,
            health: 50,
            max_health: 50,
            experience: 0,
            max_experience: default_max_experience(),
            eaten_fruits: Vec::new(),
            actions: Vec::new(),
            action_speed_ms: 1000,
            health_potion_uses: 0,
            health_potion_capacity: default_health_potion_capacity(),
            equipment_inventory: Vec::new(),
            equipped_main_hand: None,
            equipped_off_hand: None,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0
    }

    pub fn has_eaten_fruit(&self, id: &str) -> bool {
        self.eaten_fruits.iter().any(|f| f == id)
    }

    pub fn has_auto_combat(&self) -> bool {
        self.has_eaten_fruit("fruit_of_instinct")
    }

    pub fn has_action(&self, action_id: &str) -> bool {
        self.actions.iter().any(|a| a.id == action_id)
    }

    pub fn take_damage(&mut self, amount: u32) {
        self.health = self.health.saturating_sub(amount);
    }

    pub fn heal(&mut self, amount: u32) -> u32 {
        let before = self.health;
        self.health = self.health.saturating_add(amount).min(self.max_health);
        self.health.saturating_sub(before)
    }

    pub fn gain_experience(&mut self, amount: u64) -> bool {
        if amount == 0 {
            return false;
        }

        self.experience = self.experience.saturating_add(amount);
        if self.experience >= self.max_experience {
            self.level_up();
            return true;
        }

        false
    }

    pub fn eat_fruit(&mut self, fruit_id: &str) {
        if self.has_eaten_fruit(fruit_id) {
            return;
        }
        self.eaten_fruits.push(fruit_id.to_string());
        self.apply_fruit_effect(fruit_id);
    }

    pub fn ensure_auto_combat_actions(&mut self) {
        if !self.has_auto_combat() {
            return;
        }

        let had_health_potion = self.has_action("health_potion");
        let attack_idx = self.actions.iter().position(|a| a.id == "attack");

        if !had_health_potion {
            let potion = Action::default_health_potion();
            if let Some(idx) = attack_idx {
                self.actions.insert(idx, potion);
            } else {
                self.actions.push(potion);
            }
        }

        if attack_idx.is_none() {
            self.actions.push(Action::default_attack());
        }

        if !had_health_potion {
            self.refill_health_potions();
        }
    }

    pub fn refill_health_potions(&mut self) {
        if self.has_action("health_potion") {
            self.health_potion_uses = self.health_potion_capacity;
        }
    }

    pub fn can_use_health_potion(&self, threshold_percent: u32) -> bool {
        if self.health_potion_uses == 0 || self.max_health == 0 {
            return false;
        }

        let threshold_hp =
            ((self.max_health as f64) * (threshold_percent as f64 / 100.0)).floor() as u32;
        self.health < threshold_hp
    }

    pub fn use_health_potion(&mut self, threshold_percent: u32) -> Option<u32> {
        if !self.can_use_health_potion(threshold_percent) {
            return None;
        }

        let heal_amount = ((self.max_health as f64) * 0.5).ceil() as u32;
        let healed = self.heal(heal_amount.max(1));
        if healed == 0 {
            return None;
        }

        self.health_potion_uses = self.health_potion_uses.saturating_sub(1);
        Some(healed)
    }

    fn level_up(&mut self) {
        self.level = self.level.saturating_add(1);
        self.experience = 0;

        let growth = ((self.max_experience as f64 * self.level as f64) * 0.3).round() as u64;
        self.max_experience = self.max_experience.saturating_add(growth.max(1));
    }

    fn apply_fruit_effect(&mut self, fruit_id: &str) {
        match fruit_id {
            "fruit_of_instinct" => {
                self.ensure_auto_combat_actions();
            }
            _ => {}
        }
    }

    pub fn add_equipment_item(&mut self, item_id: &str) {
        self.equipment_inventory.push(item_id.to_string());
    }

    pub fn list_equipment_inventory_items(&self) -> Vec<EquipmentItem> {
        self.equipment_inventory
            .iter()
            .filter_map(|id| EquipmentItem::get_by_id(id))
            .collect()
    }

    pub fn equipped_item(&self, slot: EquipmentSlot) -> Option<EquipmentItem> {
        let equipped_id = match slot {
            EquipmentSlot::MainHand => self.equipped_main_hand.as_deref(),
            EquipmentSlot::OffHand => self.equipped_off_hand.as_deref(),
        }?;

        EquipmentItem::get_by_id(equipped_id)
    }

    pub fn equip_item_to_slot(&mut self, item_id: &str, slot: EquipmentSlot) -> bool {
        let Some(item) = EquipmentItem::get_by_id(item_id) else {
            return false;
        };
        if !item.can_equip_in(slot) {
            return false;
        }

        let Some(inventory_idx) = self.equipment_inventory.iter().position(|id| id == item_id)
        else {
            return false;
        };

        self.equipment_inventory.remove(inventory_idx);
        let replaced = match slot {
            EquipmentSlot::MainHand => self.equipped_main_hand.replace(item_id.to_string()),
            EquipmentSlot::OffHand => self.equipped_off_hand.replace(item_id.to_string()),
        };
        if let Some(item_id) = replaced {
            self.equipment_inventory.push(item_id);
        }
        true
    }

    pub fn unequip_slot(&mut self, slot: EquipmentSlot) -> bool {
        let equipped = match slot {
            EquipmentSlot::MainHand => self.equipped_main_hand.take(),
            EquipmentSlot::OffHand => self.equipped_off_hand.take(),
        };

        if let Some(item_id) = equipped {
            self.equipment_inventory.push(item_id);
            true
        } else {
            false
        }
    }

    pub fn attack_damage_range(&self) -> (u32, u32) {
        let main_hand = self.equipped_item(EquipmentSlot::MainHand);
        let off_hand = self.equipped_item(EquipmentSlot::OffHand);

        match (main_hand, off_hand) {
            (Some(main), Some(off)) => {
                let combined_min = main.min_damage.saturating_add(off.min_damage);
                let combined_max = main.max_damage.saturating_add(off.max_damage);
                (
                    combined_min.saturating_mul(75) / 100,
                    combined_max.saturating_mul(75) / 100,
                )
            }
            (Some(weapon), None) | (None, Some(weapon)) => (weapon.min_damage, weapon.max_damage),
            (None, None) => (default_fist_damage_min(), default_fist_damage_max()),
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new("Hero")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{ActionCondition, ActionTrigger};

    #[test]
    fn default_player_has_correct_name() {
        let player = Player::default();
        assert_eq!(player.name, "Hero");
    }

    #[test]
    fn default_player_has_starter_stats() {
        let player = Player::default();
        assert_eq!(player.level, 1);
        assert_eq!(player.health, 50);
        assert_eq!(player.max_health, 50);
        assert_eq!(player.experience, 0);
        assert_eq!(player.max_experience, 250);
        assert!(player.eaten_fruits.is_empty());
        assert!(player.actions.is_empty());
        assert_eq!(player.action_speed_ms, 1000);
        assert_eq!(player.health_potion_uses, 0);
        assert_eq!(player.health_potion_capacity, 5);
        assert!(player.equipment_inventory.is_empty());
        assert!(player.equipped_main_hand.is_none());
        assert!(player.equipped_off_hand.is_none());
    }

    #[test]
    fn default_player_is_alive() {
        let player = Player::default();
        assert!(player.is_alive());
    }

    #[test]
    fn custom_name_player() {
        let player = Player::new("Exile");
        assert_eq!(player.name, "Exile");
        assert_eq!(player.level, 1);
    }

    #[test]
    fn dead_player_is_not_alive() {
        let mut player = Player::default();
        player.health = 0;
        assert!(!player.is_alive());
    }

    #[test]
    fn take_damage_clamps_at_zero() {
        let mut player = Player::default();
        player.take_damage(5);
        assert_eq!(player.health, 45);
        player.take_damage(100);
        assert_eq!(player.health, 0);
    }

    #[test]
    fn gain_experience_increments_without_level_up() {
        let mut player = Player::default();
        let did_level = player.gain_experience(10);
        assert!(!did_level);
        assert_eq!(player.level, 1);
        assert_eq!(player.experience, 10);
        assert_eq!(player.max_experience, 250);
    }

    #[test]
    fn gain_experience_levels_up_at_threshold_and_resets_exp() {
        let mut player = Player::default();
        let did_level = player.gain_experience(250);
        assert!(did_level);
        assert_eq!(player.level, 2);
        assert_eq!(player.experience, 0);
        assert_eq!(player.max_experience, 400);
    }

    #[test]
    fn player_serialization_roundtrip() {
        let player = Player::default();
        let json = serde_json::to_string(&player).unwrap();
        let deserialized: Player = serde_json::from_str(&json).unwrap();
        assert_eq!(player, deserialized);
    }

    #[test]
    fn player_deserialization_rejects_invalid_json() {
        let result = serde_json::from_str::<Player>("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn player_deserialization_rejects_missing_fields() {
        let result = serde_json::from_str::<Player>(r#"{"name": "Hero"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn eat_fruit_adds_to_list_and_sets_up_action() {
        let mut player = Player::default();
        player.eat_fruit("fruit_of_instinct");
        assert!(player.has_eaten_fruit("fruit_of_instinct"));
        assert_eq!(player.actions.len(), 2);
        assert_eq!(player.actions[0].id, "health_potion");
        assert_eq!(player.actions[1].id, "attack");
        assert_eq!(player.actions[0].trigger, ActionTrigger::EveryAction);
        assert_eq!(
            player.actions[0].condition,
            ActionCondition::HealthBelowPercent(50)
        );
        assert_eq!(player.health_potion_uses, 5);
    }

    #[test]
    fn has_auto_combat_true_after_eating_instinct() {
        let mut player = Player::default();
        player.eat_fruit("fruit_of_instinct");
        assert!(player.has_auto_combat());
    }

    #[test]
    fn has_auto_combat_false_on_fresh_player() {
        let player = Player::default();
        assert!(!player.has_auto_combat());
    }

    #[test]
    fn has_eaten_fruit_false_for_uneaten() {
        let player = Player::default();
        assert!(!player.has_eaten_fruit("fruit_of_instinct"));
    }

    #[test]
    fn eating_same_fruit_twice_does_not_duplicate() {
        let mut player = Player::default();
        player.eat_fruit("fruit_of_instinct");
        player.eat_fruit("fruit_of_instinct");
        assert_eq!(player.eaten_fruits.len(), 1);
        assert_eq!(player.actions.len(), 2);
        assert_eq!(player.health_potion_uses, 5);
    }

    #[test]
    fn use_health_potion_heals_and_consumes_use() {
        let mut player = Player::default();
        player.eat_fruit("fruit_of_instinct");
        player.health = 10;
        let healed = player.use_health_potion(50);
        assert_eq!(healed, Some(25));
        assert_eq!(player.health, 35);
        assert_eq!(player.health_potion_uses, 4);
    }

    #[test]
    fn health_potion_does_not_trigger_at_or_above_threshold() {
        let mut player = Player::default();
        player.eat_fruit("fruit_of_instinct");
        player.health = 25;
        assert_eq!(player.use_health_potion(50), None);
        assert_eq!(player.health_potion_uses, 5);
    }

    #[test]
    fn refill_health_potions_restores_capacity() {
        let mut player = Player::default();
        player.eat_fruit("fruit_of_instinct");
        player.health_potion_uses = 1;
        player.refill_health_potions();
        assert_eq!(player.health_potion_uses, 5);
    }

    #[test]
    fn ensure_auto_combat_actions_migrates_old_layout() {
        let mut player = Player::default();
        player.eaten_fruits.push("fruit_of_instinct".to_string());
        player.actions.push(Action::default_attack());
        player.ensure_auto_combat_actions();
        assert_eq!(player.actions[0].id, "health_potion");
        assert_eq!(player.actions[1].id, "attack");
        assert_eq!(player.health_potion_uses, 5);
    }

    #[test]
    fn player_with_fruits_serialization_roundtrip() {
        let mut player = Player::default();
        player.eat_fruit("fruit_of_instinct");
        let json = serde_json::to_string(&player).unwrap();
        let deserialized: Player = serde_json::from_str(&json).unwrap();
        assert_eq!(player, deserialized);
    }

    #[test]
    fn attack_damage_defaults_to_fists() {
        let player = Player::default();
        assert_eq!(player.attack_damage_range(), (1, 2));
    }

    #[test]
    fn equip_item_to_main_hand_uses_weapon_range() {
        let mut player = Player::default();
        player.add_equipment_item("split_hilt_blade");
        let equipped = player.equip_item_to_slot("split_hilt_blade", EquipmentSlot::MainHand);

        assert!(equipped);
        assert_eq!(
            player.equipped_main_hand,
            Some("split_hilt_blade".to_string())
        );
        assert!(player.equipment_inventory.is_empty());
        assert_eq!(player.attack_damage_range(), (1, 4));
    }

    #[test]
    fn dual_wield_scales_combined_weapon_range() {
        let mut player = Player::default();
        player.add_equipment_item("split_hilt_blade");
        player.add_equipment_item("split_hilt_blade");
        assert!(player.equip_item_to_slot("split_hilt_blade", EquipmentSlot::MainHand));
        assert!(player.equip_item_to_slot("split_hilt_blade", EquipmentSlot::OffHand));
        assert_eq!(player.attack_damage_range(), (1, 6));
    }

    #[test]
    fn unequip_slot_returns_item_to_inventory() {
        let mut player = Player::default();
        player.add_equipment_item("split_hilt_blade");
        assert!(player.equip_item_to_slot("split_hilt_blade", EquipmentSlot::MainHand));
        assert!(player.unequip_slot(EquipmentSlot::MainHand));
        assert!(player.equipped_main_hand.is_none());
        assert_eq!(
            player.equipment_inventory,
            vec!["split_hilt_blade".to_string()]
        );
        assert_eq!(player.attack_damage_range(), (1, 2));
    }
}
