use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub mobs: Vec<String>,
    #[serde(default)]
    pub bosses: Vec<String>,
    #[serde(default)]
    pub mob_spawn_table_id: Option<String>,
    #[serde(default)]
    pub item_spawn_table_id: Option<String>,
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
        mobs: Vec<String>,
        bosses: Vec<String>,
        mob_spawn_table_id: Option<String>,
        item_spawn_table_id: Option<String>,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            connected_areas,
            base_encounter_amount,
            mobs,
            bosses,
            mob_spawn_table_id,
            item_spawn_table_id,
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
        assert!(
            !AREA_REGISTRY.is_empty(),
            "Area registry should not be empty"
        );
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
    fn starting_area_has_no_spawn_table_ids() {
        let area = Area::starting_area();
        assert!(area.mob_spawn_table_id.is_none());
        assert!(area.item_spawn_table_id.is_none());
    }

    #[test]
    fn custom_area_creation() {
        let area = Area::new(
            "dark_forest",
            "Dark Forest",
            "Twisted trees block the sunlight.",
            vec!["the_beach".to_string()],
            15,
            vec!["bat".to_string()],
            vec!["tree_ent".to_string()],
            Some("dark_forest_mobs".to_string()),
            Some("dark_forest_items".to_string()),
        );
        assert_eq!(area.id, "dark_forest");
        assert_eq!(area.name, "Dark Forest");
        assert_eq!(area.description, "Twisted trees block the sunlight.");
        assert_eq!(area.connected_areas, vec!["the_beach"]);
        assert_eq!(area.base_encounter_amount, 15);
        assert_eq!(area.mobs, vec!["bat"]);
        assert_eq!(area.bosses, vec!["tree_ent"]);
        assert_eq!(
            area.mob_spawn_table_id,
            Some("dark_forest_mobs".to_string())
        );
        assert_eq!(
            area.item_spawn_table_id,
            Some("dark_forest_items".to_string())
        );
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

    #[test]
    fn get_by_id_returns_dying_forest_with_spawn_tables() {
        let area = Area::get_by_id("dying_forest").expect("dying_forest should exist");
        assert_eq!(area.name, "The Dying Forest");
        assert_eq!(area.base_encounter_amount, 10);
        assert_eq!(
            area.mob_spawn_table_id.as_deref(),
            Some("dying_forest_mobs")
        );
        assert_eq!(
            area.item_spawn_table_id.as_deref(),
            Some("dying_forest_items")
        );
    }

    #[test]
    fn area_deserialization_without_spawn_tables_defaults_to_none() {
        let json = r#"{
            "id":"legacy_area",
            "name":"Legacy Area",
            "description":"legacy",
            "connected_areas":[],
            "base_encounter_amount":1,
            "mobs":[],
            "bosses":[]
        }"#;
        let area: Area = serde_json::from_str(json).expect("legacy area should deserialize");
        assert!(area.mob_spawn_table_id.is_none());
        assert!(area.item_spawn_table_id.is_none());
    }
}
