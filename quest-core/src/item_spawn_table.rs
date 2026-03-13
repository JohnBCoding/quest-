use crate::item::Item;
use once_cell::sync::Lazy;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ItemDropCategory {
    Weapons,
    Armor,
    Fruit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WeightedItemEntry {
    pub item_id: String,
    #[serde(default = "default_weight")]
    pub weight: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WeightedItemCategory {
    pub category: ItemDropCategory,
    #[serde(default = "default_weight")]
    pub weight: u32,
    #[serde(default)]
    pub items: Vec<WeightedItemEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ItemSpawnTable {
    #[serde(alias = "area_id")]
    pub id: String,
    #[serde(default)]
    pub base_drop_chance_percent: u8,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub categories: Vec<WeightedItemCategory>,
}

fn default_weight() -> u32 {
    1
}

pub static ITEM_SPAWN_TABLE_REGISTRY: Lazy<HashMap<String, ItemSpawnTable>> = Lazy::new(|| {
    let json_data = include_str!("../data/item_spawn_tables.json");
    let parsed: Vec<ItemSpawnTable> =
        serde_json::from_str(json_data).expect("Failed to parse item_spawn_tables.json");

    let mut registry = HashMap::new();
    for table in parsed {
        validate_spawn_table(&table);
        let table_id = table.id.clone();
        let prev = registry.insert(table_id.clone(), table);
        assert!(
            prev.is_none(),
            "Duplicate item spawn table id found in item_spawn_tables.json: {}",
            table_id
        );
    }

    registry
});

fn validate_spawn_table(table: &ItemSpawnTable) {
    assert!(
        !table.id.trim().is_empty(),
        "ItemSpawnTable id cannot be empty"
    );
    assert!(
        table.base_drop_chance_percent <= 100,
        "base_drop_chance_percent must be between 0 and 100 for table {}",
        table.id
    );
    assert!(
        !table.categories.is_empty(),
        "ItemSpawnTable {} must define at least one category",
        table.id
    );

    let mut seen_categories = HashSet::new();
    for category in &table.categories {
        assert!(
            category.weight > 0,
            "Category weight must be > 0 for table {}",
            table.id
        );
        assert!(
            seen_categories.insert(category.category),
            "Duplicate category {:?} in table {}",
            category.category,
            table.id
        );
        assert!(
            !category.items.is_empty(),
            "Category {:?} in table {} must include at least one item",
            category.category,
            table.id
        );

        let mut seen_items = HashSet::new();
        for item in &category.items {
            assert!(
                item.weight > 0,
                "Item weight must be > 0 for item {} in table {}",
                item.item_id,
                table.id
            );
            assert!(
                seen_items.insert(item.item_id.clone()),
                "Duplicate item {} in category {:?} for table {}",
                item.item_id,
                category.category,
                table.id
            );
            assert!(
                Item::get_by_id(&item.item_id).is_some(),
                "Unknown item_id {} referenced in table {}",
                item.item_id,
                table.id
            );
        }
    }
}

fn pick_weighted<'a, T, R, F>(entries: &'a [T], rng: &mut R, weight_of: F) -> Option<&'a T>
where
    R: Rng + ?Sized,
    F: Fn(&T) -> u32,
{
    let total_weight: u64 = entries
        .iter()
        .map(|entry| u64::from(weight_of(entry)))
        .sum();
    if total_weight == 0 {
        return None;
    }

    let mut roll = rng.gen_range(0..total_weight);
    for entry in entries {
        let weight = u64::from(weight_of(entry));
        if weight == 0 {
            continue;
        }
        if roll < weight {
            return Some(entry);
        }
        roll -= weight;
    }

    None
}

impl ItemSpawnTable {
    pub fn get_by_id(id: &str) -> Option<Self> {
        ITEM_SPAWN_TABLE_REGISTRY.get(id).cloned()
    }

    pub fn get_by_area_id(area_id: &str) -> Option<Self> {
        Self::get_by_id(area_id)
    }

    pub fn roll_drop<R: Rng + ?Sized>(&self, rng: &mut R) -> bool {
        if self.base_drop_chance_percent == 0 {
            return false;
        }
        if self.base_drop_chance_percent >= 100 {
            return true;
        }

        let roll: u8 = rng.gen_range(0..100);
        roll < self.base_drop_chance_percent
    }

    pub fn pick_category<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<&WeightedItemCategory> {
        pick_weighted(&self.categories, rng, |entry| entry.weight)
    }

    pub fn pick_item_id<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<&str> {
        let category = self.pick_category(rng)?;
        let item = pick_weighted(&category.items, rng, |entry| entry.weight)?;
        Some(item.item_id.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use std::collections::HashSet;

    #[test]
    fn registry_loads_successfully() {
        assert!(!ITEM_SPAWN_TABLE_REGISTRY.is_empty());
    }

    #[test]
    fn get_by_id_returns_dying_forest_table() {
        let table = ItemSpawnTable::get_by_id("dying_forest_items").expect("table should exist");
        assert_eq!(table.base_drop_chance_percent, 25);
        assert_eq!(table.categories.len(), 3);
    }

    #[test]
    fn spawn_table_defaults_weights_when_omitted() {
        let json = r#"{
            "id": "test_table",
            "categories": [
                {
                    "category": "fruit",
                    "items": [{"item_id": "fruit_of_assassination"}]
                }
            ]
        }"#;

        let table: ItemSpawnTable =
            serde_json::from_str(json).expect("Should deserialize with defaults");
        assert_eq!(table.base_drop_chance_percent, 0);
        assert_eq!(table.categories[0].weight, 1);
        assert_eq!(table.categories[0].items[0].weight, 1);
    }

    #[test]
    fn spawn_table_deserializes_area_id_alias() {
        let json = r#"{
            "area_id": "legacy_table",
            "categories": [
                {
                    "category": "fruit",
                    "items": [{"item_id": "fruit_of_assassination"}]
                }
            ]
        }"#;

        let table: ItemSpawnTable =
            serde_json::from_str(json).expect("Should deserialize with area_id alias");
        assert_eq!(table.id, "legacy_table");
    }

    #[test]
    fn roll_drop_respects_zero_and_hundred_percent() {
        let mut rng = StdRng::seed_from_u64(123);
        let never_drop = ItemSpawnTable {
            id: "never".to_string(),
            base_drop_chance_percent: 0,
            notes: None,
            categories: vec![],
        };
        let always_drop = ItemSpawnTable {
            id: "always".to_string(),
            base_drop_chance_percent: 100,
            notes: None,
            categories: vec![],
        };

        for _ in 0..10 {
            assert!(!never_drop.roll_drop(&mut rng));
            assert!(always_drop.roll_drop(&mut rng));
        }
    }

    #[test]
    fn pick_weighted_returns_none_when_all_weights_are_zero() {
        let mut rng = StdRng::seed_from_u64(42);
        let entries = [WeightedItemEntry {
            item_id: "dull_claymore".to_string(),
            weight: 0,
        }];

        let picked = pick_weighted(&entries, &mut rng, |entry| entry.weight);
        assert!(picked.is_none());
    }

    #[test]
    fn pick_item_id_only_returns_known_entries_from_table() {
        let table = ItemSpawnTable::get_by_id("dying_forest_items").expect("table should exist");
        let mut rng = StdRng::seed_from_u64(7);
        let allowed_ids: HashSet<String> = table
            .categories
            .iter()
            .flat_map(|category| category.items.iter().map(|item| item.item_id.clone()))
            .collect();

        for _ in 0..200 {
            let item_id = table
                .pick_item_id(&mut rng)
                .expect("table should be able to pick an item");
            assert!(allowed_ids.contains(item_id));
        }
    }
}
