use std::os::raw::c_int;
use crate::biomes::cubiomes::{BiomeGenerator, CGenerator};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CPos {
    pub x: c_int,
    pub z: c_int,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubiomesStructure {
    Feature = 0,
    DesertPyramid = 1,
    JungleTemple = 2,
    SwampHut = 3,
    Igloo = 4,
    Village = 5,
    OceanRuin = 6,
    Shipwreck = 7,
    Monument = 8,
    Mansion = 9,
    Outpost = 10,
    RuinedPortal = 11,
    RuinedPortalNether = 12,
    AncientCity = 13,
    Treasure = 14,
    Mineshaft = 15,
    DesertWell = 16,
    Geode = 17,
    Fortress = 18,
    Bastion = 19,
    EndCity = 20,
    EndGateway = 21,
    EndIsland = 22,
    TrailRuins = 23,
    TrialChambers = 24,
}

unsafe extern "C" {
    fn getStructurePos(
        structure_type: c_int,
        mc: c_int,
        seed: u64,
        reg_x: c_int,
        reg_z: c_int,
        pos: *mut CPos,
    ) -> c_int;

    fn isViableStructurePos(
        struct_type: c_int,
        g: *mut CGenerator,
        block_x: c_int,
        block_z: c_int,
        flags: u32,
    ) -> c_int;

    fn getSpawn(g: *const CGenerator) -> CPos;
}

pub struct StructureLocator;

impl StructureLocator {
    /// Computes candidate structure position in a region for a given structure seed.
    pub fn get_structure_pos(
        structure: CubiomesStructure,
        mc_version: i32,
        seed: u64,
        reg_x: i32,
        reg_z: i32,
    ) -> Option<(i32, i32)> {
        let mut pos = CPos { x: 0, z: 0 };
        let ok = unsafe {
            getStructurePos(
                structure as c_int,
                mc_version,
                seed,
                reg_x,
                reg_z,
                &mut pos as *mut CPos,
            )
        };
        if ok != 0 {
            Some((pos.x, pos.z))
        } else {
            None
        }
    }

    /// Verifies if a structure at block coordinates is biome-viable according to world generation.
    pub fn is_viable(
        structure: CubiomesStructure,
        generator: &mut BiomeGenerator,
        block_x: i32,
        block_z: i32,
    ) -> bool {
        let raw_ptr = generator.as_raw_mut();
        let ok = unsafe {
            isViableStructurePos(structure as c_int, raw_ptr, block_x, block_z, 0)
        };
        ok != 0
    }

    /// Locates the nearest structure to a center block position within a search radius (in regions).
    pub fn locate_nearest(
        structure: CubiomesStructure,
        generator: &mut BiomeGenerator,
        seed: u64,
        center_x: i32,
        center_z: i32,
        max_region_radius: i32,
    ) -> Option<(i32, i32, f64)> {
        let center_reg_x = center_x >> 9; // approximate 512 block region
        let center_reg_z = center_z >> 9;

        let mut closest: Option<(i32, i32, f64)> = None;

        for r in 0..=max_region_radius {
            for dx in -r..=r {
                for dz in -r..=r {
                    if dx.abs() != r && dz.abs() != r {
                        continue; // Only check perimeter of current spiral ring
                    }
                    let rx = center_reg_x + dx;
                    let rz = center_reg_z + dz;

                    if let Some((bx, bz)) = Self::get_structure_pos(structure, generator.version, seed, rx, rz) {
                        if Self::is_viable(structure, generator, bx, bz) {
                            let dist_sq = ((bx - center_x) as f64).powi(2) + ((bz - center_z) as f64).powi(2);
                            let dist = dist_sq.sqrt();
                            if closest.as_ref().map_or(true, |c| dist < c.2) {
                                closest = Some((bx, bz, dist));
                            }
                        }
                    }
                }
            }
            if closest.is_some() && r >= 2 {
                break; // Found within inner ring
            }
        }

        closest
    }

    /// Computes the spawn point of the world.
    pub fn get_spawn(generator: &BiomeGenerator) -> (i32, i32) {
        let raw_ptr = generator.as_raw();
        let pos = unsafe { getSpawn(raw_ptr) };
        (pos.x, pos.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biomes::MC_1_20;

    #[test]
    fn test_structure_pos_generation() {
        let seed = 765906787396911863u64;
        let pos = StructureLocator::get_structure_pos(
            CubiomesStructure::DesertPyramid,
            MC_1_20,
            seed,
            0,
            0,
        );
        assert!(pos.is_some());
        let (x, z) = pos.unwrap();
        println!("Desert Pyramid pos at reg (0, 0): ({}, {})", x, z);
    }
}
