//! Java's 48-bit LCG implementation matching Java's Random exactly.
//! This is the core RNG used throughout Minecraft world generation.

/// Java LCG multiplier
pub const MULTIPLIER: u64 = 0x5DEECE66D;
/// Java LCG addend
pub const ADDEND: u64 = 11;
/// 48-bit mask
pub const MASK48: u64 = 0xFFFFFFFFFFFF;
/// Modular inverse of multiplier for reversing LCG
pub const INV_MULTIPLIER: u64 = 0xDFE05BCB1365;
/// Modular inverse of addend for reversing LCG
pub const INV_ADDEND: u64 = 0x615C0E462AA9;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct JavaRandom {
    pub seed: u64,
}

impl JavaRandom {
    /// Create new JavaRandom from a 64-bit seed (applies Java's seed initialization)
    #[inline]
    pub fn new(seed: u64) -> Self {
        Self {
            seed: (seed ^ MULTIPLIER) & MASK48,
        }
    }

    /// Create from already-masked 48-bit state
    #[inline]
    pub const fn from_state(state: u64) -> Self {
        Self { seed: state & MASK48 }
    }

    /// Set seed with Java's initialization
    #[inline]
    pub fn set_seed(&mut self, seed: u64) {
        self.seed = (seed ^ MULTIPLIER) & MASK48;
    }

    /// Get next `bits` bits
    #[inline]
    pub fn next(&mut self, bits: u32) -> i32 {
        self.seed = self.seed.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND) & MASK48;
        (self.seed >> (48 - bits)) as i32
    }

    #[inline]
    pub fn next_int(&mut self) -> i32 {
        self.next(32)
    }

    #[inline]
    pub fn next_int_bounded(&mut self, bound: i32) -> Option<i32> {
        if bound <= 0 {
            return None;
        }
        if (bound & (bound - 1)) == 0 {
            // Power of 2 optimization
            return Some((((bound as i64) * (self.next(31) as i64)) >> 31) as i32);
        }
        let mut bits = self.next(31);
        let mut val = bits % bound;
        while bits - val + (bound - 1) < 0 {
            bits = self.next(31);
            val = bits % bound;
        }
        Some(val)
    }

    #[inline]
    pub fn next_long(&mut self) -> i64 {
        ((self.next(32) as i64) << 32).wrapping_add(self.next(32) as i64)
    }

    #[inline]
    pub fn next_float(&mut self) -> f32 {
        self.next(24) as f32 / ((1 << 24) as f32)
    }

    #[inline]
    pub fn next_double(&mut self) -> f64 {
        let high = (self.next(26) as i64) << 27;
        let low = self.next(27) as i64;
        (high.wrapping_add(low)) as f64 / ((1i64 << 53) as f64)
    }

    /// Get next seed (forward one LCG step)
    #[inline]
    pub fn next_seed(seed: u64) -> u64 {
        (seed.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND)) & MASK48
    }

    /// Get previous seed (reverse one LCG step)
    #[inline]
    pub fn prev_seed(seed: u64) -> u64 {
        (seed.wrapping_mul(INV_MULTIPLIER).wrapping_add(INV_ADDEND)) & MASK48
    }

    /// Compute combined multiplier and addend for `steps` LCG steps.
    /// Positive steps = forward, negative steps = backward.
    pub fn combine_steps(steps: i64) -> (u64, u64) {
        if steps == 0 {
            return (1, 0);
        }
        let (mut mult, add) = if steps > 0 {
            (MULTIPLIER, ADDEND)
        } else {
            (INV_MULTIPLIER, INV_ADDEND)
        };

        let mut exp = steps.unsigned_abs();
        let mut cur_mult = 1u64;
        let mut cur_add = 0u64;

        while exp > 0 {
            if exp & 1 == 1 {
                cur_mult = (cur_mult.wrapping_mul(mult)) & MASK48;
                cur_add = (cur_add.wrapping_mul(mult).wrapping_add(add)) & MASK48;
            }
            exp >>= 1;
            if exp > 0 {
                cur_add = (mult.wrapping_add(1).wrapping_mul(add)) & MASK48;
                mult = (mult.wrapping_mul(mult)) & MASK48;
            }
        }

        (cur_mult, cur_add)
    }

    /// Advance the generator by `steps` (positive = forward, negative = backward)
    pub fn advance(&mut self, steps: i64) {
        let (mult, add) = Self::combine_steps(steps);
        self.seed = (self.seed.wrapping_mul(mult).wrapping_add(add)) & MASK48;
    }

    /// Get the internal 48-bit state
    #[inline]
    pub fn state(&self) -> u64 {
        self.seed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_lcg_sequence() {
        let mut rng = JavaRandom::new(0);
        assert_eq!(rng.next_int(), -1155484576);
        assert_eq!(rng.next_int(), -723955400);
    }

    #[test]
    fn test_next_long() {
        let mut rng = JavaRandom::new(0);
        assert_eq!(rng.next_long(), -4962768465676381896);
    }

    #[test]
    fn test_reversibility() {
        let seed = 0x123456789ABCu64 & MASK48;
        let mut rng = JavaRandom::from_state(seed);
        let next = rng.next_long();
        let prev = JavaRandom::prev_seed(rng.seed);
        assert_eq!(prev, seed);
    }

    #[test]
    fn test_combine_steps() {
        let (mult_2, add_2) = JavaRandom::combine_steps(2);
        let seed = 0x123456789ABCu64 & MASK48;
        let mut rng1 = JavaRandom::from_state(seed);
        rng1.advance(2);
        let seed2 = (seed.wrapping_mul(mult_2).wrapping_add(add_2)) & MASK48;
        assert_eq!(rng1.seed, seed2);
    }

    #[test]
    fn test_combine_steps_inverse() {
        let (mult_f, add_f) = JavaRandom::combine_steps(5);
        let (mult_b, add_b) = JavaRandom::combine_steps(-5);
        let seed = 0x123456789ABCu64 & MASK48;
        let forward = (seed.wrapping_mul(mult_f).wrapping_add(add_f)) & MASK48;
        let backward = (forward.wrapping_mul(mult_b).wrapping_add(add_b)) & MASK48;
        assert_eq!(backward, seed);
    }
}
