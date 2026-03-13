use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EquipmentSlot {
    MainHand,
    OffHand,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EquipmentSection {
    Weapon,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EquipmentItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub weight: u32,
    pub min_damage: u32,
    pub max_damage: u32,
    pub slots: Vec<EquipmentSlot>,
    pub section: EquipmentSection,
    pub drop_source: String,
}

#[derive(Debug, Clone, Deserialize)]
struct EquipmentData {
    #[serde(default)]
    weapon: Vec<WeaponData>,
}

#[derive(Debug, Clone, Deserialize)]
struct WeaponData {
    id: String,
    name: String,
    description: String,
    weight: u32,
    min_damage: u32,
    max_damage: u32,
    slots: Vec<EquipmentSlot>,
    drop_source: String,
}

pub static EQUIPMENT_REGISTRY: Lazy<HashMap<String, EquipmentItem>> = Lazy::new(|| {
    let json_data = include_str!("../data/equipment.json");
    let parsed: EquipmentData =
        serde_json::from_str(json_data).expect("Failed to parse equipment.json");

    let mut registry = HashMap::new();

    for weapon in parsed.weapon {
        assert!(
            weapon.min_damage <= weapon.max_damage,
            "Invalid damage range for equipment item {}",
            weapon.id
        );

        let id = weapon.id.clone();
        let item = EquipmentItem {
            id: weapon.id,
            name: weapon.name,
            description: weapon.description,
            weight: weapon.weight,
            min_damage: weapon.min_damage,
            max_damage: weapon.max_damage,
            slots: weapon.slots,
            section: EquipmentSection::Weapon,
            drop_source: weapon.drop_source,
        };

        let prev = registry.insert(id.clone(), item);
        assert!(prev.is_none(), "Duplicate equipment id found: {}", id);
    }

    registry
});

impl EquipmentItem {
    pub fn get_by_id(id: &str) -> Option<Self> {
        EQUIPMENT_REGISTRY.get(id).cloned()
    }

    pub fn get_first_by_drop_source(source_id: &str) -> Option<Self> {
        EQUIPMENT_REGISTRY
            .values()
            .find(|item| item.drop_source == source_id)
            .cloned()
    }

    pub fn get_all_by_drop_source(source_id: &str) -> Vec<Self> {
        EQUIPMENT_REGISTRY
            .values()
            .filter(|item| item.drop_source == source_id)
            .cloned()
            .collect()
    }

    pub fn can_equip_in(&self, slot: EquipmentSlot) -> bool {
        self.slots.contains(&slot)
    }

    pub fn damage_range(&self) -> (u32, u32) {
        (self.min_damage, self.max_damage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_loads_successfully() {
        assert!(!EQUIPMENT_REGISTRY.is_empty());
    }

    #[test]
    fn get_by_id_returns_split_hilt_blade() {
        let blade = EquipmentItem::get_by_id("split_hilt_blade").unwrap();
        assert_eq!(blade.name, "Split-Hilt Blade");
        assert_eq!(blade.weight, 1);
        assert_eq!(blade.damage_range(), (1, 4));
    }

    #[test]
    fn get_all_by_drop_source_returns_split_hilt_blade() {
        let drops = EquipmentItem::get_all_by_drop_source("rat_face");
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].id, "split_hilt_blade");
    }

    #[test]
    fn get_first_by_drop_source_returns_none_for_unknown_source() {
        assert!(EquipmentItem::get_first_by_drop_source("unknown").is_none());
    }

    #[test]
    fn split_hilt_blade_can_equip_both_hands() {
        let blade = EquipmentItem::get_by_id("split_hilt_blade").unwrap();
        assert!(blade.can_equip_in(EquipmentSlot::MainHand));
        assert!(blade.can_equip_in(EquipmentSlot::OffHand));
    }
}
