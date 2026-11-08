use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use rayon::prelude::*;

use crate::bedrock::block::BedrockBlock;
use crate::bedrock::filter::{BlockFilter, CheckObject};
use crate::bedrock::solver::{FLOOR_HASH, ROOF_HASH};
use crate::math::lcg::JavaRandom;

/// Reverses a Java Random `next_long()` call back to possible internal seeds.
/// In Java Random:
/// next_long() = ((next(32) as i64) << 32) + next(32) as i64
/// where next(32) = seed >> 16.
pub fn reverse_next_long(val: u64) -> Vec<u64> {
    let lo = val as u32;
    let hi = if (lo as i32) < 0 {
        ((val >> 32) as u32).wrapping_add(1)
    } else {
        (val >> 32) as u32
    };
    let mut matches = Vec::with_capacity(2);

    let hi_part = (hi as u64) << 16;
    for lower16 in 0..=0xFFFFu64 {
        let s1 = hi_part | lower16;
        let s2 = JavaRandom::next_seed(s1);
        if (s2 >> 16) as u32 == lo {
            // Previous seed before the next_long call
            matches.push(JavaRandom::prev_seed(s1));
        }
    }

    matches
}

/// Bit-sliced hierarchical search tree for fast Bedrock seed cracking.
/// Progressively evaluates bit constraints from 12 down to 0 bits.
#[derive(Clone, Debug)]
pub struct LayerFilterTree {
    pub bits: u32,
    pub checks: Vec<CheckObject>,
    pub next: Option<Box<LayerFilterTree>>,
}

impl LayerFilterTree {
    pub fn build(filters: &mut [BlockFilter], start_bits: u32) -> Self {
        let mut checks = Vec::new();
        for f in filters.iter_mut() {
            if f.discarded_seeds(start_bits as u64) > 0.0 {
                checks.push(f.create_check(start_bits as u64));
            }
        }

        let next = if start_bits > 0 {
            Some(Box::new(Self::build(filters, start_bits - 1)))
        } else {
            None
        };

        Self {
            bits: start_bits,
            checks,
            next,
        }
    }

    /// Evaluates checks at this bit level. If passing, descends to the next bit level.
    pub fn evaluate(
        &self,
        upper_bits: u64,
        secondary_checks: &[CheckObject],
        results: &mut Vec<u64>,
    ) {
        for check in &self.checks {
            if check.check(upper_bits) {
                return; // Failed branchless check, prune subtree
            }
        }

        if let Some(ref next_layer) = self.next {
            let split = 1u64 << (self.bits.saturating_sub(1));
            next_layer.evaluate(upper_bits, secondary_checks, results);
            next_layer.evaluate(upper_bits + split, secondary_checks, results);
        } else {
            // Terminal layer (0 bits). Verify against secondary checks and reverse
            let mut matches_secondary = true;
            for check in secondary_checks {
                if check.check(upper_bits) {
                    matches_secondary = false;
                    break;
                }
            }

            if matches_secondary {
                results.push(upper_bits);
            }
        }
    }
}

/// High-speed Bedrock searcher using hierarchical bit-sliced trees.
pub struct HierarchicalBedrockCracker;

impl HierarchicalBedrockCracker {
    pub fn search(
        blocks: &[BedrockBlock],
        threads: usize,
        cancel: Option<Arc<AtomicBool>>,
    ) -> Vec<u64> {
        let mut floor_filters = Vec::new();
        let mut roof_filters = Vec::new();

        for b in blocks {
            let filter = BlockFilter::from_block(b);
            if b.is_roof() {
                roof_filters.push(filter);
            } else {
                floor_filters.push(filter);
            }
        }

        let (mut primary, mut secondary, primary_hash, _secondary_hash) =
            if floor_filters.len() >= roof_filters.len() {
                (floor_filters, roof_filters, FLOOR_HASH, ROOF_HASH)
            } else {
                (roof_filters, floor_filters, ROOF_HASH, FLOOR_HASH)
            };

        let tree = LayerFilterTree::build(&mut primary, 12);
        let secondary_checks: Vec<CheckObject> = secondary
            .iter_mut()
            .map(|f| f.create_check(0))
            .collect();

        // Search the 36-bit prefix space in parallel chunks
        let total_chunks = (threads * 16).max(64);
        let step = (1u64 << 36) / total_chunks as u64;

        (0..total_chunks)
            .into_par_iter()
            .flat_map(|chunk_idx| {
                if let Some(ref c) = cancel {
                    if c.load(Ordering::Relaxed) {
                        return Vec::new();
                    }
                }

                let start = (chunk_idx as u64) * step;
                let end = start + step;
                let mut local_results = Vec::new();

                for prefix in start..end {
                    let seed_prefix = prefix << 12;
                    let mut matched_seeds = Vec::new();
                    tree.evaluate(seed_prefix, &secondary_checks, &mut matched_seeds);

                    for seed in matched_seeds {
                        // Reverse through secondary hash and PRNG
                        for raw_seed in reverse_next_long(seed ^ primary_hash) {
                            local_results.push(raw_seed);
                        }
                    }
                }

                local_results
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::{MASK48, MULTIPLIER};

    #[test]
    fn test_reverse_next_long() {
        let original_seed = 9876543210123u64;
        let mut r = JavaRandom::new(original_seed);
        let long_val = r.next_long() as u64;

        let reversed = reverse_next_long(long_val);
        let expected_raw = (original_seed ^ MULTIPLIER) & MASK48;
        assert!(reversed.contains(&expected_raw));
    }
}
