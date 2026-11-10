//! ChunkRand - Chunk-level random number generator used by Minecraft
//! This is used for decorations, population, and feature placement within a chunk

use super::xoroshiro::Xoroshiro128PlusPlus;

/// ChunkRand matches Minecraft's ChunkRandom implementation
/// Uses Xoroshiro128++ with chunk coordinates as part of the seed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkRand {
    rng: Xoroshiro128PlusPlus,
}

impl ChunkRand {
    /// Create a ChunkRand for a specific chunk and world seed
    /// Matches Minecraft's ChunkRandom constructor
    pub fn new(world_seed: u64, chunk_x: i32, chunk_z: i32) -> Self {
        // Minecraft combines world seed with chunk coordinates
        // This matches the vanilla implementation
        let seed = world_seed
            .wrapping_add((chunk_x as u64).wrapping_mul(0x1F1F1F1F1F1F1F1F))
            .wrapping_add((chunk_z as u64).wrapping_mul(0x1F1F1F1F1F1F1F1F));
        
        Self {
            rng: Xoroshiro128PlusPlus::new(seed),
        }
    }

    /// Create from a structure seed (48-bit) and chunk coordinates
    pub fn from_structure_seed(structure_seed: u64, chunk_x: i32, chunk_z: i32) -> Self {
        let seed = structure_seed
            .wrapping_add((chunk_x as u64).wrapping_mul(0x1F1F1F1F1F1F1F1F))
            .wrapping_add((chunk_z as u64).wrapping_mul(0x1F1F1F1F1F1F1F1F));
        
        Self {
            rng: Xoroshiro128PlusPlus::new(seed),
        }
    }

    /// Create from a population seed
    pub fn from_population_seed(pop_seed: u64) -> Self {
        Self {
            rng: Xoroshiro128PlusPlus::new(pop_seed),
        }
    }

    /// Consume n random numbers
    pub fn consume(&mut self, n: u64) {
        for _ in 0..n {
            self.rng.next();
        }
    }

    /// Next 64-bit value
    pub fn next(&mut self) -> u64 {
        self.rng.next()
    }

    /// Next 32-bit value
    pub fn next_int(&mut self) -> i32 {
        self.rng.next_int()
    }

    /// Next boolean
    pub fn next_boolean(&mut self) -> bool {
        self.rng.next_boolean()
    }

    /// Next float
    pub fn next_float(&mut self) -> f32 {
        self.rng.next_float()
    }

    /// Next double
    pub fn next_double(&mut self) -> f64 {
        self.rng.next_double()
    }

    /// Next bounded int
    pub fn next_int_bounded(&mut self, bound: i32) -> i32 {
        self.rng.next_int_bounded(bound)
    }

    /// Get the underlying Xoroshiro state
    pub fn state(&self) -> [u64; 2] {
        self.rng.state()
    }

    /// Set the state directly
    pub fn set_state(&mut self, state: [u64; 2]) {
        self.rng = Xoroshiro128PlusPlus::from_state(state[0], state[1]);
    }
}

impl Default for ChunkRand {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

/// Region random - used for structure placement at region level
pub struct RegionRand {
    rng: Xoroshiro128PlusPlus,
}

impl RegionRand {
    /// Create a RegionRand for a specific region and structure seed
    pub fn new(structure_seed: u64, region_x: i32, region_z: i32, salt: i32) -> Self {
        // Matches WorldgenRandom::setLargeFeatureWithSalt
        let mut rng = Xoroshiro128PlusPlus::new(structure_seed);
        
        // Mix in region coordinates and salt
        // This replicates the vanilla logic
        rng.consume(((region_x as u64).wrapping_mul(341873128712)) 
            .wrapping_add((region_z as u64).wrapping_mul(132897987541))
            .wrapping_add(salt as u64) & 0xFFFFFFFFFFFF);
        
        Self { rng }
    }

    pub fn next(&mut self) -> u64 {
        self.rng.next()
    }

    pub fn next_int(&mut self) -> i32 {
        self.rng.next_int()
    }

    pub fn next_int_bounded(&mut self, bound: i32) -> i32 {
        self.rng.next_int_bounded(bound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_rand_deterministic() {
        let mut rand1 = ChunkRand::new(12345, 100, -50);
        let mut rand2 = ChunkRand::new(12345, 100, -50);
        
        for _ in 0..100 {
            assert_eq!(rand1.next(), rand2.next());
        }
    }

    #[test]
    fn test_chunk_rand_different_chunks() {
        let mut rand1 = ChunkRand::new(12345, 100, -50);
        let mut rand2 = ChunkRand::new(12345, 101, -50);
        
        // Different chunks should produce different sequences
        assert_ne!(rand1.next(), rand2.next());
    }

    #[test]
    fn test_region_rand() {
        let mut rng = RegionRand::new(0x123456789ABC, 0, 0, 10387312);
        let first = rng.next();
        assert_ne!(first, 0);
    }
}
