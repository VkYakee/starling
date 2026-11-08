use std::ffi::{c_int, c_longlong, c_ulonglong};
use std::slice;

use crate::bedrock::block::{BedrockBlock, BlockType};
use crate::bedrock::solver::BedrockSolver;
use crate::biomes::cubiomes::BiomeGenerator;
use crate::math::hashed_seed::HashedSeed;
use crate::pillars::EndPillars;
use crate::structures::lifting::StructureLifter;
use crate::structures::{StructureInstance, StructureType};

#[unsafe(no_mangle)]
pub extern "C" fn seedcracker_init() {
    println!("[SeedCracker FFI] Native Rust engine initialized.");
}

#[unsafe(no_mangle)]
pub extern "C" fn seedcracker_crack_bedrock_list(
    seeds_ptr: *const c_ulonglong,
    seeds_len: usize,
    blocks_x: *const c_int,
    blocks_y: *const c_int,
    blocks_z: *const c_int,
    blocks_type: *const c_int,
    blocks_len: usize,
    out_results: *mut c_ulonglong,
    out_max_capacity: usize,
) -> usize {
    if seeds_ptr.is_null() || blocks_x.is_null() || out_results.is_null() {
        return 0;
    }

    let seeds = unsafe { slice::from_raw_parts(seeds_ptr as *const u64, seeds_len) };
    let xs = unsafe { slice::from_raw_parts(blocks_x, blocks_len) };
    let ys = unsafe { slice::from_raw_parts(blocks_y, blocks_len) };
    let zs = unsafe { slice::from_raw_parts(blocks_z, blocks_len) };
    let types = unsafe { slice::from_raw_parts(blocks_type, blocks_len) };

    let mut blocks = Vec::with_capacity(blocks_len);
    for i in 0..blocks_len {
        let btype = if types[i] == 1 {
            BlockType::Bedrock
        } else {
            BlockType::Other
        };
        blocks.push(BedrockBlock::new(xs[i], ys[i], zs[i], btype));
    }

    let matched = BedrockSolver::crack_from_list(seeds, &blocks);
    let count = matched.len().min(out_max_capacity);
    for i in 0..count {
        unsafe {
            *out_results.add(i) = matched[i];
        }
    }
    count
}

#[unsafe(no_mangle)]
pub extern "C" fn seedcracker_lift_hashed_seed(
    structure_seed: c_ulonglong,
    hashed_seed: c_longlong,
    out_results: *mut c_longlong,
    out_max_capacity: usize,
) -> usize {
    let matched = HashedSeed::find_world_seeds_from_hash(structure_seed, hashed_seed);
    let count = matched.len().min(out_max_capacity);
    for i in 0..count {
        unsafe {
            *out_results.add(i) = matched[i];
        }
    }
    count
}

#[unsafe(no_mangle)]
pub extern "C" fn seedcracker_lift_structures(
    types: *const c_int,
    chunk_xs: *const c_int,
    chunk_zs: *const c_int,
    len: usize,
    out_results: *mut c_ulonglong,
    out_max_capacity: usize,
) -> usize {
    if types.is_null() || chunk_xs.is_null() || chunk_zs.is_null() || out_results.is_null() {
        return 0;
    }

    let types_slice = unsafe { slice::from_raw_parts(types, len) };
    let xs = unsafe { slice::from_raw_parts(chunk_xs, len) };
    let zs = unsafe { slice::from_raw_parts(chunk_zs, len) };

    let mut structures = Vec::with_capacity(len);
    for i in 0..len {
        let st_type = match types_slice[i] {
            0 => StructureType::DesertPyramid,
            1 => StructureType::JungleTemple,
            2 => StructureType::SwampHut,
            3 => StructureType::Igloo,
            4 => StructureType::Shipwreck,
            5 => StructureType::OceanMonument,
            6 => StructureType::PillagerOutpost,
            7 => StructureType::EndCity,
            8 => StructureType::TrialChambers,
            _ => StructureType::AncientCity,
        };
        structures.push(StructureInstance::new(st_type, xs[i], zs[i]));
    }

    let results = StructureLifter::lift_structures(&structures);
    let count = results.len().min(out_max_capacity);
    for i in 0..count {
        unsafe {
            *out_results.add(i) = results[i];
        }
    }
    count
}

#[unsafe(no_mangle)]
pub extern "C" fn seedcracker_find_pillar_seeds(
    heights: *const c_int,
    out_results: *mut c_int,
    out_max_capacity: usize,
) -> usize {
    if heights.is_null() || out_results.is_null() {
        return 0;
    }

    let slice_in = unsafe { slice::from_raw_parts(heights, 10) };
    let mut observed = [None; 10];
    for i in 0..10 {
        if slice_in[i] > 0 {
            observed[i] = Some(slice_in[i]);
        }
    }

    let candidates = EndPillars::find_pillar_seeds(&observed);
    let count = candidates.len().min(out_max_capacity);
    for i in 0..count {
        unsafe {
            *out_results.add(i) = candidates[i] as c_int;
        }
    }
    count
}

#[unsafe(no_mangle)]
pub extern "C" fn seedcracker_get_biome_at(
    mc_version: c_int,
    seed: c_ulonglong,
    x: c_int,
    y: c_int,
    z: c_int,
) -> c_int {
    let mut r#gen = BiomeGenerator::new(mc_version);
    r#gen.set_seed(seed);
    r#gen.get_biome_at_block(x, y, z)
}

#[unsafe(no_mangle)]
pub extern "C" fn seedcracker_locate_structure(
    struct_type: c_int,
    mc_version: c_int,
    seed: c_ulonglong,
    center_x: c_int,
    center_z: c_int,
    out_pos: *mut c_int,
) -> c_int {
    if out_pos.is_null() {
        return 0;
    }
    use crate::structures::finder::{CubiomesStructure, StructureLocator};
    let st = match struct_type {
        1 => CubiomesStructure::DesertPyramid,
        2 => CubiomesStructure::JungleTemple,
        3 => CubiomesStructure::SwampHut,
        4 => CubiomesStructure::Igloo,
        5 => CubiomesStructure::Village,
        7 => CubiomesStructure::Shipwreck,
        8 => CubiomesStructure::Monument,
        9 => CubiomesStructure::Mansion,
        10 => CubiomesStructure::Outpost,
        13 => CubiomesStructure::AncientCity,
        20 => CubiomesStructure::EndCity,
        24 => CubiomesStructure::TrialChambers,
        _ => CubiomesStructure::DesertPyramid,
    };

    let mut r#gen = BiomeGenerator::new(mc_version);
    r#gen.set_seed(seed);

    if let Some((x, z, _)) = StructureLocator::locate_nearest(st, &mut r#gen, seed, center_x, center_z, 10) {
        unsafe {
            *out_pos = x;
            *out_pos.add(1) = z;
        }
        1
    } else {
        0
    }
}
