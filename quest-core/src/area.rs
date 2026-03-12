use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Represents a game area/zone the player can explore.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Area {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub connected_areas: Vec<String>,
    #[serde(default)]
    pub base_encounter_amount: u32,
    #[serde(default)]
    pub bosses: Vec<String>,
}

pub static AREA_REGISTRY: Lazy<HashMap<String, Area>> = Lazy::new(|| {
    let json_data = include_str!("../data/areas.json");
    let areas: Vec<Area> = serde_json::from_str(json_data).expect("Failed to parse areas.json");
    let mut registry = HashMap::new();
    for area in areas {
        registry.insert(area.id.clone(), area);
    }
    registry
});

impl Area {
    /// Creates a new area with the given parameters.
    pub fn new(
        id: &str,
        name: &str,
        description: &str,
        connected_areas: Vec<String>,
        base_encounter_amount: u32,
        bosses: Vec<String>,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            connected_areas,
            base_encounter_amount,
            bosses,
        }
    }

    /// Returns the starting area of the game.
    pub fn starting_area() -> Self {
        Self::get_by_id("the_beach").expect("Starting area 'the_beach' not found in registry")
    }

    /// Retrieves an Area by its ID from the static registry.
    pub fn get_by_id(id: &str) -> Option<Self> {
        AREA_REGISTRY.get(id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_loads_successfully() {
        assert!(!AREA_REGISTRY.is_empty(), "Area registry should not be empty");
    }

    #[test]
    fn starting_area_is_the_beach() {
        let area = Area::starting_area();
        assert_eq!(area.name, "The Beach");
        assert_eq!(area.id, "the_beach");
    }

    #[test]
    fn starting_area_has_description() {
        let area = Area::starting_area();
        assert!(!area.description.is_empty());
    }

    #[test]
    fn custom_area_creation() {
        let area = Area::new("dark_forest", "Dark Forest", "Twisted trees block the sunlight.", vec!["the_beach".to_string()], 15, vec!["tree_ent".to_string()]);
        assert_eq!(area.id, "dark_forest");
        assert_eq!(area.name, "Dark Forest");
        assert_eq!(area.description, "Twisted trees block the sunlight.");
        assert_eq!(area.connected_areas, vec!["the_beach"]);
        assert_eq!(area.base_encounter_amount, 15);
        assert_eq!(area.bosses, vec!["tree_ent"]);
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

    #[test]
    fn get_by_id_returns_none_for_invalid_id() {
        assert!(Area::get_by_id("invalid_area_id_123").is_none());
    }
}
