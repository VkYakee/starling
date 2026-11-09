use std::sync::atomic::{AtomicBool, Ordering};
use rayon::prelude::*;

use crate::biomes::{BiomeLifter, BiomeSample};
use crate::pillars::EndPillars;
use crate::structures::{lifting::StructureLifter, StructureInstance};

pub const STRUCTURE_SEARCH_SPACE: u64 = 1_u64 << 32;
pub const DEFAULT_SEARCH_BUDGET: u64 = 1_000_000;

/// Cursor for SeedCrackerX's pillar-seed -> exhaustive partial-world-seed route.
/// Four shards cover the 32-bit partial seed domain without overlap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructureSearchCursor {
    pub pillar_index: usize,
    pub shard: u8,
    pub offset: u64,
}
impl Default for StructureSearchCursor {
    fn default()->Self{Self{pillar_index:0,shard:0,offset:0}}
}
#[derive(Debug, Clone, Default)]
pub struct StructureSearchBatch {
    pub seeds:Vec<u64>,
    pub cursor:StructureSearchCursor,
    pub finished:bool,
    pub tested:u64,
}

/// Exhaustive, resumable search route equivalent to SeedCrackerX pokeStructures.
/// Each invocation tests at most budget partial seeds; callers persist cursor between worker runs.
pub fn search_structure_seeds(
    pillars:&[u16], structures:&[StructureInstance], cursor:StructureSearchCursor,
    budget:u64, cancel:&AtomicBool,
)->StructureSearchBatch{
    let mut out=StructureSearchBatch{cursor,..Default::default()};
    if pillars.is_empty()||structures.is_empty()||budget==0{return out;}
    let mut remaining=budget;
    while out.cursor.pillar_index<pillars.len() && remaining>0 && !cancel.load(Ordering::Relaxed){
        let pillar=pillars[out.cursor.pillar_index];
        let shard_start=(out.cursor.shard as u64)*(1_u64<<30);
        let shard_end=shard_start+(1_u64<<30);
        let start=shard_start+out.cursor.offset;
        let count=remaining.min(shard_end.saturating_sub(start));
        if count==0{
            out.cursor.shard+=1;out.cursor.offset=0;
            if out.cursor.shard>=4{out.cursor.shard=0;out.cursor.pillar_index+=1;}
            continue;
        }
        let found:Vec<u64>=(start..start+count).into_par_iter().filter_map(|partial|{
            if cancel.load(Ordering::Relaxed){return None;}
            let seed=EndPillars::time_machine_combine(partial,pillar);
            StructureLifter::test_all(seed,structures).then_some(seed)
        }).collect();
        out.seeds.extend(found);
        out.tested+=count;remaining-=count;out.cursor.offset+=count;
        if out.cursor.offset>=(1_u64<<30){
            out.cursor.offset=0;out.cursor.shard+=1;
            if out.cursor.shard>=4{out.cursor.shard=0;out.cursor.pillar_index+=1;}
        }
    }
    out.finished=out.cursor.pillar_index>=pillars.len();
    out.seeds.sort_unstable();out.seeds.dedup();out
}

/// Exact upper-16 biome lifting route for a known 48-bit structure seed.
pub fn deep_biome_world_search(seed:u64,samples:&[BiomeSample],version:i32,cancel:&AtomicBool)->Vec<i64>{
    if cancel.load(Ordering::Relaxed){return Vec::new();}
    BiomeLifter::lift_seed(seed,samples,version)
}

/// Validate and deduplicate a world-seed set against every supplied structure and biome.
pub fn validate_world_seeds(
    candidates:impl IntoIterator<Item=i64>,structures:&[StructureInstance],
    samples:&[BiomeSample],version:i32,cancel:&AtomicBool,
)->Vec<i64>{
    let mut candidates:Vec<i64>=candidates.into_iter().collect();
    candidates.sort_unstable();candidates.dedup();
    candidates.into_par_iter().filter(|seed|{
        if cancel.load(Ordering::Relaxed){return false;}
        let low=(*seed as u64)&((1_u64<<48)-1);
        StructureLifter::test_all(low,structures)
            && (samples.is_empty()||!BiomeLifter::lift_seed(low,samples,version).is_empty())
    }).collect()
}
