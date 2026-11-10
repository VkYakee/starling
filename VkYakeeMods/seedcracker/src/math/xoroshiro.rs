//! Xoroshiro128++ implementation matching Minecraft's usage.
//! Used for some world generation features.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Xoroshiro128PlusPlus {
    state: [u64; 2],
}

impl Xoroshiro128PlusPlus {
    const GOLDEN_RATIO: u64 = 0x9e3779b97f4a7c15;
    const SILVER_RATIO: u64 = 0x6a09e667f3bcc909;

    /// Create new generator from seed
    pub fn new(seed: u64) -> Self {
        let mut state = [0u64; 2];
        state[0] = splitmix64(seed.wrapping_add(Self::GOLDEN_RATIO));
        state[1] = splitmix64(state[0].wrapping_add(Self::SILVER_RATIO));
        Self { state }
    }

    /// Create from existing state
    pub fn from_state(s0: u64, s1: u64) -> Self {
        Self { state: [s0, s1] }
    }

    /// Get next 64-bit value
    pub fn next_long(&mut self) -> u64 {
        let s0 = self.state[0];
        let s1 = self.state[1];
        let result = s0.wrapping_add(s1).rotate_left(17).wrapping_add(s0);
        
        self.state[0] = s0.rotate_left(49) ^ s1 ^ (s1 << 21);
        self.state[1] = s1.rotate_left(28);
        
        result
    }

    /// Get next 32-bit value
    pub fn next_int(&mut self) -> u32 {
        self.next_long() as u32
    }

    /// Get current state
    pub fn state(&self) -> [u64; 2] {
        self.state
    }
}

/// SplitMix64 for seeding
fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xoroshiro_deterministic() {
        let mut rng = Xoroshiro128PlusPlus::new(0);
        let v1 = rng.next_long();
        let v2 = rng.next_long();
        
        let mut rng2 = Xoroshiro128PlusPlus::new(0);
        assert_eq!(rng2.next_long(), v1);
        assert_eq!(rng2.next_long(), v2);
    }
}
