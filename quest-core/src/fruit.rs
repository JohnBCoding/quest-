use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::item::{ItemCategory, ITEM_REGISTRY};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Fruit {
    pub id: String,
    pub name: String,
    pub description: String,
    pub effect: String,
    pub drop_source: String,
}

pub static FRUIT_REGISTRY: Lazy<HashMap<String, Fruit>> = Lazy::new(|| {
    let mut registry = HashMap::new();

    for item in ITEM_REGISTRY.values() {
        if item.category != ItemCategory::Fruit {
            continue;
        }

        let Some(effect) = item.effect.as_ref() else {
            continue;
        };

        let fruit = Fruit {
            id: item.id.clone(),
            name: item.name.clone(),
            description: item.description.clone(),
            effect: effect.clone(),
            drop_source: item.drop_source.clone().unwrap_or_default(),
        };
        registry.insert(fruit.id.clone(), fruit);
    }

    registry
});

impl Fruit {
    pub fn get_by_id(id: &str) -> Option<Self> {
        FRUIT_REGISTRY.get(id).cloned()
    }

    pub fn get_by_drop_source(source_id: &str) -> Option<Self> {
        FRUIT_REGISTRY
            .values()
            .find(|f| f.drop_source == source_id)
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_loads_successfully() {
        assert!(!FRUIT_REGISTRY.is_empty());
    }

    #[test]
    fn get_by_id_returns_valid_fruit() {
        let fruit = Fruit::get_by_id("fruit_of_instinct").expect("Should find fruit_of_instinct");
        assert_eq!(fruit.name, "Fruit of Instinct");
        assert_eq!(fruit.effect, "unlock_auto_combat");
    }

    #[test]
    fn get_by_drop_source_finds_instinct_fruit() {
        let fruit = Fruit::get_by_drop_source("rat_lord").expect("Rat Lord should drop a fruit");
        assert_eq!(fruit.id, "fruit_of_instinct");
    }

    #[test]
    fn get_by_id_returns_none_for_nonexistent() {
        assert!(Fruit::get_by_id("nonexistent_fruit").is_none());
    }

    #[test]
    fn get_by_drop_source_returns_none_for_unknown_boss() {
        assert!(Fruit::get_by_drop_source("no_boss").is_none());
    }

    #[test]
    fn fruit_serialization_roundtrip() {
        let fruit = Fruit::get_by_id("fruit_of_instinct").unwrap();
        let json = serde_json::to_string(&fruit).unwrap();
        let deserialized: Fruit = serde_json::from_str(&json).unwrap();
        assert_eq!(fruit, deserialized);
    }
}
