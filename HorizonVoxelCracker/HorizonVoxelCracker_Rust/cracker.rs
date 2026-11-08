// Copyright (c) 2026 VkYakee
// VkYakee VoxelMap x seedcrackerX
//! Mathematical seed-cracking engine matching SeedcrackerX algorithms with MC version adaptation.
//! Handles Pre-1.18 vs 1.18+ vs 1.21+ differences and per-world isolated storage.

use super::storage;
use serde::{Deserialize, Serialize};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU32, Ordering},
};

// ---------------------------------------------------------------- Version Adaptation

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MCVersion {
    Auto,
    Pre1_18,     // 1.14 - 1.17 (Legacy biome layers, 256 height limit)
    V1_18_Plus,   // 1.18 - 1.20 (3D Multi-noise biomes, -64 to 320 height limit)
    V1_21_Plus,   // 1.21+ (Trial Chambers & modern worldgen)
}

impl MCVersion {
    pub fn name(self) -> &'static str {
        match self {
            Self::Auto => "Auto Detect",
            Self::Pre1_18 => "Pre-1.18 (1.14-1.17)",
            Self::V1_18_Plus => "1.18+ (Caves & Cliffs)",
            Self::V1_21_Plus => "1.21+ (Modern)",
        }
    }

    pub fn cycle(self) -> Self {
        match self {
            Self::Auto => Self::V1_18_Plus,
            Self::V1_18_Plus => Self::Pre1_18,
            Self::Pre1_18 => Self::V1_21_Plus,
            Self::V1_21_Plus => Self::Auto,
        }
    }

    /// Automatically infer version from loaded chunk height bounds or blocks
    pub fn infer_from_environment(min_y: i32, has_modern_blocks: bool) -> Self {
        if min_y < 0 || has_modern_blocks {
            Self::V1_18_Plus
        } else {
            Self::Pre1_18
        }
    }
}

// ---------------------------------------------------------------- LCG Constants

pub const LCG_MULTIPLIER: u64 = 0x5_DEEC_E66D;
pub const LCG_ADDEND: u64 = 0xB;
pub const LCG_MASK: u64 = (1 << 48) - 1;

/// Java LCG inverse constants: X_{n-1} = (X_n * INV_MULT + INV_ADD) & MASK
pub const LCG_INV_MULTIPLIER: u64 = 0xDFE0_5BCB_1365;
pub const LCG_INV_ADDEND: u64 = 0x615C_0E46_2AA9;

pub const REGION_K1: i64 = 341873128712;
pub const REGION_K2: i64 = 132897987541;

#[derive(Clone, Copy, Debug)]
pub struct JavaRandom {
    pub seed: u64,
}

impl JavaRandom {
    #[inline]
    pub fn new(seed: i64) -> Self {
        Self {
            seed: (seed as u64 ^ LCG_MULTIPLIER) & LCG_MASK,
        }
    }

    #[inline]
    pub const fn from_raw(state: u64) -> Self {
        Self {
            seed: state & LCG_MASK,
        }
    }

    #[inline]
    pub fn next(&mut self, bits: u32) -> i32 {
        self.seed = self.seed.wrapping_mul(LCG_MULTIPLIER).wrapping_add(LCG_ADDEND) & LCG_MASK;
        (self.seed >> (48 - bits)) as i32
    }

    #[inline]
    pub fn next_int(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            return 0;
        }
        if (bound & -bound) == bound {
            return ((bound as i64 * self.next(31) as i64) >> 31) as i32;
        }
        let mut bits = self.next(31);
        let mut val = bits % bound;
        while bits - val + (bound - 1) < 0 {
            bits = self.next(31);
            val = bits % bound;
        }
        val
    }

    #[inline]
    pub fn next_long(&mut self) -> i64 {
        let hi = (self.next(32) as i64) << 32;
        let lo = self.next(32) as i64;
        hi.wrapping_add(lo)
    }

    #[inline]
    pub fn step_back(&mut self) {
        self.seed = self.seed.wrapping_mul(LCG_INV_MULTIPLIER).wrapping_add(LCG_INV_ADDEND) & LCG_MASK;
    }
}

// ---------------------------------------------------------------- Region Seeds

pub fn get_region_seed(world_seed: i64, reg_x: i32, reg_z: i32, salt: i32) -> i64 {
    (reg_x as i64)
        .wrapping_mul(REGION_K1)
        .wrapping_add((reg_z as i64).wrapping_mul(REGION_K2))
        .wrapping_add(world_seed)
        .wrapping_add(salt as i64)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructureType {
    DesertPyramid,
    JungleTemple,
    SwampHut,
    Igloo,
    Shipwreck,
    PillagerOutpost,
    OceanMonument,
    BuriedTreasure,
    EndPillars,
    Dungeon,
}

impl StructureType {
    pub fn name(self) -> &'static str {
        match self {
            Self::DesertPyramid => "Desert Pyramid",
            Self::JungleTemple => "Jungle Temple",
            Self::SwampHut => "Swamp Hut",
            Self::Igloo => "Igloo",
            Self::Shipwreck => "Shipwreck",
            Self::PillagerOutpost => "Pillager Outpost",
            Self::OceanMonument => "Ocean Monument",
            Self::BuriedTreasure => "Buried Treasure",
            Self::EndPillars => "End Pillars",
            Self::Dungeon => "Dungeon",
        }
    }

    pub fn spacing_and_separation(self) -> (i32, i32, i32) {
        match self {
            Self::DesertPyramid => (32, 8, 14357617),
            Self::JungleTemple => (32, 8, 14357619),
            Self::SwampHut => (32, 8, 14357620),
            Self::Igloo => (32, 8, 14357618),
            Self::Shipwreck => (24, 4, 165745295),
            Self::PillagerOutpost => (32, 8, 165745296),
            Self::OceanMonument => (32, 5, 10387313),
            Self::BuriedTreasure => (1, 0, 0),
            Self::EndPillars => (0, 0, 0),
            Self::Dungeon => (0, 0, 0),
        }
    }

    pub fn bits(self) -> f32 {
        match self {
            Self::DesertPyramid | Self::JungleTemple | Self::SwampHut | Self::Igloo | Self::PillagerOutpost => 9.17,
            Self::Shipwreck => 8.64,
            Self::OceanMonument => 9.51,
            Self::BuriedTreasure => 6.64,
            Self::EndPillars => 30.0,
            Self::Dungeon => 15.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CollectedFeature {
    pub kind: StructureType,
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub block_y: i32,
    pub offset_x: i32,
    pub offset_z: i32,
    pub reg_x: i32,
    pub reg_z: i32,
    pub reg_const: i64,
}

impl CollectedFeature {
    pub fn new(kind: StructureType, chunk_x: i32, chunk_z: i32, block_y: i32) -> Self {
        let (spacing, _, salt) = kind.spacing_and_separation();
        let (reg_x, reg_z, off_x, off_z) = if spacing > 0 {
            let rx = if chunk_x < 0 { (chunk_x - spacing + 1) / spacing } else { chunk_x / spacing };
            let rz = if chunk_z < 0 { (chunk_z - spacing + 1) / spacing } else { chunk_z / spacing };
            (rx, rz, chunk_x - rx * spacing, chunk_z - rz * spacing)
        } else {
            (0, 0, 0, 0)
        };
        let reg_const = (reg_x as i64)
            .wrapping_mul(REGION_K1)
            .wrapping_add((reg_z as i64).wrapping_mul(REGION_K2))
            .wrapping_add(salt as i64);

        Self {
            kind,
            chunk_x,
            chunk_z,
            block_y,
            offset_x: off_x,
            offset_z: off_z,
            reg_x,
            reg_z,
            reg_const,
        }
    }

    #[inline]
    pub fn test_structure_seed(&self, seed: u64) -> bool {
        let (spacing, separation, _) = self.kind.spacing_and_separation();
        if spacing <= 0 {
            return true;
        }
        let bound = spacing - separation;
        let reg_seed = (seed as i64).wrapping_add(self.reg_const);
        let mut rand = JavaRandom::new(reg_seed);
        let ox = rand.next_int(bound);
        if ox != self.offset_x {
            return false;
        }
        let oz = rand.next_int(bound);
        oz == self.offset_z
    }
}

// ---------------------------------------------------------------- SHA-256 for Hashed Seed

pub fn sha256_long(val: i64) -> i64 {
    let bytes = val.to_be_bytes();
    let mut h = [
        0x6a09e667u32, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let k = [
        0x428a2f98u32, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    let mut block = [0u8; 64];
    block[..8].copy_from_slice(&bytes);
    block[8] = 0x80;
    block[63] = 64;

    let mut w = [0u32; 64];
    for i in 0..16 {
        w[i] = u32::from_be_bytes([
            block[i * 4],
            block[i * 4 + 1],
            block[i * 4 + 2],
            block[i * 4 + 3],
        ]);
    }
    for i in 16..64 {
        let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
        let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
        w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
    }

    let mut a = h[0];
    let mut b = h[1];
    let mut c = h[2];
    let mut d = h[3];
    let mut e = h[4];
    let mut f = h[5];
    let mut g = h[6];
    let mut h_v = h[7];

    for i in 0..64 {
        let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let ch = (e & f) ^ ((!e) & g);
        let temp1 = h_v.wrapping_add(s1).wrapping_add(ch).wrapping_add(k[i]).wrapping_add(w[i]);
        let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let temp2 = s0.wrapping_add(maj);

        h_v = g;
        g = f;
        f = e;
        e = d.wrapping_add(temp1);
        d = c;
        c = b;
        b = a;
        a = temp1.wrapping_add(temp2);
    }

    h[0] = h[0].wrapping_add(a);
    h[1] = h[1].wrapping_add(b);

    let high = (h[0] as i64) << 32;
    let low = h[1] as i64 & 0xFFFF_FFFF;
    high | low
}

// ---------------------------------------------------------------- Cracking Engine State

#[derive(Clone, Debug, PartialEq)]
pub enum CrackState {
    Idle,
    CollectingData { bits: f32, count: usize },
    Searching { progress_percent: f32, phase: &'static str },
    Success { structure_seed: i64, world_seed: Option<i64> },
    Failed { reason: String },
}

pub struct CrackerEngine {
    pub current_world_id: Mutex<String>,
    pub version: Mutex<MCVersion>,
    pub features: Mutex<Vec<CollectedFeature>>,
    pub hashed_seed: Mutex<Option<i64>>,
    pub state: Mutex<CrackState>,
    pub candidate_structure_seeds: Mutex<Vec<i64>>,
    pub candidate_world_seeds: Mutex<Vec<i64>>,
    pub stop_flag: Arc<AtomicBool>,
    pub progress: Arc<AtomicU32>,
}

impl CrackerEngine {
    pub fn new() -> Self {
        Self {
            current_world_id: Mutex::new("default_world".into()),
            version: Mutex::new(MCVersion::Auto),
            features: Mutex::new(Vec::new()),
            hashed_seed: Mutex::new(None),
            state: Mutex::new(CrackState::Idle),
            candidate_structure_seeds: Mutex::new(Vec::new()),
            candidate_world_seeds: Mutex::new(Vec::new()),
            stop_flag: Arc::new(AtomicBool::new(false)),
            progress: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Load or switch to another world without mixing data
    pub fn switch_world(&self, world_id: &str) {
        let mut cur = self.current_world_id.lock().unwrap();
        if *cur == world_id {
            return;
        }
        *cur = world_id.to_string();

        let (ver, stored_features, s_seed, w_seed) = storage::load_world_cracker_data(world_id);
        *self.version.lock().unwrap() = ver;
        *self.features.lock().unwrap() = stored_features.clone();
        *self.candidate_structure_seeds.lock().unwrap() = s_seed.map(|s| vec![s]).unwrap_or_default();
        *self.candidate_world_seeds.lock().unwrap() = w_seed.map(|w| vec![w]).unwrap_or_default();

        if let Some(ss) = s_seed {
            *self.state.lock().unwrap() = CrackState::Success {
                structure_seed: ss,
                world_seed: w_seed,
            };
        } else if !stored_features.is_empty() {
            let total_bits: f32 = stored_features.iter().map(|f| f.kind.bits()).sum();
            *self.state.lock().unwrap() = CrackState::CollectingData {
                bits: total_bits,
                count: stored_features.len(),
            };
        } else {
            *self.state.lock().unwrap() = CrackState::Idle;
        }
    }

    pub fn set_hashed_seed(&self, seed: i64) {
        *self.hashed_seed.lock().unwrap() = Some(seed);
    }

    /// Clears transient memory without deleting files on disk
    pub fn reset_memory(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        self.features.lock().unwrap().clear();
        self.candidate_structure_seeds.lock().unwrap().clear();
        self.candidate_world_seeds.lock().unwrap().clear();
        *self.hashed_seed.lock().unwrap() = None;
        *self.state.lock().unwrap() = CrackState::Idle;
        self.progress.store(0, Ordering::SeqCst);
    }

    /// User explicitly requested deleting cracker data for the current world (/hvc reset or Del button)
    pub fn delete_world_data(&self) {
        self.reset_memory();
        let wid = self.current_world_id.lock().unwrap().clone();
        let ver = *self.version.lock().unwrap();
        storage::save_world_cracker_data(&wid, ver, &[], None, None);
    }

    pub fn add_feature(&self, feature: CollectedFeature) -> bool {
        let mut list = self.features.lock().unwrap();
        if list.iter().any(|f| f.chunk_x == feature.chunk_x && f.chunk_z == feature.chunk_z && f.kind == feature.kind) {
            return false;
        }
        list.push(feature);
        let total_bits: f32 = list.iter().map(|f| f.kind.bits()).sum();
        let count = list.len();
        *self.state.lock().unwrap() = CrackState::CollectingData { bits: total_bits, count };

        let wid = self.current_world_id.lock().unwrap().clone();
        let ver = *self.version.lock().unwrap();
        storage::save_world_cracker_data(&wid, ver, &*list, None, None);
        true
    }

    pub fn total_bits(&self) -> f32 {
        let list = self.features.lock().unwrap();
        list.iter().map(|f| f.kind.bits()).sum()
    }

    pub fn start_cracking(self: &Arc<Self>) -> bool {
        let bits = self.total_bits();
        if bits < 30.0 && self.features.lock().unwrap().len() < 3 {
            return false;
        }

        self.stop_flag.store(false, Ordering::SeqCst);
        self.progress.store(0, Ordering::SeqCst);
        *self.state.lock().unwrap() = CrackState::Searching {
            progress_percent: 0.0,
            phase: "Lifting lower 19 bits",
        };

        let engine = Arc::clone(self);
        std::thread::spawn(move || {
            engine.run_cracking_job();
        });
        true
    }

    fn run_cracking_job(&self) {
        let features = self.features.lock().unwrap().clone();
        let hashed = *self.hashed_seed.lock().unwrap();

        // Phase 1: 19-bit lower bits modular reduction
        // Mathematically, rand.nextInt(bound) % 4 == offset % 4 is only invariant
        // across higher seed bits when bound is divisible by 4 (spacing - separation % 4 == 0).
        let eligible: Vec<&CollectedFeature> = features
            .iter()
            .filter(|f| {
                let (spacing, separation, _) = f.kind.spacing_and_separation();
                spacing > 0 && (spacing - separation) % 4 == 0
            })
            .collect();

        let mut valid_lower = Vec::new();
        let total_lower = 1u64 << 19;
        let report_step = total_lower / 16;

        for lower in 0..total_lower {
            if (lower % report_step) == 0 {
                if self.stop_flag.load(Ordering::Relaxed) {
                    return;
                }
                let pct = (lower as f32 / total_lower as f32) * 35.0;
                *self.state.lock().unwrap() = CrackState::Searching {
                    progress_percent: pct,
                    phase: "Lifting lower 19 bits",
                };
                self.progress.store(pct as u32, Ordering::Relaxed);
            }

            let mut ok = true;
            for f in &eligible {
                let (spacing, separation, _) = f.kind.spacing_and_separation();
                let bound = spacing - separation;
                let reg_seed = (lower as i64).wrapping_add(f.reg_const);
                let mut rand = JavaRandom::new(reg_seed);
                if rand.next_int(bound) % 4 != f.offset_x % 4 || rand.next_int(bound) % 4 != f.offset_z % 4 {
                    ok = false;
                    break;
                }
            }
            if ok {
                valid_lower.push(lower);
            }
        }

        if valid_lower.is_empty() {
            *self.state.lock().unwrap() = CrackState::Failed {
                reason: "Contradictory lower bits. Verify structures are from this world.".into(),
            };
            return;
        }

        *self.state.lock().unwrap() = CrackState::Searching {
            progress_percent: 35.0,
            phase: "Verifying 48-bit structures",
        };
        self.progress.store(35, Ordering::Relaxed);

        // Phase 2: Full 48-bit extension
        let mut structure_seeds = Vec::new();
        let upper_max = 1u64 << (48 - 19);
        let total_valid = valid_lower.len();

        for (idx, &lower) in valid_lower.iter().enumerate() {
            if self.stop_flag.load(Ordering::Relaxed) {
                return;
            }
            let pct = 35.0 + (idx as f32 / total_valid as f32) * 40.0;
            *self.state.lock().unwrap() = CrackState::Searching {
                progress_percent: pct,
                phase: "Verifying 48-bit structures",
            };
            self.progress.store(pct as u32, Ordering::Relaxed);

            for upper in 0..upper_max {
                if (upper & 0x7F_FFFF) == 0 && self.stop_flag.load(Ordering::Relaxed) {
                    return;
                }
                let seed = (upper << 19) | lower;
                let mut match_all = true;
                for f in &features {
                    if !f.test_structure_seed(seed) {
                        match_all = false;
                        break;
                    }
                }
                if match_all {
                    structure_seeds.push(seed as i64);
                    if structure_seeds.len() >= 100 {
                        break;
                    }
                }
            }
            if structure_seeds.len() >= 100 {
                break;
            }
        }

        *self.candidate_structure_seeds.lock().unwrap() = structure_seeds.clone();

        if structure_seeds.is_empty() {
            *self.state.lock().unwrap() = CrackState::Failed {
                reason: "No matching seed found for features. Need more structures.".into(),
            };
            return;
        }

        // Phase 3: 64-bit World Seed lifting with Hashed Seed
        let mut world_seeds = Vec::new();
        if let Some(target_hash) = hashed {
            *self.state.lock().unwrap() = CrackState::Searching {
                progress_percent: 75.0,
                phase: "Lifting to 64-bit World Seed",
            };
            self.progress.store(75, Ordering::Relaxed);

            for &struct_seed in &structure_seeds {
                for upper in 0..(1i64 << 16) {
                    let full_seed = (upper << 48) | (struct_seed & 0x0000_FFFF_FFFF_FFFF);
                    if sha256_long(full_seed) == target_hash {
                        world_seeds.push(full_seed);
                    }
                }
            }
        }

        let best_struct = structure_seeds[0];
        let best_world = world_seeds.first().copied();

        *self.candidate_world_seeds.lock().unwrap() = world_seeds;
        *self.state.lock().unwrap() = CrackState::Success {
            structure_seed: best_struct,
            world_seed: best_world,
        };
        self.progress.store(100, Ordering::Relaxed);

        let wid = self.current_world_id.lock().unwrap().clone();
        let ver = *self.version.lock().unwrap();
        storage::save_world_cracker_data(&wid, ver, &features, Some(best_struct), best_world);
    }
}
