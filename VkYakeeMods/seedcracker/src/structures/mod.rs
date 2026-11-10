pub mod finder;
pub mod lifting;
pub mod treasure;

pub use finder::{CPos, CubiomesStructure, StructureLocator};
pub use lifting::StructureLifter;
pub use treasure::{TreasureDatabase, TreasureFormation};

use crate::math::lcg::JavaRandom;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructureType {
    DesertPyramid,
    JungleTemple,
    SwampHut,
    Igloo,
    Village,
    Shipwreck,
    OceanMonument,
    PillagerOutpost,
    EndCity,
    TrialChambers,
    AncientCity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StructureConfig {
    pub salt: u64,
    pub spacing: i32,
    pub separation: i32,
}

impl StructureConfig {
    pub const fn new(salt: u64, spacing: i32, separation: i32) -> Self {
        Self {
            salt,
            spacing,
            separation,
        }
    }

    #[inline]
    pub fn offset(&self) -> i32 {
        self.spacing - self.separation
    }
}

pub fn get_structure_config(structure_type: StructureType) -> StructureConfig {
    match structure_type {
        StructureType::DesertPyramid => StructureConfig::new(14357617, 32, 8),
        StructureType::JungleTemple => StructureConfig::new(14357619, 32, 8),
        StructureType::SwampHut => StructureConfig::new(14357620, 32, 8),
        StructureType::Igloo => StructureConfig::new(14357618, 32, 8),
        StructureType::Village => StructureConfig::new(10387312, 34, 8),
        StructureType::Shipwreck => StructureConfig::new(165745295, 24, 4),
        StructureType::OceanMonument => StructureConfig::new(10387313, 32, 5),
        StructureType::PillagerOutpost => StructureConfig::new(165745296, 32, 8),
        StructureType::EndCity => StructureConfig::new(10387313, 20, 11),
        StructureType::TrialChambers => StructureConfig::new(94251327, 34, 12),
        StructureType::AncientCity => StructureConfig::new(20083232, 24, 8),
    }
}

/// Predicts the chunk coordinate (chunk_x, chunk_z) of a structure in a given region for a world seed.
pub fn predict_structure_chunk(seed: u64, structure_type: StructureType, reg_x: i32, reg_z: i32) -> (i32, i32) {
    let config = get_structure_config(structure_type);
    let salt = config.salt;
    let step_seed = (reg_x as i64).wrapping_mul(341873128712)
        .wrapping_add((reg_z as i64).wrapping_mul(132897987541))
        .wrapping_add(seed as i64)
        .wrapping_add(salt as i64) as u64;

    let mut rng = JavaRandom::new(step_seed);
    let range = (config.spacing - config.separation).max(1);
    let ox = rng.next_int_bounded(range).unwrap_or(0);
    let oz = rng.next_int_bounded(range).unwrap_or(0);
    let chunk_x = reg_x * config.spacing + ox;
    let chunk_z = reg_z * config.spacing + oz;
    (chunk_x, chunk_z)
}

/// Finds the closest predicted structure around block coordinates (player_x, player_z)
/// returns: (StructureType, chunk_x, chunk_z, block_x, block_z, distance_blocks)
pub fn find_closest_predicted_structure(
    seed: u64,
    structure_types: &[StructureType],
    player_x: i32,
    player_z: i32,
    max_search_dist_blocks: i32,
) -> Option<(StructureType, i32, i32, i32, i32, f32)> {
    let mut closest = None;
    let mut min_dist_sq = (max_search_dist_blocks as f64) * (max_search_dist_blocks as f64);

    for &st in structure_types {
        let config = get_structure_config(st);
        let center_chunk_x = player_x >> 4;
        let center_chunk_z = player_z >> 4;
        let reg_x = center_chunk_x.div_euclid(config.spacing);
        let reg_z = center_chunk_z.div_euclid(config.spacing);

        // Check 3x3 surrounding regions
        for rz in (reg_z - 1)..=(reg_z + 1) {
            for rx in (reg_x - 1)..=(reg_x + 1) {
                let (cx, cz) = predict_structure_chunk(seed, st, rx, rz);
                let bx = cx * 16 + 8;
                let bz = cz * 16 + 8;
                let dx = (bx - player_x) as f64;
                let dz = (bz - player_z) as f64;
                let dist_sq = dx * dx + dz * dz;
                if dist_sq < min_dist_sq {
                    min_dist_sq = dist_sq;
                    closest = Some((st, cx, cz, bx, bz, dist_sq.sqrt() as f32));
                }
            }
        }
    }

    closest
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StructureInstance {
    pub structure_type: StructureType,
    pub chunk_x: i32,
    pub chunk_z: i32,
}

impl StructureInstance {
    pub fn new(structure_type: StructureType, chunk_x: i32, chunk_z: i32) -> Self {
        Self {
            structure_type,
            chunk_x,
            chunk_z,
        }
    }

    pub fn from_block_coords(structure_type: StructureType, block_x: i32, block_z: i32) -> Self {
        Self::new(structure_type, block_x >> 4, block_z >> 4)
    }

    pub fn region_x(&self) -> i32 {
        let config = get_structure_config(self.structure_type);
        let mut r = self.chunk_x / config.spacing;
        if self.chunk_x < 0 && self.chunk_x % config.spacing != 0 {
            r -= 1;
        }
        r
    }

    pub fn region_z(&self) -> i32 {
        let config = get_structure_config(self.structure_type);
        let mut r = self.chunk_z / config.spacing;
        if self.chunk_z < 0 && self.chunk_z % config.spacing != 0 {
            r -= 1;
        }
        r
    }

    pub fn offset_x(&self) -> i32 {
        let config = get_structure_config(self.structure_type);
        let rx = self.region_x();
        self.chunk_x - rx * config.spacing
    }

    pub fn offset_z(&self) -> i32 {
        let config = get_structure_config(self.structure_type);
        let rz = self.region_z();
        self.chunk_z - rz * config.spacing
    }
}
