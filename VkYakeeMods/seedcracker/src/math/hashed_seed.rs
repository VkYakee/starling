//! SHA-256 based world seed obfuscation matching Java's HashedSeed.compute.
//! 
//! In Minecraft 1.15+, the world seed sent to clients is obfuscated via SHA-256:
//! hashed_seed = first_8_bytes_of_SHA256(little_endian(world_seed)) as i64

use sha2::{Digest, Sha256};

/// Compute the hashed (obfuscated) seed exactly as Java's HashedSeed.compute does.
/// 
/// Java implementation:
/// ```java
/// public static long compute(long worldSeed) {
///     return new BigInteger(1, new SecureHashAlgorithm().digest(
///         ByteBuffer.allocate(8).order(ByteOrder.LITTLE_ENDIAN).putLong(worldSeed).array()
///     )).longValue();
/// }
/// ```
/// 
/// Note: Java uses only the first 8 bytes of the SHA-256 digest, interpreted as a signed long.
pub fn obfuscate_seed(world_seed: i64) -> i64 {
    let mut hasher = Sha256::new();
    hasher.update(world_seed.to_le_bytes());
    let result = hasher.finalize();
    
    // Take first 8 bytes as little-endian signed i64
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&result[0..8]);
    i64::from_le_bytes(bytes)
}

/// Find all world seeds that produce a given hashed seed, given a 48-bit structure seed.
/// 
/// Since the hashed seed depends on the full 64-bit world seed, and we know the lower
/// 48 bits (structure seed), we only need to search the upper 16 bits (65536 possibilities).
pub fn find_world_seeds_from_hash(structure_seed: u64, target_hash: i64) -> Vec<i64> {
    let mut results = Vec::new();
    let base_48 = structure_seed & 0xFFFFFFFFFFFF;
    
    for upper in 0..=0xFFFFu64 {
        let world_seed = ((upper << 48) | base_48) as i64;
        if obfuscate_seed(world_seed) == target_hash {
            results.push(world_seed);
        }
    }
    
    results
}

/// Find all world seeds that produce a given hashed seed (full 64-bit search).
/// This is much slower but doesn't require a structure seed.
pub fn find_world_seeds_full(target_hash: i64) -> Vec<i64> {
    let mut results = Vec::new();
    
    // We can parallelize this search
    for upper in 0..=0xFFFFu64 {
        let base = upper << 48;
        for lower in 0..=0xFFFFFFFFFFFFu64 {
            let world_seed = (base | lower) as i64;
            if obfuscate_seed(world_seed) == target_hash {
                results.push(world_seed);
            }
        }
    }
    
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obfuscate_seed() {
        // Test with known seed
        let world_seed = 765906787396911863i64;
        let hashed = obfuscate_seed(world_seed);
        
        // Just verify it's deterministic and non-zero
        assert_ne!(hashed, 0);
        assert_eq!(obfuscate_seed(world_seed), hashed);
    }

    #[test]
    fn test_find_from_hash() {
        let world_seed = 765906787396911863i64;
        let structure_seed = (world_seed as u64) & 0xFFFFFFFFFFFF;
        let hashed = obfuscate_seed(world_seed);
        
        let candidates = find_world_seeds_from_hash(structure_seed, hashed);
        
        assert!(candidates.contains(&world_seed));
        // Should typically find exactly 1 seed, sometimes a few collisions
        assert!(!candidates.is_empty());
    }

    #[test]
    fn test_obfuscate_deterministic() {
        let seeds = [0i64, 1, -1, i64::MIN, i64::MAX, 0x0123_4567_89ab_cdef];
        for seed in seeds {
            let h1 = obfuscate_seed(seed);
            let h2 = obfuscate_seed(seed);
            assert_eq!(h1, h2, "Obfuscation must be deterministic for seed {}", seed);
        }
    }
}
