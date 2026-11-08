use rayon::prelude::*;

use crate::math::chunk_rand::ChunkRand;
use crate::math::lcg::{JavaRandom, MASK48};
use crate::structures::{get_structure_config, StructureInstance, StructureType};

pub struct StructureLifter;

impl StructureLifter {
    /// Tests if a 48-bit structure seed generates the exact structure coordinates.
    pub fn test_structure(seed: u64, structure: &StructureInstance) -> bool {
        let config = get_structure_config(structure.structure_type);
        let rx = structure.region_x();
        let rz = structure.region_z();
        let ox = structure.offset_x();
        let oz = structure.offset_z();

        let region_seed = ChunkRand::get_region_seed(seed, rx, rz, config.salt);
        let mut rand = JavaRandom::new(region_seed);

        if structure.structure_type == StructureType::OceanMonument {
            let gen_x = (rand.next_int_bounded(config.offset())
                + rand.next_int_bounded(config.offset()))
                / 2;
            let gen_z = (rand.next_int_bounded(config.offset())
                + rand.next_int_bounded(config.offset()))
                / 2;
            gen_x == ox && gen_z == oz
        } else {
            let gen_x = rand.next_int_bounded(config.offset());
            let gen_z = rand.next_int_bounded(config.offset());
            gen_x == ox && gen_z == oz
        }
    }

    /// Verifies all structures against candidate seed.
    pub fn test_all(seed: u64, structures: &[StructureInstance]) -> bool {
        for s in structures {
            if !Self::test_structure(seed, s) {
                return false;
            }
        }
        true
    }

    /// Solves for 48-bit structure seeds from multiple structure instances.
    /// Uses 19-bit congruence reduction to prune >99.9% of candidate space before full test.
    pub fn lift_structures(structures: &[StructureInstance]) -> Vec<u64> {
        if structures.is_empty() {
            return Vec::new();
        }

        // Filter structures suitable for lower-19 congruence (offset multiple of 4)
        let congruence_structures: Vec<&StructureInstance> = structures
            .iter()
            .filter(|s| {
                let conf = get_structure_config(s.structure_type);
                s.structure_type != StructureType::OceanMonument && conf.offset() % 4 == 0
            })
            .collect();

        // Stage 1: Filter lower 19 bits (0..524,288)
        let matching_lower19: Vec<u64> = (0..(1u64 << 19))
            .into_par_iter()
            .filter(|&lower19| {
                for s in &congruence_structures {
                    let conf = get_structure_config(s.structure_type);
                    let reg_seed = ChunkRand::get_region_seed(lower19, s.region_x(), s.region_z(), conf.salt);
                    let mut rand = JavaRandom::new(reg_seed);
                    let ox = rand.next_int_bounded(conf.offset()) % 4;
                    let oz = rand.next_int_bounded(conf.offset()) % 4;
                    if ox != s.offset_x() % 4 || oz != s.offset_z() % 4 {
                        return false;
                    }
                }
                true
            })
            .collect();

        // Stage 2: Lift matching lower bits to full 48 bits (2^(48-19) = 2^29 upper bits)
        // If we have 2+ structures, matching_lower19 is very small (often < 100 entries).
        let total_upper = 1u64 << (48 - 19);

        matching_lower19
            .into_par_iter()
            .flat_map(|lower19| {
                let mut matched = Vec::new();
                for upper in 0..total_upper {
                    let candidate = ((upper << 19) | lower19) & MASK48;
                    if Self::test_all(candidate, structures) {
                        matched.push(candidate);
                    }
                }
                matched
            })
            .collect()
    }
}
