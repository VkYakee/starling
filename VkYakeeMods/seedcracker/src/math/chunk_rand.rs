//! Region seed computation matching Horizon's WorldgenRandom exactly.
//! This is used for structure placement and population seed generation.

use horizon_core::random::LegacyRandom;

/// Compute the region seed exactly as Horizon's WorldgenRandom::set_large_feature_with_salt does.
/// This matches Java's ChunkGenerator::getRegionSeed.
/// 
/// Instead of going through WorldgenRandom, we compute the seed directly:
/// WorldgenRandom::set_large_feature_with_salt sets the seed to:
/// (x * 341873128712) + (z * 132897987541) + seed + salt
/// Then LegacyRandom's state() returns the 48-bit LCG state.
pub fn get_region_seed(base_seed: u64, region_x: i32, region_z: i32, salt: i32) -> u64 {
    // This matches WorldgenRandom::set_large_feature_with_salt implementation
    let seed = (region_x as i64)
        .wrapping_mul(341873128712)
        .wrapping_add((region_z as i64).wrapping_mul(132897987541))
        .wrapping_add(base_seed as i64)
        .wrapping_add(salt as i64);
    
    // Create LegacyRandom with that seed and get its 48-bit state
    let rand = LegacyRandom::new(seed);
    rand.state()
}

/// Get the population seed for a chunk given a structure seed.
/// This matches WorldgenRandom::get_population_seed (salt = 0).
pub fn get_population_seed(structure_seed: u64, chunk_x: i32, chunk_z: i32) -> u64 {
    // Same formula with salt = 0
    let seed = (chunk_x as i64)
        .wrapping_mul(341873128712)
        .wrapping_add((chunk_z as i64).wrapping_mul(132897987541))
        .wrapping_add(structure_seed as i64);
    
    let rand = LegacyRandom::new(seed);
    rand.state()
}

/// Compute the structure start position within a region for a given structure seed.
/// Returns the chunk coordinates within the region (0..spacing-1).
pub fn get_structure_chunk_pos(
    structure_seed: u64,
    region_x: i32,
    region_z: i32,
    spacing: i32,
    separation: i32,
    salt: i32,
) -> Option<(i32, i32)> {
    let region_seed = get_region_seed(structure_seed, region_x, region_z, salt);
    let mut rand = LegacyRandom::new(region_seed as i64);
    
    let limit = spacing - separation;
    if limit <= 0 {
        return None;
    }
    
    let offset_x = rand.next_int_bound(limit).ok()?;
    let offset_z = rand.next_int_bound(limit).ok()?;
    
    Some((offset_x, offset_z))
}

/// Test if a structure would generate at specific chunk coordinates.
/// This is a direct parity check with Horizon's StructureRegistry::is_structure_chunk
/// for RandomSpread placement.
pub fn test_structure_at_chunk(
    structure_seed: u64,
    chunk_x: i32,
    chunk_z: i32,
    spacing: i32,
    separation: i32,
    salt: i32,
) -> bool {
    let region_x = chunk_x.div_euclid(spacing);
    let region_z = chunk_z.div_euclid(spacing);
    
    if let Some((offset_x, offset_z)) = get_structure_chunk_pos(structure_seed, region_x, region_z, spacing, separation, salt) {
        region_x * spacing + offset_x == chunk_x && region_z * spacing + offset_z == chunk_z
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_core::random::{LegacyRandom, WorldgenRandom};

    #[test]
    fn test_region_seed_matches_horizon() {
        let seed = 765906787396911863u64 & 0xFFFFFFFFFFFF;
        let region_x = 0;
        let region_z = 0;
        let salt = 10387312; // villages salt

        let ours = get_region_seed(seed, region_x, region_z, salt);
        
        let mut horizon_random = WorldgenRandom::new(LegacyRandom::new(0));
        horizon_random.set_large_feature_with_salt(seed as i64, region_x, region_z, salt);
        let horizon_seed = horizon_random.state().state();
        
        assert_eq!(ours, horizon_seed, "Region seed must match Horizon exactly");
    }

    #[test]
    fn test_population_seed() {
        let structure_seed = 0x123456789ABCu64 & 0xFFFFFFFFFFFF;
        let chunk_x = 100;
        let chunk_z = -50;
        
        let pop1 = get_population_seed(structure_seed, chunk_x, chunk_z);
        let pop2 = get_population_seed(structure_seed, chunk_x, chunk_z);
        
        assert_eq!(pop1, pop2, "Population seed must be deterministic");
    }
}
