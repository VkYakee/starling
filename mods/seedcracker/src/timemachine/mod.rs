//! TimeMachine - The complete seed cracking pipeline that combines all cracking methods
//! This is the main entry point that coordinates structures, pillars, biome, bedrock,
//! and hashed seed cracking to find the exact world seed.

use crate::biome::{BiomeCracker, ObservedBiome};
use crate::bedrock::{BedrockCracker, BedrockCrackerResult};
use crate::math::hashed_seed::HashedSeed;
use crate::pillar::{EndPillars, PillarObservations};
use crate::structure::{StructureCracker, ObservedStructure};
use horizon_world::worldgen::structure::start::{StructureRegistry, Rings};
use std::sync::atomic::{AtomicBool, Ordering};

/// Legacy cracker state for compatibility
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CrackerState {
    pub structures: Vec<ObservedStructure>,
    pub biomes: Vec<ObservedBiome>,
    pub bedrock_chunks: Vec<BedrockCracker>,
    pub pillars: PillarObservations,
    pub hashed_seed: Option<i64>,
    pub target_mc_version: i32,
}

impl CrackerState {
    pub fn new() -> Self {
        Self {
            target_mc_version: 120,
            ..Default::default()
        }
    }
}

/// Complete cracker results
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TimeMachineResult {
    pub structure_seeds: Vec<u64>,
    pub world_seeds: Vec<i64>,
    pub pillar_seeds: Vec<u16>,
    pub bedrock_seeds: Vec<u64>,
    pub cancelled: bool,
    pub validated: bool,
    pub resolution_complete: bool,
}

/// TimeMachine - The complete seed cracking engine
pub struct TimeMachine;

impl TimeMachine {
    /// Run the complete seed cracking pipeline
    pub fn crack(
        state: &CrackerState,
        registry: &StructureRegistry,
        rings: &Rings,
    ) -> TimeMachineResult {
        Self::crack_cancellable(state, registry, rings, std::sync::Arc::new(AtomicBool::new(false)))
    }

    /// Run with cancellation support
    pub fn crack_cancellable(
        state: &CrackerState,
        registry: &StructureRegistry,
        rings: &Rings,
        cancel: std::sync::Arc<AtomicBool>,
    ) -> TimeMachineResult {
        let mut result = TimeMachineResult::new();
        let cancelled = || cancel.load(Ordering::Relaxed);

        // Phase 1: Structure cracking (most constraining)
        if !state.structures.is_empty() {
            let cracker = StructureCracker::with_rings(
                registry,
                state.structures.clone(),
                rings,
            );
            
            let structure_seeds = cracker.search_exhaustive(&cancel);
            result.structure_seeds = structure_seeds;
            
            if cancelled() {
                result.cancelled = true;
                return result;
            }
        }

        // Phase 2: Pillar cracking
        if state.pillars.observed_count() >= 3 {
            result.pillar_seeds = EndPillars::find_pillar_seeds(&state.pillars.heights);
            result.pillar_seeds.sort_unstable();
            result.pillar_seeds.dedup();
        }

        // Phase 3: Cross-reference structure seeds with pillar seeds
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

        // Phase 4: Bedrock filtering
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
                    let candidates = HashedSeed::find_world_seeds_from_hash(*structure_seed, hashed);
                    for ws in candidates {
                        world_seeds.insert(ws);
                    }
                } else {
                    for upper in 0..=0xFFFFu64 {
                        let world_seed = ((upper << 48) | structure_seed) as i64;
                        world_seeds.insert(world_seed);
                    }
                }
            }
            
            result.world_seeds = world_seeds.into_iter().collect();
            result.world_seeds.sort_unstable();
        }

        // Phase 6: Validate against all observations
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

/// Solver worker for background cracking
pub struct SolverWorker {
    cancel_flag: std::sync::Arc<AtomicBool>,
}

impl SolverWorker {
    pub fn new() -> Self {
        Self {
            cancel_flag: std::sync::Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel_flag.load(Ordering::Relaxed)
    }

    pub fn solve(
        &self,
        state: CrackerState,
        registry: &StructureRegistry,
        rings: &Rings,
    ) -> TimeMachineResult {
        TimeMachine::crack_cancellable(&state, registry, rings, self.cancel_flag.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cracker_state_new() {
        let state = CrackerState::new();
        assert_eq!(state.target_mc_version, 120);
    }
}
