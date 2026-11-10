use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::math::chunk_rand::{get_region_seed, get_population_seed};
use crate::math::lcg::{JavaRandom, MASK48};
use crate::structures::{get_structure_config, StructureInstance, StructureType};

pub struct StructureLifter;
impl StructureLifter {
    pub fn test_structure(seed: u64, structure: &StructureInstance) -> bool {
        let config = get_structure_config(structure.structure_type);
        let region_seed = get_region_seed(seed, structure.region_x(), structure.region_z(), config.salt as i32);
        let mut rand = JavaRandom::new(region_seed);
        if structure.structure_type == StructureType::OceanMonument {
            let offset_x = (rand.next_int_bounded(config.offset()).unwrap_or(0) + rand.next_int_bounded(config.offset()).unwrap_or(0)) / 2;
            let offset_z = (rand.next_int_bounded(config.offset()).unwrap_or(0) + rand.next_int_bounded(config.offset()).unwrap_or(0)) / 2;
            offset_x == structure.offset_x() && offset_z == structure.offset_z()
        } else {
            rand.next_int_bounded(config.offset()) == Some(structure.offset_x())
                && rand.next_int_bounded(config.offset()) == Some(structure.offset_z())
        }
    }
    pub fn test_all(seed: u64, structures: &[StructureInstance]) -> bool {
        structures.iter().all(|s| Self::test_structure(seed, s))
    }

    /// Compatibility exhaustive scan (2^29 upper values per surviving lower-19 candidate).
    pub fn lift_structures(structures: &[StructureInstance]) -> Vec<u64> {
        Self::lift_structures_range(structures, 0, 1u64 << 29, &AtomicBool::new(false))
    }

    /// Resumable upper-bit search. Call repeatedly with consecutive ranges to bound each worker slice.
    pub fn lift_structures_range(
        structures: &[StructureInstance], upper_start: u64, upper_end: u64, cancel: &AtomicBool,
    ) -> Vec<u64> {
        if structures.is_empty() || upper_start >= upper_end || upper_start >= (1u64 << 29) { return Vec::new(); }
        let eligible: Vec<&StructureInstance> = structures.iter().filter(|s| {
            let c = get_structure_config(s.structure_type);
            s.structure_type != StructureType::OceanMonument && c.offset() % 4 == 0
        }).collect();
        if eligible.is_empty() { return Vec::new(); }
        let lower: Vec<u64> = (0..(1u64 << 19)).into_par_iter().filter(|&v| {
            if cancel.load(Ordering::Relaxed) { return false; }
            eligible.iter().all(|s| {
                let c = get_structure_config(s.structure_type);
                let rs = get_region_seed(v, s.region_x(), s.region_z(), c.salt as i32);
                let mut r = JavaRandom::new(rs);
                r.next_int_bounded(c.offset()).map_or(false, |ox| ox % 4 == s.offset_x() % 4)
                    && r.next_int_bounded(c.offset()).map_or(false, |oz| oz % 4 == s.offset_z() % 4)
            })
        }).collect();
        lower.into_par_iter().flat_map_iter(|lo| {
            let mut found = Vec::new();
            for upper in upper_start..upper_end.min(1u64 << 29) {
                if cancel.load(Ordering::Relaxed) { break; }
                let seed = ((upper << 19) | lo) & MASK48;
                if Self::test_all(seed, structures) { found.push(seed); }
            }
            found
        }).collect()
    }
}
