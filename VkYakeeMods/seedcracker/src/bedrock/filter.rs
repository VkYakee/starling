use crate::bedrock::block::{BedrockBlock, BlockType};
use crate::math::lcg::{MASK48, MULTIPLIER};

#[derive(Clone, Debug)]
pub struct BlockFilter {
    pub pos_hash: u64,
    pub lower_bound: u64,
    pub upper_bound: u64,
    pub possible_range: u64,
}

impl BlockFilter {
    pub fn from_block(b: &BedrockBlock) -> Self {
        let (lower_bound, upper_bound) = Self::bounds(b.y, b.block_type);
        let pos_hash = Self::hashcode(b.x, b.y, b.z) ^ MULTIPLIER;

        Self {
            pos_hash,
            lower_bound,
            upper_bound,
            possible_range: MASK48,
        }
    }

    pub fn create_check(&mut self, lower_bits: u64) -> CheckObject {
        let lower_bits_mask = if lower_bits >= 48 {
            MASK48
        } else {
            (1u64 << lower_bits) - 1
        };
        self.check_with_bits(lower_bits);
        CheckObject::new(
            self.pos_hash,
            self.lower_bound,
            self.upper_bound,
            lower_bits_mask,
        )
    }

    pub fn discarded_seeds(&self, lower_bits: u64) -> f64 {
        let lower_bits_mask = (1u64 << lower_bits) - 1;
        let bound = self.bound() + lower_bits_mask * MULTIPLIER;
        let success_chance = bound as f64 / self.possible_range as f64;
        let fail_chance = 1.0 - success_chance;
        if fail_chance <= 0.0 {
            return 0.0;
        }
        fail_chance * (1u64 << lower_bits) as f64
    }

    fn check_with_bits(&mut self, lower_bits: u64) {
        let lower_bits_mask = (1u64 << lower_bits) - 1;
        let jiggle_room = lower_bits_mask * MULTIPLIER;
        let new_range = jiggle_room + self.bound();
        if new_range < self.possible_range {
            self.possible_range = new_range;
        }
    }

    fn bound(&self) -> u64 {
        self.upper_bound.saturating_sub(self.lower_bound)
    }

    pub fn hashcode(x: i32, y: i32, z: i32) -> u64 {
        let mut pos_hash =
            (x.wrapping_mul(3129871)) as i64 ^ ((z as i64).wrapping_mul(116129781)) ^ y as i64;
        pos_hash = pos_hash
            .wrapping_mul(pos_hash)
            .wrapping_mul(42317861)
            .wrapping_add(pos_hash.wrapping_mul(11));
        let pos_hash = pos_hash as u64;
        pos_hash >> 16
    }

    fn bounds(mut layer: i32, block_type: BlockType) -> (u64, u64) {
        let mut lower_bound = 0.0;
        let mut upper_bound = 1.0;

        if layer > 5 {
            layer -= 122;
            let bound = (5 - layer) as f64 / 5.0;
            match block_type {
                BlockType::Bedrock => lower_bound = bound,
                BlockType::Other => upper_bound = bound,
            }
        } else {
            let bound = (5 - layer) as f64 / 5.0;
            match block_type {
                BlockType::Bedrock => upper_bound = bound,
                BlockType::Other => lower_bound = bound,
            }
        }

        lower_bound *= MASK48 as f64;
        upper_bound *= MASK48 as f64;

        (lower_bound as u64, upper_bound as u64)
    }
}

#[derive(Debug, Clone, Default)]
pub struct CheckObject {
    pub pos_hash: u64,
    pub condition: u64,
    pub offset: u64,
}

impl CheckObject {
    pub fn new(pos_hash: u64, lower_bound: u64, upper_bound: u64, lower_bit_mask: u64) -> Self {
        let offset = MASK48 - upper_bound;
        let pos_hash = pos_hash & (MASK48 - lower_bit_mask);
        let condition = lower_bound
            .wrapping_add(offset)
            .wrapping_sub(lower_bit_mask * MULTIPLIER);
        Self {
            pos_hash,
            condition,
            offset,
        }
    }

    #[inline(always)]
    pub fn check(&self, upper_bits: u64) -> bool {
        ((upper_bits ^ self.pos_hash)
            .wrapping_mul(MULTIPLIER)
            .wrapping_add(self.offset)
            & MASK48)
            < self.condition
    }
}
