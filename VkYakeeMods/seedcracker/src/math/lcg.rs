pub const MULTIPLIER: u64 = 0x5DEECE66D;
pub const ADDEND: u64 = 11;
pub const MASK48: u64 = 0xFFFFFFFFFFFF;
pub const INV_MULTIPLIER: u64 = 0xDFE05BCB1365;
pub const INV_ADDEND: u64 = 0x615C0E462AA9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JavaRandom {
    pub seed: u64,
}

impl JavaRandom {
    #[inline]
    pub fn new(seed: u64) -> Self {
        Self {
            seed: (seed ^ MULTIPLIER) & MASK48,
        }
    }

    #[inline]
    pub fn with_raw_seed(seed: u64) -> Self {
        Self {
            seed: seed & MASK48,
        }
    }

    #[inline]
    pub fn set_seed(&mut self, seed: u64) {
        self.seed = (seed ^ MULTIPLIER) & MASK48;
    }

    #[inline]
    pub fn next(&mut self, bits: u32) -> i32 {
        self.seed = (self.seed.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND)) & MASK48;
        (self.seed >> (48 - bits)) as i32
    }

    #[inline]
    pub fn next_int(&mut self) -> i32 {
        self.next(32)
    }

    #[inline]
    pub fn next_int_bounded(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "bound must be positive");
        if (bound & (bound - 1)) == 0 {
            // Power of 2 optimization
            return (((bound as i64) * (self.next(31) as i64)) >> 31) as i32;
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

    #[inline]
    pub fn next_seed(seed: u64) -> u64 {
        (seed.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND)) & MASK48
    }

    #[inline]
    pub fn prev_seed(seed: u64) -> u64 {
        (seed.wrapping_mul(INV_MULTIPLIER).wrapping_add(INV_ADDEND)) & MASK48
    }

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

    pub fn advance(&mut self, steps: i64) {
        let (mult, add) = Self::combine_steps(steps);
        self.seed = (self.seed.wrapping_mul(mult).wrapping_add(add)) & MASK48;
    }

    /// Reverses a Java Random nextLong() return value into the internal 48-bit seed.
    /// In Java, nextLong() is: ((next(32) << 32) + next(32)).
    pub fn reverse_next_long(high: i32, low: i32) -> Vec<u64> {
        let mut results = Vec::new();
        let high_u64 = (high as u64 & 0xFFFFFFFF) << 16;
        for lower_bits in 0..=0xFFFF {
            let candidate_seed1 = high_u64 | lower_bits;
            let candidate_seed2 = Self::next_seed(candidate_seed1);
            if (candidate_seed2 >> 16) as i32 == low {
                results.push(candidate_seed1);
            }
        }
        results
    }
}
