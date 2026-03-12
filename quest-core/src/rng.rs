use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Manages multiple independent seeded RNGs, keyed by category name.
/// Each category (e.g., "loot", "combat", "world") has its own seed,
/// ensuring independence and reproducibility per-category.
#[derive(Debug, Clone)]
pub struct RngManager {
    seeds: HashMap<String, u64>,
    rngs: HashMap<String, StdRng>,
}

/// Serializable snapshot of RNG state (seeds only).
/// When loading, we re-create RNGs from seeds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RngSnapshot {
    pub seeds: HashMap<String, u64>,
}

impl RngManager {
    /// Default categories every game starts with.
    const DEFAULT_CATEGORIES: &'static [&'static str] = &["loot", "combat", "world", "encounter"];

    /// Creates a new RngManager with random seeds for all default categories.
    pub fn new() -> Self {
        let mut seeds = HashMap::new();
        let mut rngs = HashMap::new();
        let mut entropy_rng = StdRng::from_entropy();

        for &category in Self::DEFAULT_CATEGORIES {
            let seed: u64 = entropy_rng.gen();
            seeds.insert(category.to_string(), seed);
            rngs.insert(category.to_string(), StdRng::seed_from_u64(seed));
        }

        Self { seeds, rngs }
    }

    /// Creates an RngManager from a snapshot (for loading saved games).
    pub fn from_snapshot(snapshot: &RngSnapshot) -> Self {
        let mut rngs = HashMap::new();
        for (name, &seed) in &snapshot.seeds {
            rngs.insert(name.clone(), StdRng::seed_from_u64(seed));
        }
        Self {
            seeds: snapshot.seeds.clone(),
            rngs,
        }
    }

    /// Takes a snapshot of the current seeds for serialization.
    pub fn snapshot(&self) -> RngSnapshot {
        RngSnapshot {
            seeds: self.seeds.clone(),
        }
    }

    /// Gets the RNG for a given category. Creates a new seeded RNG if the
    /// category doesn't exist yet.
    pub fn get(&mut self, category: &str) -> &mut StdRng {
        if !self.rngs.contains_key(category) {
            let mut entropy_rng = StdRng::from_entropy();
            let seed: u64 = entropy_rng.gen();
            self.seeds.insert(category.to_string(), seed);
            self.rngs
                .insert(category.to_string(), StdRng::seed_from_u64(seed));
        }
        self.rngs.get_mut(category).unwrap()
    }

    /// Returns the list of active category names.
    pub fn categories(&self) -> Vec<String> {
        self.seeds.keys().cloned().collect()
    }

    /// Generates a random number in the given range for a category.
    pub fn gen_range(&mut self, category: &str, low: u32, high: u32) -> u32 {
        self.get(category).gen_range(low..=high)
    }
}

impl Default for RngManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rng_manager_has_default_categories() {
        let rng = RngManager::new();
        let categories = rng.categories();
        for &cat in RngManager::DEFAULT_CATEGORIES {
            assert!(
                categories.contains(&cat.to_string()),
                "Missing default category: {}",
                cat
            );
        }
    }

    #[test]
    fn different_categories_produce_different_sequences() {
        let mut rng = RngManager::new();
        let vals_loot: Vec<u32> = (0..10).map(|_| rng.gen_range("loot", 0, 1000)).collect();
        let vals_combat: Vec<u32> = (0..10).map(|_| rng.gen_range("combat", 0, 1000)).collect();
        // Extremely unlikely to be identical with different seeds
        assert_ne!(vals_loot, vals_combat);
    }

    #[test]
    fn seeded_rng_is_reproducible() {
        let rng1 = RngManager::new();
        let snapshot = rng1.snapshot();

        let mut rng_a = RngManager::from_snapshot(&snapshot);
        let mut rng_b = RngManager::from_snapshot(&snapshot);

        let vals_a: Vec<u32> = (0..20).map(|_| rng_a.gen_range("loot", 0, 10000)).collect();
        let vals_b: Vec<u32> = (0..20).map(|_| rng_b.gen_range("loot", 0, 10000)).collect();
        assert_eq!(vals_a, vals_b);
    }

    #[test]
    fn snapshot_roundtrip_serialization() {
        let rng = RngManager::new();
        let snapshot = rng.snapshot();
        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: RngSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snapshot, deserialized);
    }

    #[test]
    fn unknown_category_auto_creates() {
        let mut rng = RngManager::new();
        // Should not panic — auto-creates category
        let val = rng.gen_range("brand_new_category", 1, 100);
        assert!((1..=100).contains(&val));
        assert!(rng.categories().contains(&"brand_new_category".to_string()));
    }

    #[test]
    fn gen_range_respects_bounds() {
        let mut rng = RngManager::new();
        for _ in 0..100 {
            let val = rng.gen_range("loot", 5, 10);
            assert!((5..=10).contains(&val), "Value {} out of range 5..=10", val);
        }
    }

    #[test]
    fn snapshot_deserialization_rejects_invalid_json() {
        let result = serde_json::from_str::<RngSnapshot>("not valid");
        assert!(result.is_err());
    }
}
