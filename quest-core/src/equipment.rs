use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::item::{ItemCategory, ItemType, ITEM_REGISTRY};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EquipmentSlot {
    MainHand,
    OffHand,
    Head,
    Body,
    Hands,
    Feet,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EquipmentSection {
    Weapon,
    Armor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EquipmentItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub weight: u32,
    pub item_type: ItemType,
    pub min_damage: u32,
    pub max_damage: u32,
    pub slots: Vec<EquipmentSlot>,
    pub section: EquipmentSection,
    pub drop_source: String,
}

pub static EQUIPMENT_REGISTRY: Lazy<HashMap<String, EquipmentItem>> = Lazy::new(|| {
    let mut registry = HashMap::new();

    for item in ITEM_REGISTRY.values() {
        if item.category != ItemCategory::Equipment {
            continue;
        }

        let slots = parse_slots(&item.slots, &item.id);
        let (min_damage, max_damage) = item.damage_range().unwrap_or((0, 0));
        let section = if item.is_weapon() {
            EquipmentSection::Weapon
        } else {
            EquipmentSection::Armor
        };

        let equipment_item = EquipmentItem {
            id: item.id.clone(),
            name: item.name.clone(),
            description: item.description.clone(),
            weight: item.weight,
            item_type: item.item_type,
            min_damage,
            max_damage,
            slots,
            section,
            drop_source: item.drop_source.clone().unwrap_or_default(),
        };

        let item_id = equipment_item.id.clone();
        let prev = registry.insert(item_id.clone(), equipment_item);
        assert!(prev.is_none(), "Duplicate equipment id found: {}", item_id);
    }

    registry
});

fn parse_slots(slots: &[String], item_id: &str) -> Vec<EquipmentSlot> {
    let mut parsed = Vec::with_capacity(slots.len());

    for slot in slots {
        let mapped = match slot.as_str() {
            "main_hand" => EquipmentSlot::MainHand,
            "off_hand" => EquipmentSlot::OffHand,
            "head" => EquipmentSlot::Head,
            "body" => EquipmentSlot::Body,
            "hands" => EquipmentSlot::Hands,
            "feet" => EquipmentSlot::Feet,
            other => panic!("Unknown equipment slot '{other}' for item {item_id}"),
        };
        parsed.push(mapped);
    }

    parsed
}

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

    pub fn is_two_handed_weapon(&self) -> bool {
        self.item_type == ItemType::TwoHandedSword
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

    #[test]
    fn armor_item_has_armor_slot() {
        let helm = EquipmentItem::get_by_id("battered_helm").unwrap();
        assert!(helm.can_equip_in(EquipmentSlot::Head));
        assert_eq!(helm.section, EquipmentSection::Armor);
    }
}
