use serde::{Deserialize, Serialize};

/// Represents a player character in the game.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Player {
    pub name: String,
    pub level: u32,
    pub health: u32,
    pub max_health: u32,
    pub experience: u64,
}

impl Player {
    /// Creates a new player with custom name and default starter stats.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            level: 1,
            health: 50,
            max_health: 50,
            experience: 0,
        }
    }

    /// Returns true if the player is alive.
    pub fn is_alive(&self) -> bool {
        self.health > 0
    }
}

impl Default for Player {
    /// Creates the default "Hero" character.
    fn default() -> Self {
        Self::new("Hero")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
