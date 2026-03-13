use once_cell::sync::Lazy;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeightedSpawn {
    pub id: String,
    #[serde(default)]
    pub weight: u32,
}

impl WeightedSpawn {
    pub fn new(id: &str, weight: u32) -> Self {
        Self {
            id: id.to_string(),
            weight,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MobSpawnTable {
    pub id: String,
    #[serde(default)]
    pub mobs: Vec<WeightedSpawn>,
    #[serde(default)]
    pub bosses: Vec<WeightedSpawn>,
}

pub static MOB_SPAWN_TABLE_REGISTRY: Lazy<HashMap<String, MobSpawnTable>> = Lazy::new(|| {
    let json_data = include_str!("../data/mob_spawn_tables.json");
    let tables: Vec<MobSpawnTable> =
        serde_json::from_str(json_data).expect("Failed to parse mob_spawn_tables.json");

    let mut registry = HashMap::new();
    for table in tables {
        registry.insert(table.id.clone(), table);
    }
    registry
});

impl MobSpawnTable {
    pub fn new(id: &str, mobs: Vec<WeightedSpawn>, bosses: Vec<WeightedSpawn>) -> Self {
        Self {
            id: id.to_string(),
            mobs,
            bosses,
        }
    }

    pub fn get_by_id(id: &str) -> Option<Self> {
        MOB_SPAWN_TABLE_REGISTRY.get(id).cloned()
    }

    pub fn roll_mob_id<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<String> {
        pick_weighted_id(&self.mobs, rng)
    }

    pub fn roll_boss_id<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<String> {
        pick_weighted_id(&self.bosses, rng)
    }

    pub fn roll_mob_id_for_table<R: Rng + ?Sized>(table_id: &str, rng: &mut R) -> Option<String> {
        Self::get_by_id(table_id).and_then(|table| table.roll_mob_id(rng))
    }

    pub fn roll_boss_id_for_table<R: Rng + ?Sized>(table_id: &str, rng: &mut R) -> Option<String> {
        Self::get_by_id(table_id).and_then(|table| table.roll_boss_id(rng))
    }

    pub fn max_mob_weight(&self) -> Option<u32> {
        self.mobs.iter().map(|entry| entry.weight).max()
    }

    pub fn max_boss_weight(&self) -> Option<u32> {
        self.bosses.iter().map(|entry| entry.weight).max()
    }

    pub fn mob_weight(&self, mob_id: &str) -> Option<u32> {
        self.mobs
            .iter()
            .find(|entry| entry.id == mob_id)
            .map(|entry| entry.weight)
    }

    pub fn boss_weight(&self, mob_id: &str) -> Option<u32> {
        self.bosses
            .iter()
            .find(|entry| entry.id == mob_id)
            .map(|entry| entry.weight)
    }
}

fn pick_weighted_id<R: Rng + ?Sized>(entries: &[WeightedSpawn], rng: &mut R) -> Option<String> {
    let total_weight: u64 = entries.iter().map(|entry| entry.weight as u64).sum();
    if total_weight == 0 {
        return None;
    }

    let mut roll = rng.gen_range(0..total_weight);
    for entry in entries {
        if entry.weight == 0 {
            continue;
        }

        let weight = entry.weight as u64;
        if roll < weight {
            return Some(entry.id.clone());
        }
        roll -= weight;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn registry_loads_successfully() {
        assert!(
            !MOB_SPAWN_TABLE_REGISTRY.is_empty(),
            "spawn table registry should not be empty"
        );
    }

    #[test]
    fn get_by_id_returns_none_for_invalid_id() {
        assert!(MobSpawnTable::get_by_id("missing_spawn_table").is_none());
    }

    #[test]
    fn get_by_id_returns_dying_forest_table() {
        let table =
            MobSpawnTable::get_by_id("dying_forest_mobs").expect("dying_forest_mobs should exist");
        assert_eq!(table.mobs.len(), 3);
        assert_eq!(table.bosses.len(), 2);
    }

    #[test]
    fn roll_for_unknown_table_returns_none() {
        let mut rng = StdRng::seed_from_u64(7);
        assert!(MobSpawnTable::roll_mob_id_for_table("missing", &mut rng).is_none());
        assert!(MobSpawnTable::roll_boss_id_for_table("missing", &mut rng).is_none());
    }

    #[test]
    fn roll_returns_none_when_entries_are_empty() {
        let table = MobSpawnTable::new("empty", vec![], vec![]);
        let mut rng = StdRng::seed_from_u64(7);
        assert!(table.roll_mob_id(&mut rng).is_none());
        assert!(table.roll_boss_id(&mut rng).is_none());
    }

    #[test]
    fn roll_returns_none_when_total_weight_is_zero() {
        let table = MobSpawnTable::new(
            "zero",
            vec![WeightedSpawn::new("mugger", 0)],
            vec![WeightedSpawn::new("old_miller", 0)],
        );
        let mut rng = StdRng::seed_from_u64(19);
        assert!(table.roll_mob_id(&mut rng).is_none());
        assert!(table.roll_boss_id(&mut rng).is_none());
    }

    #[test]
    fn weighted_mob_rolls_favor_common_entries() {
        let table =
            MobSpawnTable::get_by_id("dying_forest_mobs").expect("dying_forest_mobs should exist");
        let mut rng = StdRng::seed_from_u64(42);

        let mut mugger_count = 0_u32;
        let mut poacher_count = 0_u32;
        let mut hungry_wolf_count = 0_u32;

        for _ in 0..2000 {
            match table.roll_mob_id(&mut rng).as_deref() {
                Some("mugger") => mugger_count += 1,
                Some("poacher") => poacher_count += 1,
                Some("hungry_wolf") => hungry_wolf_count += 1,
                Some(other) => panic!("unexpected mob id from weighted roll: {other}"),
                None => panic!("weighted roll should not be None for populated table"),
            }
        }

        assert!(mugger_count > poacher_count);
        assert!(poacher_count > hungry_wolf_count);
    }

    #[test]
    fn weighted_boss_rolls_favor_common_entries() {
        let table =
            MobSpawnTable::get_by_id("dying_forest_mobs").expect("dying_forest_mobs should exist");
        let mut rng = StdRng::seed_from_u64(84);

        let mut old_miller_count = 0_u32;
        let mut alpha_wolf_count = 0_u32;

        for _ in 0..1000 {
            match table.roll_boss_id(&mut rng).as_deref() {
                Some("old_miller") => old_miller_count += 1,
                Some("alpha_wolf") => alpha_wolf_count += 1,
                Some(other) => panic!("unexpected boss id from weighted roll: {other}"),
                None => panic!("weighted roll should not be None for populated table"),
            }
        }

        assert!(old_miller_count > alpha_wolf_count);
    }
}
