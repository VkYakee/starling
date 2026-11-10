//! Observer pattern for collecting seed cracking data from gameplay
//!
//! This module provides a unified observer interface that combines all
//! individual cracker observers into a single entry point for collecting
//! observations from the game client.

use crate::cracker::{CrackerState, CrackerResult, SeedCrackerObserver};
use horizon_world::worldgen::structure::start::{StructureRegistry, Rings};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

/// Main observer trait for collecting cracking data
pub trait Observer {
    /// Called when a structure is found
    fn on_structure_found(&mut self, set_name: String, chunk_x: i32, chunk_z: i32);
    
    /// Called when a biome is observed
    fn on_biome_found(&mut self, x: i32, y: i32, z: i32, biome_name: String);
    
    /// Called when a bedrock column is observed
    fn on_bedrock_column(&mut self, chunk_x: i32, chunk_z: i32, roof_mask: u64, floor_mask: u64);
    
    /// Called when an end pillar height is observed
    fn on_pillar_height(&mut self, index: usize, height: i32);
    
    /// Called when hashed seed is received (from SpawnInfo packet)
    fn on_hashed_seed(&mut self, hashed_seed: i64);
}

/// An observation collected from gameplay
#[derive(Debug, Clone)]
pub enum Observation {
    Structure(String, i32, i32),
    Biome(i32, i32, i32, String),
    Bedrock(i32, i32, u64, u64),
    Pillar(usize, i32),
    HashedSeed(i64),
}

/// Re-export the main observer
pub use crate::cracker::SeedCrackerObserver as DefaultObserver;

/// Builder for creating an observer with initial state
pub struct ObserverBuilder {
    state: CrackerState,
}

impl ObserverBuilder {
    pub fn new() -> Self {
        Self {
            state: CrackerState::new(),
        }
    }
    
    pub fn with_target_version(mut self, version: i32) -> Self {
        self.state.target_mc_version = version;
        self
    }
    
    pub fn with_hashed_seed(mut self, hashed: i64) -> Self {
        self.state.hashed_seed = Some(hashed);
        self
    }
    
    pub fn build(self) -> SeedCrackerObserver {
        SeedCrackerObserver { state: self.state }
    }
}

impl Default for ObserverBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observer_builder() {
        let observer = ObserverBuilder::new()
            .with_target_version(34) // 1.20
            .with_hashed_seed(12345)
            .build();
        
        assert_eq!(observer.state.target_mc_version, 34);
        assert_eq!(observer.state.hashed_seed, Some(12345));
    }
}
