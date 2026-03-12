use serde::{Deserialize, Serialize};

/// Represents a game area/zone the player can explore.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Area {
    pub name: String,
    pub description: String,
}

impl Area {
    /// Creates a new area with the given name and description.
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
        }
    }

    /// Returns the starting area of the game.
    pub fn starting_area() -> Self {
        Self::new(
            "The Beach",
            "Waves crash against the sandy shore. The air is thick with salt and mystery. This is where your journey begins.",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_area_is_the_beach() {
        let area = Area::starting_area();
        assert_eq!(area.name, "The Beach");
    }

    #[test]
    fn starting_area_has_description() {
        let area = Area::starting_area();
        assert!(!area.description.is_empty());
    }

    #[test]
    fn custom_area_creation() {
        let area = Area::new("Dark Forest", "Twisted trees block the sunlight.");
        assert_eq!(area.name, "Dark Forest");
        assert_eq!(area.description, "Twisted trees block the sunlight.");
    }

    #[test]
    fn area_serialization_roundtrip() {
        let area = Area::starting_area();
        let json = serde_json::to_string(&area).unwrap();
        let deserialized: Area = serde_json::from_str(&json).unwrap();
        assert_eq!(area, deserialized);
    }

    #[test]
    fn area_deserialization_rejects_invalid_json() {
        let result = serde_json::from_str::<Area>("garbage data {}[]");
        assert!(result.is_err());
    }
}
