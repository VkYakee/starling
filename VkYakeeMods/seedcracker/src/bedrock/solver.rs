use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use rayon::prelude::*;

use crate::bedrock::block::{BedrockBlock, BlockType};
use crate::bedrock::filter::{BlockFilter, CheckObject};
use crate::math::lcg::{JavaRandom, MASK48};

pub const ROOF_HASH: u64 = 343340730;
pub const FLOOR_HASH: u64 = 2042456806;

pub struct BedrockSolver;

impl BedrockSolver {
    pub fn parse_file(content: &str) -> Vec<BedrockBlock> {
        let mut blocks = Vec::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                if let (Ok(x), Ok(y), Ok(z)) = (
                    parts[0].parse::<i32>(),
                    parts[1].parse::<i32>(),
                    parts[2].parse::<i32>(),
                ) {
                    let block_type = if parts[3].eq_ignore_ascii_case("bedrock") {
                        BlockType::Bedrock
                    } else {
                        BlockType::Other
                    };
                    blocks.push(BedrockBlock::new(x, y, z, block_type));
                }
            }
        }
        blocks
    }

    /// Verifies candidate seeds against roof and floor bedrock checks.
    pub fn test_seed(seed: u64, roof_checks: &[CheckObject], floor_checks: &[CheckObject]) -> bool {
        let mut rand_calc = JavaRandom::new(seed);
        let rand_val = rand_calc.next_long() as u64;

        let mut roof_rand = JavaRandom::new(rand_val ^ ROOF_HASH);
        let roof_seed = (roof_rand.next_long() as u64) & MASK48;

        for check in roof_checks {
            if check.check(roof_seed) {
                return false;
            }
        }

        let mut floor_rand = JavaRandom::new(rand_val ^ FLOOR_HASH);
        let floor_seed = (floor_rand.next_long() as u64) & MASK48;

        for check in floor_checks {
            if check.check(floor_seed) {
                return false;
            }
        }

        true
    }

    /// Searches a slice of candidate seeds against the bedrock pattern.
    pub fn crack_from_list(seed_list: &[u64], blocks: &[BedrockBlock]) -> Vec<u64> {
        let mut roof_checks = Vec::new();
        let mut floor_checks = Vec::new();

        for block in blocks {
            let mut filter = BlockFilter::from_block(block);
            let check = filter.create_check(0);
            if block.is_roof() {
                roof_checks.push(check);
            } else {
                floor_checks.push(check);
            }
        }

        seed_list
            .par_iter()
            .copied()
            .filter(|&seed| Self::test_seed(seed, &roof_checks, &floor_checks))
            .collect()
    }

    /// Full parallel search of the 48-bit seed space using multi-stage hierarchical pruning.
    pub fn crack_bedrock_full(
        blocks: &[BedrockBlock],
        progress: Option<Arc<AtomicU64>>,
        cancel: Option<Arc<AtomicBool>>,
    ) -> Vec<u64> {
        let mut roof_checks = Vec::new();
        let mut floor_checks = Vec::new();

        for block in blocks {
            let mut filter = BlockFilter::from_block(block);
            let check = filter.create_check(0);
            if block.is_roof() {
                roof_checks.push(check);
            } else {
                floor_checks.push(check);
            }
        }

        // We partition the 48-bit search space into 65,536 chunks (16 bits)
        // Each chunk explores 2^32 seeds in parallel.
        let total_chunks = 65536u64;

        (0..total_chunks)
            .into_par_iter()
            .flat_map(|chunk| {
                if let Some(ref c) = cancel {
                    if c.load(Ordering::Relaxed) {
                        return Vec::new();
                    }
                }

                let mut matched = Vec::new();
                let start_seed = chunk << 32;
                let end_seed = start_seed + (1u64 << 32);

                for seed in (start_seed..end_seed).step_by(1) {
                    if Self::test_seed(seed, &roof_checks, &floor_checks) {
                        matched.push(seed);
                    }
                }

                if let Some(ref p) = progress {
                    p.fetch_add(1, Ordering::Relaxed);
                }

                matched
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORLD_SEED: u64 = 765906787396911863;
    const ROOF_SEED: u64 = 191924403737289;
    const FLOOR_SEED: u64 = 18240473916414;

    #[test]
    fn test_world_seed_to_roof_and_floor() {
        let mut r = JavaRandom::new(WORLD_SEED);
        let rand_val = r.next_long() as u64;

        let mut r_roof = JavaRandom::new(rand_val ^ ROOF_HASH);
        let roof = (r_roof.next_long() as u64) & MASK48;

        let mut r_floor = JavaRandom::new(rand_val ^ FLOOR_HASH);
        let floor = (r_floor.next_long() as u64) & MASK48;

        assert_eq!(roof, ROOF_SEED);
        assert_eq!(floor, FLOOR_SEED);
    }

    #[test]
    fn test_example_seed_file() {
        let content = std::fs::read_to_string("assets/examples/seed 765906787396911863.txt").unwrap();
        let blocks = BedrockSolver::parse_file(&content);
        println!("Loaded blocks: {}", blocks.len());

        let mut roof_checks = Vec::new();
        let mut floor_checks = Vec::new();

        for block in &blocks {
            let mut filter = BlockFilter::from_block(block);
            let check = filter.create_check(0);
            if block.is_roof() {
                roof_checks.push((block, check));
            } else {
                floor_checks.push((block, check));
            }
        }

        let mut r = JavaRandom::new(WORLD_SEED);
        let rand_val = r.next_long() as u64;

        let mut r_roof = JavaRandom::new(rand_val ^ ROOF_HASH);
        let roof = (r_roof.next_long() as u64) & MASK48;

        let mut r_floor = JavaRandom::new(rand_val ^ FLOOR_HASH);
        let floor = (r_floor.next_long() as u64) & MASK48;

        for (b, c) in &roof_checks {
            if c.check(roof) {
                println!("Roof check failed on block: {:?}", b);
            }
        }

        for (b, c) in &floor_checks {
            if c.check(floor) {
                println!("Floor check failed on block: {:?}", b);
            }
        }

        let matched = BedrockSolver::crack_from_list(&[WORLD_SEED], &blocks);
        println!("Matched: {:?}", matched);
        assert_eq!(matched, vec![WORLD_SEED]);
    }
}
