//! Main SeedCracker module - the unified cracking engine.
//! Coordinates structure, biome, bedrock, and pillar cracking using Horizon's native APIs.

mod bedrock;
mod biome;
mod structure;

use horizon_core::biome::obfuscate_seed;
use horizon_core::random::{LegacyRandom, WorldgenRandom};
use horizon_world::worldgen::structure::start::{StructureRegistry, Rings};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use rayon::prelude::*;

pub use crate::cracker::bedrock::{BedrockCracker, BedrockCrackerResult, BedrockObservation, BedrockObserver, BedrockColumn};
pub use crate::cracker::biome::{BiomeCracker, BiomeObserver, ObservedBiome};
pub use crate::cracker::structure::{StructureCracker, StructureCrackerResult, StructureObserver, ObservedStructure};
pub use crate::pillars::{EndPillars, PillarObservations, PILLAR_COUNT};
use crate::math::chunk_rand;
use crate::math::hashed_seed;


/// Complete cracker state containing all observations
#[derive(Debug, Clone, Default)]
pub struct CrackerState {
    // Structure observations
    pub structures: Vec<StructureCrackerObservedStructure>,
    // Biome observations
    pub biomes: Vec<BiomeCrackerObservedBiome>,
    // Bedrock observations
    pub bedrock_chunks: Vec<BedrockCrackerBedrockColumn>,
    // Pillar observations
    pub pillars: PillarObservations,
    // Hashed seed from SpawnInfo packet (if available)
    pub hashed_seed: Option<i64>,
    // Target MC version
    pub target_mc_version: i32,
}

impl CrackerState {
    pub fn new() -> Self {
        Self {
            target_mc_version: 34, // MC 1.20.x
            ..Default::default()
        }
    }

    pub fn add_structure(&mut self, set_name: String, chunk_x: i32, chunk_z: i32) {
        self.structures.push(StructureCrackerObservedStructure { set_name, chunk_x, chunk_z });
    }

    pub fn add_biome(&mut self, x: i32, y: i32, z: i32, biome_name: String) {
        self.biomes.push(BiomeCrackerObservedBiome { x, y, z, biome_name });
    }

    pub fn add_bedrock_column(&mut self, chunk_x: i32, chunk_z: i32, roof_mask: u64, floor_mask: u64) {
        self.bedrock_chunks.push(BedrockCrackerBedrockColumn {
            chunk_x,
            chunk_z,
            roof_mask,
            floor_mask,
        });
    }

    pub fn set_pillar_height(&mut self, index: usize, height: i32) {
        self.pillars.set_height(index, height);
    }

    pub fn set_hashed_seed(&mut self, hashed_seed: i64) {
        self.hashed_seed = Some(hashed_seed);
    }
}

/// Bedrock column re-export for CrackerState
pub use crate::cracker::bedrock::BedrockColumn as BedrockCrackerBedrockColumn;

/// Biome observation re-export for CrackerState
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BiomeCrackerObservedBiome {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub biome_name: String,
}

/// Structure observation re-export for CrackerState
pub use crate::cracker::structure::ObservedStructure as StructureCrackerObservedStructure;

/// Complete cracker results
#[derive(Debug, Clone, Default)]
pub struct CrackerResult {
    pub structure_seeds: Vec<u64>,    // 48-bit structure seeds
    pub world_seeds: Vec<i64>,        // 64-bit world seeds
    pub pillar_seeds: Vec<u16>,       // 16-bit pillar seeds
    pub bedrock_seeds: Vec<u64>,      // Population seeds
    pub cancelled: bool,
    pub validated: bool,
    pub resolution_complete: bool,
}

impl CrackerResult {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Main SeedCracker engine
pub struct SeedCracker;

impl SeedCracker {
    /// Solve for seeds given the current state
    pub fn solve(
        state: &CrackerState,
        registry: &StructureRegistry,
        rings: &Rings,
    ) -> CrackerResult {
        Self::solve_cancellable(state, registry, rings, Arc::new(AtomicBool::new(false)))
    }

    /// Solve with cancellation support
    pub fn solve_cancellable(
        state: &CrackerState,
        registry: &StructureRegistry,
        rings: &Rings,
        cancel: Arc<AtomicBool>,
    ) -> CrackerResult {
        let mut result = CrackerResult::new();
        let cancelled = || cancel.load(Ordering::Relaxed);

        // Phase 1: Structure cracking (most constraining)
        if !state.structures.is_empty() {
            println!("[SeedCracker] Phase 1: Structure cracking with {} observations", state.structures.len());
            let cracker = StructureCracker::with_rings(
                registry,
                state.structures.clone(),
                rings,
            );
            
            // Exhaustive search for structure seeds
            let structure_seeds = cracker.search_exhaustive(&cancel);
            result.structure_seeds = structure_seeds;
            println!("[SeedCracker] Found {} candidate structure seeds", result.structure_seeds.len());
            
            if cancelled() {
                result.cancelled = true;
                return result;
            }
        }

        // Phase 2: Pillar cracking (independent, fast)
        if state.pillars.observed_count() >= 3 {
            println!("[SeedCracker] Phase 2: Pillar cracking with {} observed pillars", state.pillars.observed_count());
            let pillar_seeds = EndPillars::find_pillar_seeds(&state.pillars.to_array());
            result.pillar_seeds = pillar_seeds;
            println!("[SeedCracker] Found {} candidate pillar seeds", result.pillar_seeds.len());
            
            if cancelled() {
                result.cancelled = true;
                return result;
            }
        }

        // Phase 3: Bedrock cracking (if we have structure seeds)
        if !state.bedrock_chunks.is_empty() && !result.structure_seeds.is_empty() {
            println!("[SeedCracker] Phase 3: Bedrock cracking with {} structure seed candidates", result.structure_seeds.len());
            for &structure_seed in &result.structure_seeds {
                let pop_seed = chunk_rand::get_population_seed(structure_seed, state.bedrock_chunks[0].chunk_x, state.bedrock_chunks[0].chunk_z);
                let mut cracker = BedrockCracker::new();
                for column in &state.bedrock_chunks {
                    cracker.add_column(column.chunk_x, column.chunk_z, column.roof_mask, column.floor_mask);
                }
                if cracker.test_population_seed(pop_seed) {
                    result.bedrock_seeds.push(pop_seed);
                }
            }
            println!("[SeedCracker] Found {} matching bedrock seeds", result.bedrock_seeds.len());
            
            if cancelled() {
                result.cancelled = true;
                return result;
            }
        }

        // Phase 4: Cross-filter structure seeds with pillar seeds
        if !result.structure_seeds.is_empty() && !result.pillar_seeds.is_empty() {
            println!("[SeedCracker] Phase 4: Cross-filtering structure seeds with pillar seeds");
            let mut filtered = Vec::new();
            for &structure_seed in &result.structure_seeds {
                let pillar_seed = EndPillars::pillar_seed_from_structure_seed(structure_seed);
                if result.pillar_seeds.contains(&pillar_seed) {
                    filtered.push(structure_seed);
                }
            }
            result.structure_seeds = filtered;
            println!("[SeedCracker] {} structure seeds remain after pillar filtering", result.structure_seeds.len());
        }

        // Phase 5: Lift to 64-bit world seeds using hashed seed
        if !result.structure_seeds.is_empty() && state.hashed_seed.is_some() {
            println!("[SeedCracker] Phase 5: Lifting to 64-bit world seeds using hashed seed");
            let hashed_seed = state.hashed_seed.unwrap();
            for &structure_seed in &result.structure_seeds {
                let world_seeds = hashed_seed::find_world_seeds_from_hash(structure_seed, hashed_seed);
                result.world_seeds.extend(world_seeds);
            }
            println!("[SeedCracker] Found {} world seeds", result.world_seeds.len());
        }

        // Phase 6: Validate world seeds against all observations
        if !result.world_seeds.is_empty() {
            println!("[SeedCracker] Phase 6: Validating world seeds against all observations");
            result.world_seeds.retain(|&world_seed| {
                Self::validate_world_seed(world_seed, state, registry, rings)
            });
            result.validated = !result.world_seeds.is_empty();
            println!("[SeedCracker] {} world seeds remain after validation", result.world_seeds.len());
        }

        result.resolution_complete = result.validated && !result.world_seeds.is_empty();
        result
    }

    /// Validate a world seed against all observations
    fn validate_world_seed(
        world_seed: i64, 
        state: &CrackerState,
        registry: &StructureRegistry,
        rings: &Rings,
    ) -> bool {
        let structure_seed = (world_seed as u64) & 0xFFFFFFFFFFFF;
        
        // Check structure observations
        if !state.structures.is_empty() {
            let cracker = StructureCracker::with_rings(
                registry,
                state.structures.clone(),
                rings,
            );
            if !cracker.test_seed(structure_seed) {
                return false;
            }
        }

        // Check pillar observations
        if state.pillars.observed_count() >= 3 {
            let pillar_seed = EndPillars::pillar_seed_from_structure_seed(structure_seed);
            let heights = EndPillars::get_pillar_heights(pillar_seed);
            
            for i in 0..crate::pillars::PILLAR_COUNT {
                if let Some(expected) = state.pillars.heights[i] {
                    if heights[i] != expected {
                        return false;
                    }
                }
            }
        }

        // Check biome observations (would need biome source integration)
        if !state.biomes.is_empty() {
            // Placeholder - in full implementation would check against biome source
        }

        // Check bedrock observations
        for column in &state.bedrock_chunks {
            let pop_seed = chunk_rand::get_population_seed(structure_seed, column.chunk_x, column.chunk_z);
            let mut cracker = BedrockCracker::new();
            cracker.add_column(column.chunk_x, column.chunk_z, column.roof_mask, column.floor_mask);
            if !cracker.test_population_seed(pop_seed) {
                return false;
            }
        }

        true
    }

    /// Get population seed from structure seed
    pub fn get_population_seed(structure_seed: u64, chunk_x: i32, chunk_z: i32) -> u64 {
        chunk_rand::get_population_seed(structure_seed, chunk_x, chunk_z)
    }
}

/// Observer builder for collecting data from gameplay
pub struct SeedCrackerObserver {
    state: CrackerState,
}

impl SeedCrackerObserver {
    pub fn new() -> Self {
        Self { state: CrackerState::new() }
    }

    pub fn on_structure_found(&mut self, set_name: String, chunk_x: i32, chunk_z: i32) {
        self.state.add_structure(set_name, chunk_x, chunk_z);
    }

    pub fn on_biome_found(&mut self, x: i32, y: i32, z: i32, biome_name: String) {
        self.state.add_biome(x, y, z, biome_name);
    }

    pub fn on_bedrock_column(&mut self, chunk_x: i32, chunk_z: i32, roof_mask: u64, floor_mask: u64) {
        self.state.add_bedrock_column(chunk_x, chunk_z, roof_mask, floor_mask);
    }

    pub fn on_pillar_height(&mut self, index: usize, height: i32) {
        self.state.set_pillar_height(index, height);
    }

    pub fn on_hashed_seed(&mut self, hashed_seed: i64) {
        self.state.set_hashed_seed(hashed_seed);
    }

    pub fn solve(
        &self,
        registry: &StructureRegistry,
        rings: &Rings,
    ) -> CrackerResult {
        SeedCracker::solve(&self.state, registry, rings)
    }

    pub fn solve_cancellable(
        &self,
        registry: &StructureRegistry,
        rings: &Rings,
        cancel: Arc<AtomicBool>,
    ) -> CrackerResult {
        SeedCracker::solve_cancellable(&self.state, registry, rings, cancel)
    }

    pub fn get_state(&self) -> &CrackerState {
        &self.state
    }

    /// Get a mutable reference to the state (for advanced usage)
    pub fn get_state_mut(&mut self) -> &mut CrackerState {
        &mut self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_core::biome::obfuscate_seed;

    #[test]
    fn test_observer_basic() {
        let mut observer = SeedCrackerObserver::new();
        observer.on_structure_found("minecraft:villages".to_string(), 0, 0);
        observer.on_biome_found(100, 64, 200, "minecraft:plains".to_string());
        
        assert_eq!(observer.state.structures.len(), 1);
        assert_eq!(observer.state.biomes.len(), 1);
    }

    #[test]
    fn test_structure_seed_mask() {
        let world_seed = 765906787396911863i64;
        let structure_seed = (world_seed as u64) & 0xFFFFFFFFFFFF;
        
        // Should be 48 bits
        assert!(structure_seed < (1u64 << 48));
    }

    #[test]
    fn test_hashed_seed_computation() {
        let world_seed = 765906787396911863i64;
        let hashed = obfuscate_seed(world_seed);
        assert_ne!(hashed, 0);
    }
}
