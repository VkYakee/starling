//! Structure cracking using Horizon's native StructureRegistry as oracle.
//! This guarantees 100% parity by testing candidate seeds against Horizon's
//! exact `is_structure_chunk` implementation.

use horizon_core::random::{LegacyRandom, RandomSource};
use horizon_world::worldgen::structure::start::{StructureRegistry, StructureSet, Rings};
use std::sync::atomic::{AtomicBool, Ordering};
use rayon::prelude::*;

/// An observed structure at specific chunk coordinates
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObservedStructure {
    /// Structure set name (e.g., "minecraft:villages", "minecraft:strongholds")
    pub set_name: String,
    /// Chunk X coordinate
    pub chunk_x: i32,
    /// Chunk Z coordinate
    pub chunk_z: i32,
}

/// Result of structure cracking
#[derive(Debug, Clone, Default)]
pub struct StructureCrackerResult {
    /// Candidate 48-bit structure seeds that match all observed structures
    pub candidates: Vec<u64>,
    /// Whether the search was cancelled
    pub cancelled: bool,
    /// Whether the search completed exhaustively
    pub exhaustive: bool,
}

impl StructureCrackerResult {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Structure cracker that uses Horizon's StructureRegistry as the ground truth oracle
pub struct StructureCracker<'a> {
    registry: &'a StructureRegistry,
    observed: Vec<ObservedStructure>,
    possible_sets: Vec<usize>,
    rings: &'a Rings,
}

impl<'a> StructureCracker<'a> {
    /// Create a new StructureCracker from Horizon's StructureRegistry
    pub fn new(registry: &'a StructureRegistry, observed: Vec<ObservedStructure>, rings: &'a Rings) -> Self {
        // Determine which sets could have generated the observed structures
        let possible_sets = registry
            .sets
            .iter()
            .enumerate()
            .filter(|(_, set)| {
                observed.iter().any(|obs| obs.set_name == set.name)
            })
            .map(|(i, _)| i)
            .collect();

        Self {
            registry,
            observed,
            possible_sets,
            rings,
        }
    }

    /// Create with pre-computed rings (for testing/observers)
    pub fn with_rings(
        registry: &'a StructureRegistry, 
        observed: Vec<ObservedStructure>, 
        rings: &'a Rings
    ) -> Self {
        Self::new(registry, observed, rings)
    }

    /// Test if a candidate 48-bit structure seed matches all observed structures
    pub fn test_seed(&self, structure_seed: u64) -> bool {
        for obs in &self.observed {
            // Find the set index for this observation
            let set_idx = self.registry.sets.iter().position(|s| s.name == obs.set_name);
            let Some(set_idx) = set_idx else { continue; };
            let set = &self.registry.sets[set_idx];

            // Use Horizon's exact is_structure_chunk logic
            if !self.registry.is_structure_chunk(set, structure_seed as i64, obs.chunk_x, obs.chunk_z, self.rings) {
                return false;
            }
        }
        true
    }

    /// Search for candidate structure seeds within a range of upper bits
    /// Returns all matching 48-bit seeds in the range
    pub fn search_range(
        &self,
        upper_start: u64,
        upper_end: u64,
        cancel: &AtomicBool,
    ) -> Vec<u64> {
        if self.observed.is_empty() || upper_start >= upper_end || upper_start >= (1u64 << 29) {
            return Vec::new();
        }

        // Filter to sets with RandomSpread placement (for lower-bit filtering optimization)
        let eligible: Vec<&StructureSet> = self
            .possible_sets
            .iter()
            .filter_map(|&idx| {
                let set = &self.registry.sets[idx];
                if set.placement_spacing().is_some() {
                    Some(set)
                } else {
                    None
                }
            })
            .collect();

        if eligible.is_empty() {
            // Fall back to exhaustive check without lower-bit filtering
            return self.search_range_exhaustive(upper_start, upper_end, cancel);
        }

        // Phase 1: Find valid lower 19 bits by checking modulo pattern
        // This uses the fact that region seeds at (0,0) only depend on lower 19 bits
        let valid_lower: Vec<u64> = (0..(1u64 << 19))
            .into_par_iter()
            .filter(|&lower| {
                if cancel.load(Ordering::Relaxed) {
                    return false;
                }
                eligible.iter().all(|set| {
                    let spacing = set.placement_spacing().unwrap_or(1);
                    // Check the region seed pattern at (0,0) for this lower seed
                    let region_seed = Self::get_region_seed(lower, 0, 0, set.salt);
                    let mut rand = LegacyRandom::new(region_seed as i64);
                    let limit = spacing - set.separation();
                    if limit <= 0 { return true; }
                    let offset_x = rand.next_int_bound(limit).unwrap_or(0);
                    let offset_z = rand.next_int_bound(limit).unwrap_or(0);
                    // Check modulo 4 pattern (structure offsets must align to 4 for many structures)
                    offset_x % 4 == 0 && offset_z % 4 == 0
                })
            })
            .collect();

        // Phase 2: Combine with upper bits and test full seeds
        let mut results = Vec::new();
        for lower in valid_lower {
            for upper in upper_start..upper_end.min(1u64 << 29) {
                if cancel.load(Ordering::Relaxed) {
                    return results;
                }
                let seed = ((upper << 19) | lower) & 0xFFFFFFFFFFFF;
                if self.test_seed(seed) {
                    results.push(seed);
                }
            }
        }
        results
    }

    /// Exhaustive search without lower-bit optimization
    fn search_range_exhaustive(
        &self,
        upper_start: u64,
        upper_end: u64,
        cancel: &AtomicBool,
    ) -> Vec<u64> {
        let mut results = Vec::new();
        for upper in upper_start..upper_end.min(1u64 << 29) {
            if cancel.load(Ordering::Relaxed) {
                return results;
            }
            for lower in 0..(1u64 << 19) {
                let seed = ((upper << 19) | lower) & 0xFFFFFFFFFFFF;
                if self.test_seed(seed) {
                    results.push(seed);
                }
            }
        }
        results
    }

    /// Exhaustive search for all candidate structure seeds (parallel)
    pub fn search_exhaustive(&self, cancel: &AtomicBool) -> Vec<u64> {
        // Use parallel search over upper 29 bits
        (0..(1u64 << 29))
            .into_par_iter()
            .flat_map(|upper| {
                if cancel.load(Ordering::Relaxed) {
                    return Vec::new();
                }
                let mut local_results = Vec::new();
                for lower in 0..(1u64 << 19) {
                    let seed = ((upper << 19) | lower) & 0xFFFFFFFFFFFF;
                    if self.test_seed(seed) {
                        local_results.push(seed);
                    }
                }
                local_results
            })
            .collect()
    }

    /// Compute region seed exactly as Horizon does
    /// Matches WorldgenRandom::set_large_feature_with_salt
    /// This computes the seed directly without going through WorldgenRandom
    /// since we can't access the underlying source state.
    fn get_region_seed(base_seed: u64, region_x: i32, region_z: i32, salt: i32) -> u64 {
        // This matches WorldgenRandom::set_large_feature_with_salt implementation:
        // set_seed((x * 341873128712) + (z * 132897987541) + seed + salt)
        let seed = (region_x as i64)
            .wrapping_mul(341873128712)
            .wrapping_add((region_z as i64).wrapping_mul(132897987541))
            .wrapping_add(base_seed as i64)
            .wrapping_add(salt as i64);
        
        // Create LegacyRandom with that seed and get its 48-bit state
        let rand = LegacyRandom::new(seed);
        rand.state()
    }
}

/// Extension methods for StructureSet
trait StructureSetExt {
    fn placement_spacing(&self) -> Option<i32>;
    fn separation(&self) -> i32;
}

impl StructureSetExt for StructureSet {
    fn placement_spacing(&self) -> Option<i32> {
        match self.placement {
            Placement::RandomSpread { spacing, .. } => Some(spacing),
            _ => None,
        }
    }

    fn separation(&self) -> i32 {
        match self.placement {
            Placement::RandomSpread { separation, .. } => separation,
            _ => 0,
        }
    }
}

/// Builder for collecting observed structures from gameplay
pub struct StructureObserver {
    observations: Vec<ObservedStructure>,
}

impl StructureObserver {
    pub fn new() -> Self {
        Self { observations: Vec::new() }
    }

    pub fn add(&mut self, set_name: String, chunk_x: i32, chunk_z: i32) {
        self.observations.push(ObservedStructure { set_name, chunk_x, chunk_z });
    }

    pub fn add_block_pos(&mut self, set_name: String, block_x: i32, block_z: i32) {
        self.add(set_name, block_x >> 4, block_z >> 4);
    }

    pub fn build(self) -> Vec<ObservedStructure> {
        self.observations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_core::random::{LegacyRandom, WorldgenRandom};

    #[test]
    fn test_region_seed_matches_horizon() {
        let seed = 765906787396911863u64 & 0xFFFFFFFFFFFF;
        let region_x = 0;
        let region_z = 0;
        let salt = 10387312; // villages salt

        let ours = StructureCracker::get_region_seed(seed, region_x, region_z, salt);
        
        let mut horizon_random = WorldgenRandom::new(LegacyRandom::new(0));
        horizon_random.set_large_feature_with_salt(seed as i64, region_x, region_z, salt);
        // The state of the underlying LegacyRandom after set_seed
        // is exactly what we compute
        let horizon_seed = horizon_random.state().state();
        
        assert_eq!(ours, horizon_seed, "Region seed must match Horizon exactly");
    }
}
