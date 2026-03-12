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
}

fn default_action_speed() -> u32 {
    1000
}

fn default_max_experience() -> u64 {
    250
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

    pub fn take_damage(&mut self, amount: u32) {
        self.health = self.health.saturating_sub(amount);
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

    fn level_up(&mut self) {
        self.level = self.level.saturating_add(1);
        self.experience = 0;

        let growth = ((self.max_experience as f64 * self.level as f64) * 0.3).round() as u64;
        self.max_experience = self.max_experience.saturating_add(growth.max(1));
    }

    fn apply_fruit_effect(&mut self, fruit_id: &str) {
        match fruit_id {
            "fruit_of_instinct" => {
                if !self.actions.iter().any(|a| a.id == "attack") {
                    self.actions.push(Action::default_attack());
                }
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
    use crate::action::ActionTrigger;

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
        assert_eq!(player.actions.len(), 1);
        assert_eq!(player.actions[0].id, "attack");
        assert_eq!(player.actions[0].trigger, ActionTrigger::EveryAction);
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
        assert_eq!(player.actions.len(), 1);
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
