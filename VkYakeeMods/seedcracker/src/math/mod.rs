//! Math utilities for seed cracking: LCG, chunk random, hashed seed, xoroshiro.

pub mod lcg;
pub mod chunk_rand;
pub mod hashed_seed;
pub mod xoroshiro;

pub use lcg::{JavaRandom, MULTIPLIER, ADDEND, MASK48, INV_MULTIPLIER, INV_ADDEND};
pub use chunk_rand::{get_region_seed, get_population_seed, get_structure_chunk_pos, test_structure_at_chunk};
pub use hashed_seed::{obfuscate_seed, find_world_seeds_from_hash, find_world_seeds_full};
pub use xoroshiro::Xoroshiro128PlusPlus;
