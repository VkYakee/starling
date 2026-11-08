use crate::math::lcg::{JavaRandom, MASK48};

pub const K1: u64 = 341873128712;
pub const K2: u64 = 132897987541;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MCVersion {
    Old1_12,
    Modern1_13,
    Modern1_18,
    Modern1_20,
    Latest,
}

pub struct ChunkRand;

impl ChunkRand {
    /// Computes the population seed for a chunk in Minecraft 1.13+.
    #[inline]
    pub fn get_population_seed(world_seed: u64, chunk_x: i32, chunk_z: i32) -> u64 {
        let x_part = (chunk_x as i64 as u64).wrapping_mul(K1);
        let z_part = (chunk_z as i64 as u64).wrapping_mul(K2);
        (world_seed.wrapping_add(x_part).wrapping_add(z_part)) & MASK48
    }

    /// Reverses the 48-bit world seed (or structure seed) from population seed and chunk coordinates.
    #[inline]
    pub fn reverse_population_seed(population_seed: u64, chunk_x: i32, chunk_z: i32) -> u64 {
        let x_part = (chunk_x as i64 as u64).wrapping_mul(K1);
        let z_part = (chunk_z as i64 as u64).wrapping_mul(K2);
        (population_seed.wrapping_sub(x_part).wrapping_sub(z_part)) & MASK48
    }

    /// Computes the decorator seed given population seed and feature index/salt.
    #[inline]
    pub fn get_decorator_seed(population_seed: u64, salt: u64) -> u64 {
        (population_seed.wrapping_add(salt)) & MASK48
    }

    /// Computes the region seed for a structure in Minecraft 1.13+.
    #[inline]
    pub fn get_region_seed(world_seed: u64, region_x: i32, region_z: i32, salt: u64) -> u64 {
        let x_part = (region_x as i64 as u64).wrapping_mul(K1);
        let z_part = (region_z as i64 as u64).wrapping_mul(K2);
        (world_seed.wrapping_add(x_part).wrapping_add(z_part).wrapping_add(salt)) & MASK48
    }

    /// Tests if a chunk is a slime chunk for a given world seed.
    pub fn is_slime_chunk(world_seed: u64, chunk_x: i32, chunk_z: i32) -> bool {
        let seed = (world_seed as i64)
            .wrapping_add(((chunk_x as i64).wrapping_mul(chunk_x as i64)).wrapping_mul(0x4c1906))
            .wrapping_add((chunk_x as i64).wrapping_mul(0x5ac0db))
            .wrapping_add(((chunk_z as i64).wrapping_mul(chunk_z as i64)).wrapping_mul(0x4307a7))
            .wrapping_add((chunk_z as i64).wrapping_mul(0x5f24f))
            ^ 0x3ad8025f;

        let mut rand = JavaRandom::new(seed as u64);
        rand.next_int_bounded(10) == 0
    }
}
