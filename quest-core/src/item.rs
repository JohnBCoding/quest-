use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ItemCategory {
    Equipment,
    Fruit,
}

impl Default for ItemCategory {
    fn default() -> Self {
        Self::Equipment
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
}

impl Default for ItemRarity {
    fn default() -> Self {
        Self::Common
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    OneHandedSword,
    OneHandedDagger,
    TwoHandedSword,
    Helmet,
    BodyArmor,
    Gloves,
    Boots,
    Fruit,
    Unknown,
}

impl Default for ItemType {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub rarity: ItemRarity,
    #[serde(default)]
    pub item_type: ItemType,
    #[serde(default)]
    pub weight: u32,
    #[serde(default)]
    pub min_damage: Option<u32>,
    #[serde(default)]
    pub max_damage: Option<u32>,
    #[serde(default)]
    pub effect: Option<String>,
    #[serde(default)]
    pub slots: Vec<String>,
    #[serde(default)]
    pub drop_source: Option<String>,
    #[serde(skip)]
    pub category: ItemCategory,
}

#[derive(Debug, Clone, Deserialize)]
struct ItemData {
    #[serde(default)]
    equipment: Vec<Item>,
    #[serde(default)]
    fruit: Vec<Item>,
}

pub static ITEM_REGISTRY: Lazy<HashMap<String, Item>> = Lazy::new(|| {
    let json_data = include_str!("../data/items.json");
    let parsed: ItemData = serde_json::from_str(json_data).expect("Failed to parse items.json");
    let mut registry = HashMap::new();

    for mut item in parsed.equipment {
        item.category = ItemCategory::Equipment;
        validate_item(&item);
        let item_id = item.id.clone();
        let prev = registry.insert(item_id.clone(), item);
        assert!(
            prev.is_none(),
            "Duplicate item id found in items.json: {}",
            item_id
        );
    }

    for mut item in parsed.fruit {
        item.category = ItemCategory::Fruit;
        validate_item(&item);
        let item_id = item.id.clone();
        let prev = registry.insert(item_id.clone(), item);
        assert!(
            prev.is_none(),
            "Duplicate item id found in items.json: {}",
            item_id
        );
    }

    registry
});

fn validate_item(item: &Item) {
    assert!(
        !item.id.trim().is_empty(),
        "Item id cannot be empty in items.json"
    );
    assert!(
        !item.name.trim().is_empty(),
        "Item name cannot be empty for item {}",
        item.id
    );
    assert!(
        !item.description.trim().is_empty(),
        "Item description cannot be empty for item {}",
        item.id
    );
    assert!(
        item.item_type != ItemType::Unknown,
        "Item {} is missing a valid item_type",
        item.id
    );

    let has_min = item.min_damage.is_some();
    let has_max = item.max_damage.is_some();
    assert!(
        has_min == has_max,
        "Item {} must define both min_damage and max_damage, or neither",
        item.id
    );

    if let Some((min_damage, max_damage)) = item.damage_range() {
        assert!(
            min_damage <= max_damage,
            "Item {} has invalid damage range: {} > {}",
            item.id,
            min_damage,
            max_damage
        );
    }

    match item.category {
        ItemCategory::Equipment => {
            assert!(
                !item.slots.is_empty(),
                "Equipment item {} must include at least one slot",
                item.id
            );
            if item.is_weapon() {
                assert!(
                    item.damage_range().is_some(),
                    "Weapon item {} must include damage range",
                    item.id
                );
            }
        }
        ItemCategory::Fruit => {
            assert!(
                item.item_type == ItemType::Fruit,
                "Fruit item {} must have item_type = fruit",
                item.id
            );
            assert!(
                item.effect
                    .as_deref()
                    .is_some_and(|effect| !effect.trim().is_empty()),
                "Fruit item {} must include a non-empty effect description",
                item.id
            );
        }
    }
}

impl Item {
    pub fn get_by_id(id: &str) -> Option<Self> {
        ITEM_REGISTRY.get(id).cloned()
    }

    pub fn all() -> Vec<Self> {
        ITEM_REGISTRY.values().cloned().collect()
    }

    pub fn is_weapon(&self) -> bool {
        matches!(
            self.item_type,
            ItemType::OneHandedSword | ItemType::OneHandedDagger | ItemType::TwoHandedSword
        )
    }

    pub fn is_equipment(&self) -> bool {
        self.category == ItemCategory::Equipment
    }

    pub fn damage_range(&self) -> Option<(u32, u32)> {
        Some((self.min_damage?, self.max_damage?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_loads_successfully() {
        assert!(!ITEM_REGISTRY.is_empty());
    }

    #[test]
    fn split_hilt_blade_is_loaded_from_equipment_category() {
        let item = Item::get_by_id("split_hilt_blade").expect("Should find split_hilt_blade");
        assert_eq!(item.category, ItemCategory::Equipment);
        assert_eq!(item.drop_source.as_deref(), Some("rat_face"));
        assert!(item.slots.iter().any(|slot| slot == "main_hand"));
    }

    #[test]
    fn get_by_id_returns_dull_claymore() {
        let item = Item::get_by_id("dull_claymore").expect("Should find dull_claymore");
        assert_eq!(item.name, "Dull Claymore");
        assert_eq!(item.rarity, ItemRarity::Common);
        assert_eq!(item.item_type, ItemType::TwoHandedSword);
        assert_eq!(item.weight, 5);
        assert_eq!(item.damage_range(), Some((5, 10)));
    }

    #[test]
    fn fruit_of_assassination_has_required_effect_text() {
        let item =
            Item::get_by_id("fruit_of_assassination").expect("Should find fruit_of_assassination");
        assert_eq!(item.category, ItemCategory::Fruit);
        assert_eq!(item.rarity, ItemRarity::Rare);
        assert_eq!(item.item_type, ItemType::Fruit);
        assert_eq!(
            item.effect.as_deref(),
            Some("If the target is below 35% hp, perform a killing blow, instantly killing them.")
        );
        assert_eq!(item.damage_range(), None);
    }
}
