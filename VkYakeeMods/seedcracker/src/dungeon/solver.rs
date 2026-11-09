use rayon::prelude::*;

use crate::dungeon::floor::{DungeonFloor, DungeonFloorBlock};
use crate::math::chunk_rand::ChunkRand;
use crate::math::lcg::{JavaRandom, MASK48, MULTIPLIER};

pub const DUNGEON_SALT_MODERN: u64 = 30002;

pub struct DungeonSolver;

impl DungeonSolver {
    /// Validates if a PRNG state produces the exact dungeon floor pattern.
    #[inline]
    pub fn test_floor_sequence(seed: u64, pattern: &[DungeonFloorBlock]) -> bool {
        let mut cur_seed = seed;
        for &block in pattern {
            // Next seed
            cur_seed = JavaRandom::next_seed(cur_seed);
            // nextInt(4) is top 2 bits of seed (bits 46 and 47)
            let val = ((cur_seed >> 46) & 3) as i32;
            match block {
                DungeonFloorBlock::Cobblestone => {
                    if val != 0 {
                        return false;
                    }
                }
                DungeonFloorBlock::MossyCobblestone => {
                    if val == 0 {
                        return false;
                    }
                }
                DungeonFloorBlock::Unknown => {}
            }
        }
        true
    }

    /// Bounded candidate scan. search_range_bits is the candidate count exponent,
    /// not a partition count across all 2^48 states.
    pub fn crack_decorator_seed(floor: &DungeonFloor, search_range_bits: u32) -> Vec<u64> {
        Self::crack_decorator_seed_range(floor, 0, 1_u64 << search_range_bits.min(24), &std::sync::atomic::AtomicBool::new(false))
    }

    /// Resumable/cancellable scan of an explicit decorator-seed interval.
    pub fn crack_decorator_seed_range(
        floor: &DungeonFloor, start: u64, end: u64,
        cancel: &std::sync::atomic::AtomicBool,
    ) -> Vec<u64> {
        if floor.calls.is_empty() || start >= end { return Vec::new(); }
        (start..end.min(1_u64 << 48)).into_par_iter().filter(|seed| {
            !cancel.load(std::sync::atomic::Ordering::Relaxed)
                && Self::test_floor_sequence(*seed, &floor.calls)
        }).collect()
    }

    /// Reverses a confirmed decorator seed back to candidate structure seeds.
    pub fn decorator_to_structure_seeds(
        decorator_seed: u64,
        chunk_x: i32,
        chunk_z: i32,
        salt: u64,
    ) -> Vec<u64> {
        let mut results = Vec::new();
        // Decorator seed is initialized from: (pop_seed + salt) ^ MULTIPLIER
        let base_pop_seed = ((decorator_seed ^ MULTIPLIER).wrapping_sub(salt)) & MASK48;
        let structure_seed = ChunkRand::reverse_population_seed(base_pop_seed, chunk_x, chunk_z);
        results.push(structure_seed);

        // Also check if dungeon was generated after failed attempts (each failed attempt uses 5 calls)
        let (mult_m5, add_m5) = JavaRandom::combine_steps(-5);
        let mut prev_seed = decorator_seed;
        for _ in 0..8 {
            prev_seed = (prev_seed.wrapping_mul(mult_m5).wrapping_add(add_m5)) & MASK48;
            let pop_seed = ((prev_seed ^ MULTIPLIER).wrapping_sub(salt)) & MASK48;
            results.push(ChunkRand::reverse_population_seed(pop_seed, chunk_x, chunk_z));
        }

        results
    }
}
