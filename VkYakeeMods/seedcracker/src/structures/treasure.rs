//! Buried treasure cluster precomputed table search.
//! Allows cracking 48-bit structure seeds instantly when clusters of buried treasures
//! are found around spawn.

#[derive(Debug, Clone)]
pub struct TreasureFormation {
    pub positions: Vec<(i32, i32)>,
    pub seeds: Vec<u64>,
}

impl TreasureFormation {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }

        let mut offset = 0;
        let count = u32::from_be_bytes(data[offset..offset + 4].try_into().ok()?) as usize;
        offset += 4;

        let mut positions = Vec::with_capacity(count);
        for _ in 0..count {
            if offset + 8 > data.len() {
                return None;
            }
            let cx = i32::from_be_bytes(data[offset..offset + 4].try_into().ok()?);
            let cz = i32::from_be_bytes(data[offset + 4..offset + 8].try_into().ok()?);
            offset += 8;
            positions.push((cx, cz));
        }

        if offset + 4 > data.len() {
            return None;
        }
        let seeds_len = u32::from_be_bytes(data[offset..offset + 4].try_into().ok()?) as usize;
        offset += 4;

        if offset + seeds_len * 8 > data.len() {
            return None;
        }

        let mut seeds = Vec::with_capacity(seeds_len);
        for _ in 0..seeds_len {
            let seed = u64::from_be_bytes(data[offset..offset + 8].try_into().ok()?);
            offset += 8;
            seeds.push(seed);
        }

        Some(Self { positions, seeds })
    }

    /// Matches observed chunk relative offsets against this formation.
    pub fn matches(&self, observed: &[(i32, i32)]) -> bool {
        if observed.is_empty() {
            return false;
        }
        // Check if all observed coordinates match relative to some anchor
        let anchor = observed[0];
        for base in &self.positions {
            let dx = anchor.0 - base.0;
            let dz = anchor.1 - base.1;

            let mut all_match = true;
            for obs in observed {
                let shifted = (obs.0 - dx, obs.1 - dz);
                if !self.positions.contains(&shifted) {
                    all_match = false;
                    break;
                }
            }
            if all_match {
                return true;
            }
        }
        false
    }
}

pub struct TreasureDatabase {
    pub formations: Vec<TreasureFormation>,
}

impl TreasureDatabase {
    pub fn load_embedded() -> Self {
        const B1: &[u8] = include_bytes!("../../../data/buried_treasure_formation_1.bin");
        const B2: &[u8] = include_bytes!("../../../data/buried_treasure_formation_2.bin");
        const B3: &[u8] = include_bytes!("../../../data/buried_treasure_formation_3.bin");
        const B4: &[u8] = include_bytes!("../../../data/buried_treasure_formation_4.bin");
        const B5: &[u8] = include_bytes!("../../../data/buried_treasure_formation_5.bin");
        const B6: &[u8] = include_bytes!("../../../data/buried_treasure_formation_6.bin");
        const B7: &[u8] = include_bytes!("../../../data/buried_treasure_formation_7.bin");
        const B8: &[u8] = include_bytes!("../../../data/buried_treasure_formation_8.bin");
        const B9: &[u8] = include_bytes!("../../../data/buried_treasure_formation_9.bin");
        const B10: &[u8] = include_bytes!("../../../data/buried_treasure_formation_10.bin");

        let mut formations = Vec::new();
        for raw in [B1, B2, B3, B4, B5, B6, B7, B8, B9, B10] {
            if let Some(f) = TreasureFormation::parse(raw) {
                formations.push(f);
            }
        }
        Self { formations }
    }

    /// Finds candidate 48-bit seeds for a set of observed treasure chunk positions.
    pub fn find_candidate_seeds(&self, observed: &[(i32, i32)]) -> Vec<u64> {
        let mut candidates = Vec::new();
        for f in &self.formations {
            if f.matches(observed) {
                candidates.extend_from_slice(&f.seeds);
            }
        }
        candidates.sort_unstable();
        candidates.dedup();
        candidates
    }
}
