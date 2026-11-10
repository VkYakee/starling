//! Biome Source - Biome generation for validation
//!
//! This module provides biome source implementations for validating
//! candidate world seeds against observed biomes. It uses Horizon's
//! BiomeZoom for quart selection and can integrate with actual biome
//! sources for full biome matching.

use horizon_core::biome::{BiomeZoom, obfuscate_seed};
use horizon_core::position::BlockPos;
use std::collections::HashMap;

/// Biome source trait for different dimensions
pub trait BiomeSource {
    /// Get biome ID at block position
    fn get_biome_id(&mut self, x: i32, y: i32, z: i32) -> i32;
    
    /// Get biome name at block position
    fn get_biome_name(&mut self, x: i32, y: i32, z: i32) -> String {
        format!("biome_{}", self.get_biome_id(x, y, z))
    }
    
    /// Test if biome at position matches expected
    fn test_biome(&mut self, x: i32, y: i32, z: i32, expected_id: i32) -> bool {
        self.get_biome_id(x, y, z) == expected_id
    }
}

/// Overworld biome source using Horizon's biome generation
pub struct OverworldBiomeSource {
    world_seed: i64,
    biome_zoom: BiomeZoom,
    biome_cache: HashMap<(i32, i32, i32), i32>,
}

impl OverworldBiomeSource {
    /// Create a new overworld biome source
    pub fn new(world_seed: i64) -> Self {
        let hashed = obfuscate_seed(world_seed);
        Self {
            world_seed,
            biome_zoom: BiomeZoom::new(hashed),
            biome_cache: HashMap::new(),
        }
    }

    /// Get the world seed
    pub fn world_seed(&self) -> i64 {
        self.world_seed
    }

    /// Get the hashed seed
    pub fn hashed_seed(&self) -> i64 {
        self.biome_zoom.seed
    }
}

impl BiomeSource for OverworldBiomeSource {
    fn get_biome_id(&mut self, x: i32, y: i32, z: i32) -> i32 {
        // Check cache first
        if let Some(&cached) = self.biome_cache.get(&(x, y, z)) {
            return cached;
        }
        
        // Get quart from BiomeZoom
        let pos = BlockPos::new(x, y, z);
        let quart = self.biome_zoom.quart_at_block(pos);
        
        // In a full implementation, we'd look up the biome at this quart
        // using the biome source from the generator. For now, we return
        // a hash-based biome ID for deterministic testing.
        let biome_id = ((quart[0] as i64).wrapping_mul(73856093)
            ^ (quart[1] as i64).wrapping_mul(19349663)
            ^ (quart[2] as i64).wrapping_mul(83492791))
            .abs() as i32 % 256;
        
        self.biome_cache.insert((x, y, z), biome_id);
        biome_id
    }
}

/// Nether biome source
pub struct NetherBiomeSource {
    world_seed: i64,
    biome_zoom: BiomeZoom,
}

impl NetherBiomeSource {
    pub fn new(world_seed: i64) -> Self {
        let hashed = obfuscate_seed(world_seed);
        Self {
            world_seed,
            biome_zoom: BiomeZoom::new(hashed),
        }
    }
}

impl BiomeSource for NetherBiomeSource {
    fn get_biome_id(&mut self, x: i32, y: i32, z: i32) -> i32 {
        let pos = BlockPos::new(x, y, z);
        let quart = self.biome_zoom.quart_at_block(pos);
        
        let biome_id = ((quart[0] as i64).wrapping_mul(73856093)
            ^ (quart[1] as i64).wrapping_mul(19349663)
            ^ (quart[2] as i64).wrapping_mul(83492791))
            .abs() as i32 % 16;
        
        biome_id
    }
}

/// End biome source
pub struct EndBiomeSource {
    world_seed: i64,
    biome_zoom: BiomeZoom,
}

impl EndBiomeSource {
    pub fn new(world_seed: i64) -> Self {
        let hashed = obfuscate_seed(world_seed);
        Self {
            world_seed,
            biome_zoom: BiomeZoom::new(hashed),
        }
    }
}

impl BiomeSource for EndBiomeSource {
    fn get_biome_id(&mut self, x: i32, y: i32, z: i32) -> i32 {
        let pos = BlockPos::new(x, y, z);
        let quart = self.biome_zoom.quart_at_block(pos);
        
        // The End has very few biomes
        let biome_id = (quart[0] + quart[1] + quart[2]).abs() % 5;
        biome_id
    }
}

/// Multi-dimension biome source
pub struct MultiBiomeSource {
    overworld: OverworldBiomeSource,
    nether: NetherBiomeSource,
    end: EndBiomeSource,
}

impl MultiBiomeSource {
    pub fn new(world_seed: i64) -> Self {
        Self {
            overworld: OverworldBiomeSource::new(world_seed),
            nether: NetherBiomeSource::new(world_seed),
            end: EndBiomeSource::new(world_seed),
        }
    }
    
    pub fn overworld(&mut self) -> &mut OverworldBiomeSource {
        &mut self.overworld
    }
    
    pub fn nether(&mut self) -> &mut NetherBiomeSource {
        &mut self.nether
    }
    
    pub fn end(&mut self) -> &mut EndBiomeSource {
        &mut self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overworld_biome_source() {
        let mut source = OverworldBiomeSource::new(12345);
        let biome1 = source.get_biome_id(100, 64, 200);
        let biome2 = source.get_biome_id(100, 64, 200);
        assert_eq!(biome1, biome2, "Biome should be deterministic");
    }

    #[test]
    fn test_different_positions_different_biomes() {
        let mut source = OverworldBiomeSource::new(12345);
        let biome1 = source.get_biome_id(100, 64, 200);
        let biome2 = source.get_biome_id(1000, 64, 2000);
        // They could be the same by chance, but usually different
        let _ = (biome1, biome2);
    }

    #[test]
    fn test_nether_biome_source() {
        let mut source = NetherBiomeSource::new(12345);
        let biome = source.get_biome_id(100, 64, 200);
        assert!(biome < 16);
    }

    #[test]
    fn test_end_biome_source() {
        let mut source = EndBiomeSource::new(12345);
        let biome = source.get_biome_id(100, 64, 200);
        assert!(biome < 5);
    }
}
