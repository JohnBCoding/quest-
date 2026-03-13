use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an enemy or NPC encountered in an area.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mob {
    pub id: String,
    pub name: String,
    pub health: u32,
    pub max_health: u32,
    #[serde(default)]
    pub base_xp: u64,
    #[serde(default)]
    pub base_damage: u32,
    #[serde(default = "default_action_speed")]
    pub action_speed_ms: u32,
}

fn default_action_speed() -> u32 {
    1000
}

pub static MOB_REGISTRY: Lazy<HashMap<String, Mob>> = Lazy::new(|| {
    let json_data = include_str!("../data/mobs.json");
    // Initial deserialize doesn't have max_health populated from JSON
    #[derive(Deserialize)]
    struct MobData {
        id: String,
        name: String,
        health: u32,
        #[serde(default)]
        base_xp: u64,
        base_damage: u32,
        action_speed_ms: u32,
    }
    let parsed: Vec<MobData> = serde_json::from_str(json_data).expect("Failed to parse mobs.json");
    let mut registry = HashMap::new();
    for data in parsed {
        registry.insert(
            data.id.clone(),
            Mob {
                id: data.id,
                name: data.name,
                health: data.health,
                max_health: data.health, // Initialize max_health to base health
                base_xp: data.base_xp,
                base_damage: data.base_damage,
                action_speed_ms: data.action_speed_ms,
            },
        );
    }
    registry
});

impl Mob {
    /// Retrieves a Mob by its ID from the static registry.
    pub fn get_by_id(id: &str) -> Option<Self> {
        MOB_REGISTRY.get(id).cloned()
    }

    /// Creates a new Mob with the given name and health.
    pub fn new(
        id: &str,
        name: &str,
        health: u32,
        base_xp: u64,
        base_damage: u32,
        action_speed_ms: u32,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            health,
            max_health: health,
            base_xp,
            base_damage,
            action_speed_ms,
        }
    }

    /// Reduces the mob's health by the given amount, clamping at 0.
    pub fn take_damage(&mut self, amount: u32) {
        self.health = self.health.saturating_sub(amount);
    }

    /// Returns true if the mob is dead (health == 0).
    pub fn is_dead(&self) -> bool {
        self.health == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_loads_successfully() {
        assert!(!MOB_REGISTRY.is_empty(), "Mob registry should not be empty");
    }

    #[test]
    fn mob_registry_loads_properly() {
        let rat = Mob::get_by_id("rat").expect("Rat should be loaded from registry");
        assert_eq!(rat.name, "Rat");
        assert_eq!(rat.max_health, 2);
        assert!(rat.base_xp > 0);
        assert_eq!(rat.base_damage, 0);
        assert_eq!(rat.action_speed_ms, 2000);
    }

    #[test]
    fn mob_creation() {
        let mob = Mob::new("rat", "Rat", 2, 10, 0, 1000);
        assert_eq!(mob.id, "rat");
        assert_eq!(mob.name, "Rat");
        assert_eq!(mob.health, 2);
        assert_eq!(mob.max_health, 2);
        assert_eq!(mob.base_xp, 10);
        assert_eq!(mob.base_damage, 0);
        assert_eq!(mob.action_speed_ms, 1000);
        assert!(!mob.is_dead());
    }

    #[test]
    fn take_damage() {
        let mut mob = Mob::new("rat", "Rat", 2, 10, 0, 1000);
        mob.take_damage(1);
        assert_eq!(mob.health, 1);
        assert!(!mob.is_dead());

        mob.take_damage(2); // Overkill
        assert_eq!(mob.health, 0);
        assert!(mob.is_dead());
    }
}
