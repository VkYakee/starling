//! SeedCracker - High-performance Minecraft seed cracking engine for Horizon/Starling
//!
//! This crate provides seed cracking capabilities using Horizon's native worldgen
//! APIs as oracles, guaranteeing 100% parity with the game's actual generation.

pub mod assets;
pub mod biome;
pub mod bedrock;
pub mod command;
pub mod math;
pub mod pillar;
pub mod structure;
pub mod timemachine;

// Re-export public API
pub use assets::{AssetManager, ASSET_MANAGER, tr};
pub use biome::{BiomeCracker, BiomeCrackerResult, BiomeObserver, ObservedBiome};
pub use bedrock::{BedrockCracker, BedrockCrackerResult, BedrockObserver, BedrockObservation};
pub use command::SeedCrackerCommand;
pub use math::{chunk_rand, hashed_seed, lcg, xoroshiro};
pub use pillar::{EndPillars, PillarCracker, PillarCrackerResult, PillarObserver, PillarObservations};
pub use structure::{StructureCracker, StructureCrackerResult, StructureObserver, ObservedStructure};
pub use timemachine::{TimeMachine, TimeMachineResult, CrackerState as LegacyCrackerState};

/// Complete cracker state containing all observations
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CrackerState {
    // Structure observations
    pub structures: Vec<ObservedStructure>,
    // Biome observations
    pub biomes: Vec<ObservedBiome>,
    // Bedrock observations
    pub bedrock_chunks: Vec<BedrockCracker>,
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
            target_mc_version: 120, // 1.20.x
            ..Default::default()
        }
    }

    pub fn add_structure(&mut self, set_name: String, chunk_x: i32, chunk_z: i32) {
        self.structures.push(ObservedStructure { set_name, chunk_x, chunk_z });
    }

    pub fn add_biome(&mut self, x: i32, y: i32, z: i32, biome_id: i32, biome_name: String) {
        self.biomes.push(ObservedBiome { x, y, z, biome_id, biome_name });
    }

    pub fn add_bedrock_chunk(&mut self, chunk_x: i32, chunk_z: i32, mask: u64) {
        let mut cracker = BedrockCracker::new(chunk_x, chunk_z);
        cracker.add_bedrock_mask(mask);
        self.bedrock_chunks.push(cracker);
    }

    pub fn set_pillar_height(&mut self, index: usize, height: i32) {
        self.pillars.set_height(index, height);
    }

    pub fn set_hashed_seed(&mut self, hashed_seed: i64) {
        self.hashed_seed = Some(hashed_seed);
    }
}

/// Complete cracker results
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
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
        registry: &horizon_world::worldgen::structure::start::StructureRegistry,
        rings: &horizon_world::worldgen::structure::start::Rings,
    ) -> CrackerResult {
        Self::solve_cancellable(state, registry, rings, std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)))
    }

    /// Solve with cancellation support
    pub fn solve_cancellable(
        state: &CrackerState,
        registry: &horizon_world::worldgen::structure::start::StructureRegistry,
        rings: &horizon_world::worldgen::structure::start::Rings,
        cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> CrackerResult {
        let mut result = CrackerResult::new();
        let cancelled = || cancel.load(std::sync::atomic::Ordering::Relaxed);

        // Phase 1: Structure cracking (most constraining)
        if !state.structures.is_empty() {
            let cracker = StructureCracker::with_rings(
                registry,
                state.structures.clone(),
                rings,
            );
            
            // Exhaustive search for structure seeds
            let structure_seeds = cracker.search_exhaustive(&cancel);
            result.structure_seeds = structure_seeds;
            
            if cancelled() {
                result.cancelled = true;
                return result;
            }
        }

        // Phase 2: Pillar cracking (independent, fast)
        if state.pillars.observed_count() >= 3 {
            result.pillar_seeds = EndPillars::find_pillar_seeds(&state.pillars.heights);
            result.pillar_seeds.sort_unstable();
            result.pillar_seeds.dedup();
        }

        // Phase 3: Cross-reference structure seeds with pillar seeds
        // Pillar seed is derived from structure seed: pillar_seed = (LCG^2(structure_seed ^ MULTIPLIER) >> 16) & 0xFFFF
        if !result.structure_seeds.is_empty() && !result.pillar_seeds.is_empty() {
            let mut filtered = Vec::new();
            for struct_seed in &result.structure_seeds {
                let pillar_seed = EndPillars::pillar_seed_from_structure_seed(*struct_seed);
                if result.pillar_seeds.contains(&pillar_seed) {
                    filtered.push(*struct_seed);
                }
            }
            result.structure_seeds = filtered;
        }

        // Phase 4: Bedrock cracking
        if !state.bedrock_chunks.is_empty() && !result.structure_seeds.is_empty() {
            let mut filtered = Vec::new();
            for struct_seed in &result.structure_seeds {
                let mut matches_all = true;
                for cracker in &state.bedrock_chunks {
                    let pop_seed = BedrockCracker::get_population_seed(*struct_seed, cracker.chunk_x, cracker.chunk_z);
                    if !cracker.test_population_seed(pop_seed) {
                        matches_all = false;
                        break;
                    }
                }
                if matches_all {
                    filtered.push(*struct_seed);
                }
            }
            result.structure_seeds = filtered;
        }

        // Phase 5: Lift structure seeds to world seeds
        if !result.structure_seeds.is_empty() {
            use std::collections::HashSet;
            let mut world_seeds = HashSet::new();
            
            for structure_seed in &result.structure_seeds {
                if let Some(hashed) = state.hashed_seed {
                    // Use hashed seed to find the exact world seed
                    let candidates = math::hashed_seed::find_world_seeds_from_hash(*structure_seed, hashed);
                    for ws in candidates {
                        world_seeds.insert(ws);
                    }
                } else {
                    // No constraints - return all possible world seeds for this structure seed
                    for upper in 0..=0xFFFFu64 {
                        let world_seed = ((upper << 48) | structure_seed) as i64;
                        world_seeds.insert(world_seed);
                    }
                }
            }
            
            result.world_seeds = world_seeds.into_iter().collect();
            result.world_seeds.sort_unstable();
        }

        // Phase 6: Validate results against all observations
        if !result.world_seeds.is_empty() {
            let validated: Vec<i64> = result.world_seeds
                .iter()
                .filter(|&&ws| Self::validate_world_seed(ws, state, registry, rings))
                .copied()
                .collect();
            
            result.validated = !validated.is_empty();
            result.world_seeds = validated;
            result.resolution_complete = result.validated && 
                (state.hashed_seed.is_some() || !state.biomes.is_empty());
        }

        result
    }

    /// Validate a world seed against all observations
    fn validate_world_seed(
        world_seed: i64, 
        state: &CrackerState,
        registry: &horizon_world::worldgen::structure::start::StructureRegistry,
        rings: &horizon_world::worldgen::structure::start::Rings,
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

        // Check biome observations
        if !state.biomes.is_empty() {
            let hashed = horizon_core::biome::obfuscate_seed(world_seed);
            let mut biome_cracker = BiomeCracker::new(state.biomes.clone());
            if !biome_cracker.test_hashed_seed(hashed) {
                return false;
            }
        }

        // Check bedrock observations
        for cracker in &state.bedrock_chunks {
            let pop_seed = BedrockCracker::get_population_seed(structure_seed, cracker.chunk_x, cracker.chunk_z);
            if !cracker.test_population_seed(pop_seed) {
                return false;
            }
        }

        true
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

    pub fn on_biome_found(&mut self, x: i32, y: i32, z: i32, biome_id: i32, biome_name: String) {
        self.state.add_biome(x, y, z, biome_id, biome_name);
    }

    pub fn on_bedrock_chunk(&mut self, chunk_x: i32, chunk_z: i32, mask: u64) {
        self.state.add_bedrock_chunk(chunk_x, chunk_z, mask);
    }

    pub fn on_pillar_height(&mut self, index: usize, height: i32) {
        self.state.set_pillar_height(index, height);
    }

    pub fn on_hashed_seed(&mut self, hashed_seed: i64) {
        self.state.set_hashed_seed(hashed_seed);
    }

    pub fn solve(
        &self,
        registry: &horizon_world::worldgen::structure::start::StructureRegistry,
        rings: &horizon_world::worldgen::structure::start::Rings,
    ) -> CrackerResult {
        SeedCracker::solve(&self.state, registry, rings)
    }

    pub fn solve_cancellable(
        &self,
        registry: &horizon_world::worldgen::structure::start::StructureRegistry,
        rings: &horizon_world::worldgen::structure::start::Rings,
        cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> CrackerResult {
        SeedCracker::solve_cancellable(&self.state, registry, rings, cancel)
    }

    pub fn get_state(&self) -> &CrackerState {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observer_basic() {
        let mut observer = SeedCrackerObserver::new();
        observer.on_structure_found("minecraft:villages".to_string(), 0, 0);
        observer.on_biome_found(100, 64, 200, 1, "plains".to_string());
        
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
}
