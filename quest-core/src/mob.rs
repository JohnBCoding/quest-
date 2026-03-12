use serde::{Deserialize, Serialize};

/// Represents an enemy or NPC encountered in an area.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mob {
    pub name: String,
    pub health: u32,
    pub max_health: u32,
}

impl Mob {
    /// Creates a new Mob with the given name and health.
    pub fn new(name: &str, health: u32) -> Self {
        Self {
            name: name.to_string(),
            health,
            max_health: health,
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
    fn mob_creation() {
        let mob = Mob::new("rat", 2);
        assert_eq!(mob.name, "rat");
        assert_eq!(mob.health, 2);
        assert_eq!(mob.max_health, 2);
        assert!(!mob.is_dead());
    }

    #[test]
    fn take_damage() {
        let mut mob = Mob::new("rat", 2);
        mob.take_damage(1);
        assert_eq!(mob.health, 1);
        assert!(!mob.is_dead());
        
        mob.take_damage(2); // Overkill
        assert_eq!(mob.health, 0);
        assert!(mob.is_dead());
    }
}
