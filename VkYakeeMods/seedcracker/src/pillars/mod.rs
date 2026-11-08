use crate::math::lcg::{JavaRandom, MASK48, MULTIPLIER};

pub const PILLAR_COUNT: usize = 10;
pub const PILLAR_BASE_HEIGHT: i32 = 76;
pub const PILLAR_STEP: i32 = 3;

#[derive(Debug, Clone)]
pub struct EndPillars;

impl EndPillars {
    /// Simulates Collections.shuffle(0..10, new Random(pillar_seed)) as in Minecraft.
    pub fn get_pillar_heights(pillar_seed: u16) -> [i32; PILLAR_COUNT] {
        let mut indices: [usize; PILLAR_COUNT] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut rand = JavaRandom::new(pillar_seed as u64);

        // Java Collections.shuffle algorithm:
        for i in (2..=PILLAR_COUNT).rev() {
            let j = rand.next_int_bounded(i as i32) as usize;
            indices.swap(i - 1, j);
        }

        let mut heights = [0i32; PILLAR_COUNT];
        for k in 0..PILLAR_COUNT {
            heights[k] = PILLAR_BASE_HEIGHT + (indices[k] as i32) * PILLAR_STEP;
        }
        heights
    }

    /// Finds all 16-bit pillar seeds (0..65536) that match observed pillar heights.
    /// observed_heights: array with None for unknown pillars, Some(height) for known pillars.
    pub fn find_pillar_seeds(observed_heights: &[Option<i32>; PILLAR_COUNT]) -> Vec<u16> {
        let mut matching_seeds = Vec::new();
        for seed in 0..=0xFFFFu16 {
            let heights = Self::get_pillar_heights(seed);
            let mut matches = true;
            for i in 0..PILLAR_COUNT {
                if let Some(expected) = observed_heights[i] {
                    if heights[i] != expected {
                        matches = false;
                        break;
                    }
                }
            }
            if matches {
                matching_seeds.push(seed);
            }
        }
        matching_seeds
    }

    /// Computes the 16-bit pillar seed from a 48-bit structure seed.
    pub fn pillar_seed_from_structure_seed(structure_seed: u64) -> u16 {
        // intermediate = LCG^2(structure_seed ^ MULTIPLIER)
        let (mult_2, add_2) = JavaRandom::combine_steps(2);
        let s0 = structure_seed ^ MULTIPLIER;
        let s2 = (s0.wrapping_mul(mult_2).wrapping_add(add_2)) & MASK48;
        ((s2 >> 16) & 0xFFFF) as u16
    }

    /// TimeMachine formula: reconstructs 48-bit seed from 32-bit partial world seed and 16-bit pillar seed.
    pub fn time_machine_combine(partial_world_seed: u64, pillar_seed: u16) -> u64 {
        let mut current_seed = 0u64;
        current_seed |= (partial_world_seed & 0xFFFF0000) << 16;
        current_seed |= (pillar_seed as u64) << 16;
        current_seed |= partial_world_seed & 0xFFFF;

        // LCG^(-2)
        let (inv_mult_2, inv_add_2) = JavaRandom::combine_steps(-2);
        current_seed = (current_seed.wrapping_mul(inv_mult_2).wrapping_add(inv_add_2)) & MASK48;
        current_seed ^= MULTIPLIER;
        current_seed & MASK48
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pillar_reversible() {
        let pillar_seed = 12345u16;
        let heights = EndPillars::get_pillar_heights(pillar_seed);
        let mut observed = [None; 10];
        observed[0] = Some(heights[0]);
        observed[1] = Some(heights[1]);
        observed[2] = Some(heights[2]);
        observed[3] = Some(heights[3]);
        let candidates = EndPillars::find_pillar_seeds(&observed);
        assert!(candidates.contains(&pillar_seed));
    }
}
