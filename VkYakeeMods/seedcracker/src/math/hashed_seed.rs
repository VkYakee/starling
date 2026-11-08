use rayon::prelude::*;
use sha2::{Digest, Sha256};

pub struct HashedSeed;

impl HashedSeed {
    /// Computes the Minecraft 1.15+ hashed seed for a 64-bit world seed.
    /// This matches Mojang's Hashing.sha256().hashLong(worldSeed).asLong()
    pub fn compute(world_seed: i64) -> i64 {
        let mut hasher = Sha256::new();
        hasher.update(&world_seed.to_le_bytes());
        let result = hasher.finalize();
        let mut first8 = [0u8; 8];
        first8.copy_from_slice(&result[0..8]);
        i64::from_le_bytes(first8)
    }

    /// Lifts a 48-bit structure seed to 64-bit world seed using a target hashed seed.
    /// Tests all 65,536 upper bit combinations in parallel using Rayon.
    pub fn find_world_seeds_from_hash(structure_seed: u64, target_hashed_seed: i64) -> Vec<i64> {
        let base_48 = structure_seed & 0xFFFFFFFFFFFF;
        (0..=0xFFFFu64)
            .into_par_iter()
            .filter_map(|upper| {
                let candidate = ((upper << 48) | base_48) as i64;
                if Self::compute(candidate) == target_hashed_seed {
                    Some(candidate)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashed_seed_reversible() {
        let world_seed = 765906787396911863i64;
        let hashed = HashedSeed::compute(world_seed);
        let structure_seed = (world_seed as u64) & 0xFFFFFFFFFFFF;
        let candidates = HashedSeed::find_world_seeds_from_hash(structure_seed, hashed);
        assert_eq!(candidates, vec![world_seed]);
    }
}
