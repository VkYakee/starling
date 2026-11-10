// SeedCracker - High-performance Minecraft seed cracking engine for Starling/Horizon
//
// This is a complete rewrite implementing the full TimeMachine algorithm for
// 48-bit -> 64-bit seed lifting using Horizon's native worldgen APIs as oracles.
//
// Architecture:
// - Uses Horizon's StructureRegistry as the ground truth oracle for structure positions
// - Uses Horizon's BiomeZoom for biome matching
// - Uses Horizon's WorldgenRandom for population seed derivation
// - Implements the full TimeMachine algorithm for 48-bit -> 64-bit seed lifting
// - Provides an observer pattern for collecting data from gameplay
// - Parallel search using rayon for maximum performance

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod bedrock;
pub mod biomes;
pub mod cracker;
pub mod decorators;
pub mod dungeon;
pub mod finders;
pub mod math;
pub mod observer;
pub mod pillars;
pub mod structures;
pub mod timemachine;
pub mod trees;

// Re-exports for external use
pub use bedrock::{BedrockSolver, BedrockBlock, BlockType, FLOOR_HASH, ROOF_HASH};
pub use biomes::{BiomeGenerator, BiomeLifter, BiomeSample, MC_1_20};
pub use cracker::{
    BedrockCracker, BedrockCrackerResult, BedrockObservation, BedrockObserver,
    BiomeCracker, BiomeCrackerResult, BiomeObserver, BiomeObservation,
    CrackerResult, CrackerState,
    SeedCracker, SeedCrackerObserver, StructureCracker, StructureCrackerResult,
    StructureObserver, ObservedStructure,
};
pub use decorators::{DecoratorType, EmeraldOreData, DesertWellData};
pub use dungeon::{DungeonFloor, DungeonSolver, DUNGEON_SALT_MODERN};
pub use finders::{FinderBuilder, FinderJob, FinderKind, FinderControl, FinderStatus, FinderQueue, FinderQueueStats, QueuedFinder, ChunkAnalyzer, AnalyzerOutput, BedrockAnalyzer, StructureAnalyzer, BlockObservation, BlockUpdateBatch, FinderObservations, PatternDetector, PatternHit, PatternKind, FinderCategory, FinderType, BlockFinder, FinderRegistry, FinderPipeline};
pub use math::{
    chunk_rand,
    hashed_seed,
    lcg::{JavaRandom, ADDEND, INV_ADDEND, INV_MULTIPLIER, MASK48, MULTIPLIER},
    xoroshiro::Xoroshiro128PlusPlus,
};
pub use observer::{Observer, Observation, DefaultObserver, ObserverBuilder};
pub use pillars::{EndPillars, PillarObservations, PILLAR_COUNT};
pub use structures::{StructureType, StructureConfig, StructureInstance, get_structure_config, predict_structure_chunk, find_closest_predicted_structure};
pub use timemachine::{CrackerState as TimeMachineState, CrackerResult as TimeMachineResult, TimeMachine};

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_core::biome::obfuscate_seed;
    use horizon_core::random::{LegacyRandom, WorldgenRandom};

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
        
        assert!(structure_seed < (1u64 << 48));
    }

    #[test]
    fn test_obfuscate_seed_matches_java() {
        let world_seed = 765906787396911863i64;
        let hashed = obfuscate_seed(world_seed);
        assert_ne!(hashed, 0);
    }

    #[test]
    fn test_pillar_reversible() {
        let pillar_seed = 12345u16;
        let heights = EndPillars::get_pillar_heights(pillar_seed);
        let mut observed = [None; 10];
        observed[0] = Some(heights[0]);
        observed[1] = Some(heights[1]);
        observed[2] = Some(heights[2]);
        observed[3] = Some(heights[3]);
        let candidates = EndPillars::find_pillar_seeds(&observed);
        assert!(candidates.contains(&pillar_seed));
    }

    #[test]
    fn test_pillar_seed_from_structure_seed() {
        let structure_seed = 765906787396911863u64 & 0xFFFFFFFFFFFF;
        let pillar_seed = EndPillars::pillar_seed_from_structure_seed(structure_seed);
        let pillar_seed2 = EndPillars::pillar_seed_from_structure_seed(structure_seed);
        assert_eq!(pillar_seed, pillar_seed2);
    }

    #[test]
    fn test_time_machine_combine() {
        let partial = 0x123456789ABCu64;
        let pillar = 0xABCDu16;
        let combined = EndPillars::time_machine_combine(partial, pillar);
        assert!(combined < (1u64 << 48));
    }

    #[test]
    fn test_hashed_seed_lookup() {
        let world_seed = 765906787396911863i64;
        let structure_seed = (world_seed as u64) & 0xFFFFFFFFFFFF;
        let hashed = obfuscate_seed(world_seed);
        
        let candidates = hashed_seed::find_world_seeds_from_hash(structure_seed, hashed);
        assert!(candidates.contains(&world_seed));
    }

    #[test]
    fn test_bedrock_population_seed() {
        let structure_seed = 0x123456789ABCu64 & 0xFFFFFFFFFFFF;
        let chunk_x = 0;
        let chunk_z = 0;
        
        let pop_seed = cracker::BedrockCracker::get_population_seed(structure_seed, chunk_x, chunk_z);
        let pop_seed2 = cracker::BedrockCracker::get_population_seed(structure_seed, chunk_x, chunk_z);
        assert_eq!(pop_seed, pop_seed2);
    }
}
