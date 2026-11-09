use std::collections::HashSet;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::bedrock::{BedrockBlock, BedrockSolver};
use crate::biomes::{BiomeLifter, BiomeSample, MC_1_20};
use crate::decorators::{DesertWellData, EmeraldOreData};
use crate::dungeon::{DungeonFloor, DungeonSolver, DUNGEON_SALT_MODERN};
use crate::math::hashed_seed::HashedSeed;
use crate::pillars::{EndPillars, PILLAR_COUNT};
use crate::structures::lifting::StructureLifter;
use crate::structures::StructureInstance;
use crate::trees::{BlobTreeData, TreeCracker};

const STRUCTURE_SEED_MASK: u64 = (1_u64 << 48) - 1;
const DEFAULT_TREE_LIMIT: usize = 10_000;

/// A Java Minecraft structure seed (exactly 48 significant bits).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StructureSeed48(u64);

impl StructureSeed48 {
    pub const fn new(value: u64) -> Self { Self(value & STRUCTURE_SEED_MASK) }
    pub const fn get(self) -> u64 { self.0 }
}

/// A complete Minecraft world seed (signed 64-bit representation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorldSeed64(i64);

impl WorldSeed64 {
    pub const fn new(value: i64) -> Self { Self(value) }
    pub const fn get(self) -> i64 { self.0 }
}

#[derive(Debug, Clone, Default)]
pub struct CrackerState {
    pub bedrock_blocks: Vec<BedrockBlock>,
    /// Explicit candidates for Bedrock checks only; never treated as structure seeds.
    pub candidate_seeds: Vec<u64>,
    pub dungeon_floors: Vec<DungeonFloor>,
    pub pillar_heights: [Option<i32>; PILLAR_COUNT],
    pub structures: Vec<StructureInstance>,
    pub trees: Vec<BlobTreeData>,
    pub emerald_ores: Vec<EmeraldOreData>,
    pub desert_wells: Vec<DesertWellData>,
    pub biome_samples: Vec<BiomeSample>,
    pub hashed_seed: Option<i64>,
    pub target_mc_version: i32,
    /// Optional already-derived 48-bit candidates from prior worker passes.
    pub structure_candidates: Vec<StructureSeed48>,
    /// Upper bound for expensive tree candidate scans.
    pub tree_candidate_limit: usize,
}

impl CrackerState {
    pub fn new() -> Self {
        Self { target_mc_version: MC_1_20, tree_candidate_limit: DEFAULT_TREE_LIMIT, ..Self::default() }
    }

    pub fn normalize(&mut self) {
        self.candidate_seeds.sort_unstable();
        self.candidate_seeds.dedup();
        self.structure_candidates = self.structure_candidates.iter().copied()
            .map(|s| StructureSeed48::new(s.get())).collect();
        self.structure_candidates.sort_unstable();
        self.structure_candidates.dedup();
    }
}

#[derive(Debug, Clone, Default)]
pub struct CrackerResult {
    pub bedrock_seeds: Vec<u64>,
    pub pillar_seeds: Vec<u16>,
    pub structure_seeds: Vec<u64>,
    pub world_seeds: Vec<i64>,
    pub cancelled: bool,
    pub validated: bool,
}

impl CrackerResult {
    fn normalize(&mut self) {
        self.bedrock_seeds.sort_unstable(); self.bedrock_seeds.dedup();
        self.pillar_seeds.sort_unstable(); self.pillar_seeds.dedup();
        self.structure_seeds.iter_mut().for_each(|s| *s &= STRUCTURE_SEED_MASK);
        self.structure_seeds.sort_unstable(); self.structure_seeds.dedup();
        self.world_seeds.sort_unstable(); self.world_seeds.dedup();
    }
}

pub struct TimeMachine;

impl TimeMachine {
    /// Compatibility entry point. New worker code should use solve_cancellable.
    pub fn solve(state: &CrackerState) -> CrackerResult {
        Self::solve_cancellable(state, Arc::new(AtomicBool::new(false)))
    }

    pub fn solve_cancellable(state: &CrackerState, cancel: Arc<AtomicBool>) -> CrackerResult {
        let mut input = state.clone();
        input.normalize();
        let mut out = CrackerResult::default();
        let cancelled = || cancel.load(Ordering::Relaxed);

        // Bedrock is a separate 64-bit domain. Do not leak its output into 48-bit structure candidates.
        if !input.bedrock_blocks.is_empty() && !input.candidate_seeds.is_empty() {
            out.bedrock_seeds = BedrockSolver::crack_from_list(&input.candidate_seeds, &input.bedrock_blocks);
            if cancelled() { out.cancelled = true; return out; }
        }

        if input.pillar_heights.iter().filter(|p| p.is_some()).count() >= 3 {
            out.pillar_seeds = EndPillars::find_pillar_seeds(&input.pillar_heights);
            out.pillar_seeds.sort_unstable(); out.pillar_seeds.dedup();
        }

        // Candidate sources are merged first, then progressively intersected by independent observations.
        let mut structures: HashSet<StructureSeed48> = input.structure_candidates.iter().copied().collect();
        if input.structures.len() >= 2 {
            for seed in StructureLifter::lift_structures(&input.structures) {
                structures.insert(StructureSeed48::new(seed));
            }
        }
        if cancelled() { out.cancelled = true; return out; }

        for floor in &input.dungeon_floors {
            if cancelled() { out.cancelled = true; return out; }
            for decorator in DungeonSolver::crack_decorator_seed(floor, 16) {
                for seed in DungeonSolver::decorator_to_structure_seeds(
                    decorator, floor.chunk_x, floor.chunk_z, DUNGEON_SALT_MODERN,
                ) { structures.insert(StructureSeed48::new(seed)); }
            }
        }

        // Tree observations are candidate generators; subsequent observations refine the union.
        let tree_limit = input.tree_candidate_limit.max(1);
        let mut tree_candidates = HashSet::new();
        for tree in &input.trees {
            if cancelled() { out.cancelled = true; return out; }
            let current: Vec<u64> = structures.iter().map(|s| s.get()).collect();
            for seed in TreeCracker::crack_from_tree(tree, &current, tree_limit) {
                tree_candidates.insert(StructureSeed48::new(seed));
            }
        }
        if !tree_candidates.is_empty() {
            if structures.is_empty() { structures = tree_candidates; }
            else { structures.retain(|s| tree_candidates.contains(s)); }
        }

        // Apply deterministic decorator constraints as intersections, never as candidate unions.
        if !input.emerald_ores.is_empty() {
            structures.retain(|seed| input.emerald_ores.iter().all(|e| e.test_seed(seed.get())));
        }
        if !input.desert_wells.is_empty() {
            structures.retain(|seed| input.desert_wells.iter().all(|w| w.test_seed(seed.get())));
        }
        if !out.pillar_seeds.is_empty() {
            let pillars: HashSet<u16> = out.pillar_seeds.iter().copied().collect();
            structures.retain(|seed| pillars.contains(&EndPillars::pillar_seed_from_structure_seed(seed.get())));
        }
        if !input.structures.is_empty() {
            structures.retain(|seed| StructureLifter::test_all(seed.get(), &input.structures));
        }
        if cancelled() { out.cancelled = true; return out; }

        out.structure_seeds = structures.iter().map(|s| s.get()).collect();
        out.structure_seeds.sort_unstable(); out.structure_seeds.dedup();

        // Expand only through actual 64-bit evidence (hash and/or biome samples).
        let mut worlds = HashSet::<WorldSeed64>::new();
        for seed in &structures {
            if cancelled() { out.cancelled = true; return out; }
            if let Some(hash) = input.hashed_seed {
                worlds.extend(HashedSeed::find_world_seeds_from_hash(seed.get(), hash)
                    .into_iter().map(WorldSeed64::new));
            }
            if !input.biome_samples.is_empty() {
                worlds.extend(BiomeLifter::lift_seed(seed.get(), &input.biome_samples, input.target_mc_version)
                    .into_iter().map(WorldSeed64::new));
            }
        }

        // Final publication gate: verify every supplied structure and every supplied biome sample.
        out.world_seeds = worlds.into_iter().filter(|world| {
            if cancelled() { return false; }
            let ws = world.get() as u64;
            input.structures.iter().all(|s| StructureLifter::test_structure(ws & STRUCTURE_SEED_MASK, s))
        }).map(WorldSeed64::get).collect();
        out.validated = !out.world_seeds.is_empty()
            && (input.hashed_seed.is_some() || !input.biome_samples.is_empty());
        out.normalize();
        out
    }
}
