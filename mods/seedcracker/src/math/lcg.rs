//! Java's Linear Congruential Generator (LCG) implementation
//! Matches Java's `java.util.Random` exactly

/// Java's LCG multiplier
pub const MULTIPLIER: u64 = 0x5DEECE66D;
/// Java's LCG addend
pub const ADDEND: u64 = 0xB;
/// Java's LCG mask (48 bits)
pub const MASK48: u64 = (1u64 << 48) - 1;
/// Inverse multiplier for reversing LCG
pub const INV_MULTIPLIER: u64 = 0xDFE05BCB1365;
/// Inverse addend for reversing LCG
pub const INV_ADDEND: u64 = 0xC4CEB9FE6A5B;

/// Java's Random implementation matching java.util.Random exactly
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JavaRandom {
    state: u64,
}

impl JavaRandom {
    /// Create a new JavaRandom with the given seed
    /// Matches Java's `new Random(seed)` constructor
    pub fn new(seed: u64) -> Self {
        let state = (seed ^ MULTIPLIER) & MASK48;
        Self { state }
    }

    /// Create with raw state (already scrambled)
    pub fn with_state(state: u64) -> Self {
        Self { state: state & MASK48 }
    }

    /// Get the internal state
    pub fn state(&self) -> u64 {
        self.state
    }

    /// Next random long (48 bits)
    /// Matches Java's `nextLong()` which uses next(32) twice
    pub fn next_long(&mut self) -> i64 {
        ((self.next(32) as i64) << 32) | (self.next(32) as i64)
    }

    /// Next random int (32 bits)
    /// Matches Java's `nextInt()`
    pub fn next_int(&mut self) -> i32 {
        self.next(32) as i32
    }

    /// Next random int bounded by n
    /// Matches Java's `nextInt(int bound)`
    pub fn next_int_bounded(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            return 0;
        }
        
        // Special case for power of 2
        if (bound & (bound - 1)) == 0 {
            return ((bound as i64 * self.next(31) as i64) >> 31) as i32;
        }

        let mut bits: i32;
        let mut val: i32;
        loop {
            bits = self.next(31);
            val = bits % bound;
            if bits - val + (bound - 1) >= 0 {
                return val;
            }
        }
    }

    /// Next random boolean
    /// Matches Java's `nextBoolean()`
    pub fn next_boolean(&mut self) -> bool {
        self.next(1) != 0
    }

    /// Next random float
    /// Matches Java's `nextFloat()`
    pub fn next_float(&mut self) -> f32 {
        self.next(24) as f32 / (1 << 24) as f32
    }

    /// Next random double
    /// Matches Java's `nextDouble()`
    pub fn next_double(&mut self) -> f64 {
        (((self.next(26) as u64) << 27) | (self.next(27) as u64)) as f64 / ((1u64 << 53) as f64)
    }

    /// Next random bits (core of Java's Random)
    /// Matches Java's `next(int bits)`
    pub fn next(&mut self, bits: u32) -> i32 {
        self.state = (self.state.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND)) & MASK48;
        (self.state >> (48 - bits)) as i32
    }

    /// Set seed to a specific value (matches Java's `setSeed`)
    pub fn set_seed(&mut self, seed: u64) {
        self.state = (seed ^ MULTIPLIER) & MASK48;
    }

    /// Advance the LCG by n steps
    pub fn advance(&mut self, n: i64) {
        if n >= 0 {
            for _ in 0..n {
                self.next(1);
            }
        } else {
            // Reverse using inverse LCG
            for _ in 0..(-n) {
                self.state = self.state.wrapping_sub(ADDEND).wrapping_mul(INV_MULTIPLIER) & MASK48;
            }
        }
    }

    /// Combine multiple LCG steps (for computing LCG^n efficiently)
    /// Returns (multiplier, addend) for n steps
    pub fn combine_steps(n: i64) -> (u64, u64) {
        if n == 0 {
            return (1, 0);
        }
        
        let mut mult = MULTIPLIER;
        let mut add = ADDEND;
        let mut n = n.abs();
        let mut result_mult = 1u64;
        let mut result_add = 0u64;
        
        while n > 0 {
            if n & 1 == 1 {
                result_add = result_add.wrapping_mul(mult).wrapping_add(add) & MASK48;
                result_mult = result_mult.wrapping_mul(mult) & MASK48;
            }
            add = add.wrapping_mul(mult).wrapping_add(add) & MASK48;
            mult = mult.wrapping_mul(mult) & MASK48;
            n >>= 1;
        }
        
        if n < 0 {
            // For inverse, we need to compute the inverse of the combined transformation
            // This is more complex, so we fall back to iterative for negative
            panic!("Use iterative for negative steps");
        }
        
        (result_mult, result_add)
    }

    /// Compute the seed after n steps
    pub fn seed_after(&self, n: i64) -> u64 {
        if n == 0 {
            return self.state;
        }
        
        if n > 0 {
            let (mult, add) = Self::combine_steps(n);
            (self.state.wrapping_mul(mult).wrapping_add(add)) & MASK48
        } else {
            // For negative, compute inverse
            let (mult, add) = Self::combine_steps(-n);
            // Inverse: s' = (s - add) * inv(mult)
            // We need modular inverse of mult
            let inv_mult = Self::mod_inverse(mult);
            ((self.state.wrapping_sub(add)).wrapping_mul(inv_mult)) & MASK48
        }
    }

    /// Modular inverse using extended Euclidean algorithm
    fn mod_inverse(a: u64) -> u64 {
        let mut t = 0i128;
        let mut newt = 1i128;
        let mut r = MASK48 as i128;
        let mut newr = a as i128;
        
        while newr != 0 {
            let quotient = r / newr;
            (t, newt) = (newt, t - quotient * newt);
            (r, newr) = (newr, r - quotient * newr);
        }
        
        if t < 0 {
            (t + (MASK48 as i128)) as u64
        } else {
            t as u64
        }
    }
}

impl Default for JavaRandom {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lcg_matches_java() {
        let mut rand = JavaRandom::new(0);
        // Java: new Random(0).nextInt() == -1155484576
        assert_eq!(rand.next_int(), -1155484576);
    }

    #[test]
    fn test_next_int_bounded() {
        let mut rand = JavaRandom::new(12345);
        let mut counts = [0; 10];
        for _ in 0..10000 {
            let v = rand.next_int_bounded(10);
            counts[v as usize] += 1;
        }
        // All should be roughly 1000
        for c in counts {
            assert!(c > 800 && c < 1200, "Distribution not uniform");
        }
    }

    #[test]
    fn test_advance_reversible() {
        let mut rand1 = JavaRandom::new(0x123456789ABC);
        let mut rand2 = rand1.clone();
        
        rand1.advance(100);
        rand2.advance(200);
        rand2.advance(-100);
        
        assert_eq!(rand1.state(), rand2.state());
    }

    #[test]
    fn test_combine_steps() {
        let mut rand1 = JavaRandom::new(12345);
        let mut rand2 = JavaRandom::new(12345);
        
        // Advance by 5 steps
        for _ in 0..5 {
            rand1.next(1);
        }
        
        // Use combine_steps
        let (mult, add) = JavaRandom::combine_steps(5);
        let combined = (rand2.state.wrapping_mul(mult).wrapping_add(add)) & MASK48;
        rand2.state = combined;
        
        assert_eq!(rand1.state(), rand2.state());
    }
}
