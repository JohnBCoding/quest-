use serde::{Deserialize, Serialize};

use crate::action::Action;

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
}
