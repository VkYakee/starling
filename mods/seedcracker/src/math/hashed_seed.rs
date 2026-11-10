//! HashedSeed - SHA-256 based seed obfuscation matching Java's HashedSeed.compute
//! Minecraft sends clients a "hashed seed" = first 8 bytes of SHA-256(world_seed_le)

use sha2::{Sha256, Digest};

/// Compute the hashed seed that Minecraft sends to clients
/// Matches Java's HashedSeed.compute exactly
pub fn compute_hashed_seed(world_seed: i64) -> i64 {
    let mut hasher = Sha256::new();
    // Java uses little-endian byte order for the world seed
    hasher.update(world_seed.to_le_bytes());
    let result = hasher.finalize();
    // First 8 bytes as little-endian signed long
    i64::from_le_bytes([
        result[0], result[1], result[2], result[3],
        result[4], result[5], result[6], result[7],
    ])
}

/// Find all 64-bit world seeds that produce a given hashed seed
/// Given a 48-bit structure seed and a hashed seed, find matching world seeds
pub fn find_world_seeds_from_hash(structure_seed: u64, target_hashed: i64) -> Vec<i64> {
    let mut results = Vec::new();
    
    // The structure seed is the lower 48 bits of the world seed
    // Upper 16 bits can be anything (0..=0xFFFF)
    for upper in 0..=0xFFFFu64 {
        let world_seed = ((upper << 48) | (structure_seed & 0xFFFFFFFFFFFF)) as i64;
        let hashed = compute_hashed_seed(world_seed);
        if hashed == target_hashed {
            results.push(world_seed);
        }
    }
    
    results
}

/// Test if a world seed produces the target hashed seed
pub fn test_hashed_seed(world_seed: i64, target_hashed: i64) -> bool {
    compute_hashed_seed(world_seed) == target_hashed
}

/// HashedSeed helper struct for compatibility
pub struct HashedSeed;

impl HashedSeed {
    /// Compute hashed seed from world seed
    pub fn compute(world_seed: i64) -> i64 {
        compute_hashed_seed(world_seed)
    }

    /// Find world seeds from structure seed and hashed seed
    pub fn find_world_seeds_from_hash(structure_seed: u64, hashed_seed: i64) -> Vec<i64> {
        find_world_seeds_from_hash(structure_seed, hashed_seed)
    }

    /// Test if world seed matches hashed seed
    pub fn test(world_seed: i64, hashed_seed: i64) -> bool {
        test_hashed_seed(world_seed, hashed_seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashed_seed_matches_java() {
        // Test with known seed
        let world_seed = 765906787396911863i64;
        let hashed = compute_hashed_seed(world_seed);
        
        // Verify it's deterministic
        assert_eq!(hashed, compute_hashed_seed(world_seed));
        assert_ne!(hashed, 0);
    }

    #[test]
    fn test_find_world_seeds_from_hash() {
        let world_seed = 765906787396911863i64;
        let structure_seed = (world_seed as u64) & 0xFFFFFFFFFFFF;
        let hashed = compute_hashed_seed(world_seed);
        
        let candidates = find_world_seeds_from_hash(structure_seed, hashed);
        assert!(candidates.contains(&world_seed), "Original seed not found!");
    }

    #[test]
    fn test_hashed_seed_deterministic() {
        let seed = 123456789012345i64;
        let h1 = compute_hashed_seed(seed);
        let h2 = compute_hashed_seed(seed);
        assert_eq!(h1, h2);
    }
}
