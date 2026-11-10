use crate::math::chunk_rand;
use crate::math::lcg::{JavaRandom, MASK48, MULTIPLIER};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecoratorType {
    Dungeon,
    DesertWell,
    EmeraldOre,
    WarpedFungus,
}

#[derive(Debug, Clone)]
pub struct EmeraldOreData {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub block_x: i32,
    pub block_y: i32,
    pub block_z: i32,
    pub step: u64,
    pub index: u64,
}

impl EmeraldOreData {
    pub fn new(chunk_x: i32, chunk_z: i32, block_x: i32, block_y: i32, block_z: i32) -> Self {
        // Minecraft 1.16 default: step 6, index 14
        Self {
            chunk_x,
            chunk_z,
            block_x,
            block_y,
            block_z,
            step: 6,
            index: 14,
        }
    }

    pub fn salt(&self) -> u64 {
        self.index + 10000 * self.step
    }

    /// Verifies if a given structure seed generated the emerald ore at the exact position.
    pub fn test_seed(&self, structure_seed: u64) -> bool {
        let pop_seed = chunk_rand::get_population_seed(structure_seed, self.chunk_x, self.chunk_z);
        let dec_seed = ((pop_seed.wrapping_add(self.salt())) ^ MULTIPLIER) & MASK48;
        let mut rand = JavaRandom::with_raw_seed(dec_seed);

        let count = rand.next_int_bounded(6) + 3;
        let ox = self.block_x & 15;
        let oz = self.block_z & 15;

        for _ in 0..count {
            let x = rand.next_int_bounded(16);
            let z = rand.next_int_bounded(16);
            let y = rand.next_int_bounded(28) + 4;

            if x == ox && z == oz && y == self.block_y {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone)]
pub struct DesertWellData {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub block_x: i32,
    pub block_z: i32,
    pub salt: u64,
}

impl DesertWellData {
    pub fn new(chunk_x: i32, chunk_z: i32, block_x: i32, block_z: i32) -> Self {
        Self {
            chunk_x,
            chunk_z,
            block_x,
            block_z,
            salt: 80003, // Step 8, index 3
        }
    }

    pub fn test_seed(&self, structure_seed: u64) -> bool {
        let pop_seed = chunk_rand::get_population_seed(structure_seed, self.chunk_x, self.chunk_z);
        let dec_seed = ((pop_seed.wrapping_add(self.salt)) ^ MULTIPLIER) & MASK48;
        let mut rand = JavaRandom::with_raw_seed(dec_seed);

        // 1 in 1000 chance: nextFloat() < 0.001F
        if rand.next_float() >= 0.001 {
            return false;
        }

        let ox = rand.next_int_bounded(16);
        let oz = rand.next_int_bounded(16);

        ox == (self.block_x & 15) && oz == (self.block_z & 15)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emerald_ore_consistency() {
        let structure_seed = 765906787396911863u64;
        let chunk_x = 10;
        let chunk_z = -15;

        // Generate a known emerald placement
        let pop_seed = chunk_rand::get_population_seed(structure_seed, chunk_x, chunk_z);
        let dec_seed = ((pop_seed.wrapping_add(60014)) ^ MULTIPLIER) & MASK48;
        let mut rand = JavaRandom::with_raw_seed(dec_seed);

        let _ = rand.next_int_bounded(6) + 3;
        let ox = rand.next_int_bounded(16);
        let oz = rand.next_int_bounded(16);
        let y = rand.next_int_bounded(28) + 4;

        let block_x = (chunk_x << 4) + ox;
        let block_z = (chunk_z << 4) + oz;

        let emerald = EmeraldOreData::new(chunk_x, chunk_z, block_x, y, block_z);
        assert!(emerald.test_seed(structure_seed));
    }
}
