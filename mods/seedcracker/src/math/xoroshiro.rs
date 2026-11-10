//! Xoroshiro128++ implementation matching Minecraft's usage
//! Used for structure placement and other worldgen

/// Xoroshiro128++ random number generator
/// Matches Minecraft's implementation exactly
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Xoroshiro128PlusPlus {
    state: [u64; 2],
}

impl Xoroshiro128PlusPlus {
    /// Create from a 64-bit seed (splits into two 64-bit states)
    pub fn new(seed: u64) -> Self {
        let mut splitmix = SplitMix64::new(seed);
        Self {
            state: [splitmix.next(), splitmix.next()],
        }
    }

    /// Create from two 64-bit states
    pub fn from_state(s0: u64, s1: u64) -> Self {
        Self { state: [s0, s1] }
    }

    /// Get the internal state
    pub fn state(&self) -> [u64; 2] {
        self.state
    }

    /// Next 64-bit value
    pub fn next(&mut self) -> u64 {
        let s0 = self.state[0];
        let mut s1 = self.state[1];
        let result = s0.wrapping_add(s1).rotate_left(17).wrapping_add(s0);
        
        s1 ^= s0;
        self.state[0] = s0.rotate_left(49) ^ s1 ^ (s1 << 21);
        self.state[1] = s1.rotate_left(28);
        
        result
    }

    /// Next 32-bit value
    pub fn next_int(&mut self) -> i32 {
        self.next() as i32
    }

    /// Next boolean
    pub fn next_boolean(&mut self) -> bool {
        self.next() & 1 != 0
    }

    /// Next float
    pub fn next_float(&mut self) -> f32 {
        (self.next() >> 40) as f32 * (1.0 / (1u64 << 24) as f32)
    }

    /// Next double
    pub fn next_double(&mut self) -> f64 {
        (self.next() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// Next bounded int
    pub fn next_int_bounded(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            return 0;
        }
        
        let bound = bound as u64;
        if (bound & (bound - 1)) == 0 {
            return ((bound.wrapping_mul(self.next() >> 1)) >> 63) as i32;
        }

        let mut bits = self.next() >> 1;
        let mut val = bits % bound;
        while bits.wrapping_sub(val).wrapping_add(bound - 1) < bound {
            bits = self.next() >> 1;
            val = bits % bound;
        }
        val as i32
    }

    /// Jump function for parallel streams (2^64 steps)
    pub fn jump(&mut self) {
        const JUMP: [u64; 2] = [0x2BD7A6A6E99C2DDC, 0x0992CCAF6A6FCA05];
        let mut s0 = 0;
        let mut s1 = 0;
        for j in JUMP {
            for b in 0..64 {
                if (j >> b) & 1 != 0 {
                    s0 ^= self.state[0];
                    s1 ^= self.state[1];
                }
                self.next();
            }
        }
        self.state = [s0, s1];
    }

    /// Long jump function for parallel streams (2^96 steps)
    pub fn long_jump(&mut self) {
        const LONG_JUMP: [u64; 2] = [0x360FDAA92A4FC2F9, 0x5D2D2211735ECB51];
        let mut s0 = 0;
        let mut s1 = 0;
        for j in LONG_JUMP {
            for b in 0..64 {
                if (j >> b) & 1 != 0 {
                    s0 ^= self.state[0];
                    s1 ^= self.state[1];
                }
                self.next();
            }
        }
        self.state = [s0, s1];
    }
}

/// SplitMix64 for seeding Xoroshiro
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ z >> 31
    }
}

impl Default for Xoroshiro128PlusPlus {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xoroshiro_deterministic() {
        let mut rng1 = Xoroshiro128PlusPlus::new(12345);
        let mut rng2 = Xoroshiro128PlusPlus::new(12345);
        
        for _ in 0..100 {
            assert_eq!(rng1.next(), rng2.next());
        }
    }

    #[test]
    fn test_next_int_bounded_distribution() {
        let mut rng = Xoroshiro128PlusPlus::new(12345);
        let mut counts = [0; 10];
        for _ in 0..10000 {
            let v = rng.next_int_bounded(10);
            counts[v as usize] += 1;
        }
        for c in counts {
            assert!(c > 800 && c < 1200, "Distribution not uniform: {}", c);
        }
    }
}
