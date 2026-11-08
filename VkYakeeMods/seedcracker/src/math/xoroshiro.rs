#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Xoroshiro128PlusPlus {
    pub s0: u64,
    pub s1: u64,
}

impl Xoroshiro128PlusPlus {
    pub fn new(seed0: u64, seed1: u64) -> Self {
        Self {
            s0: seed0,
            s1: seed1,
        }
    }

    pub fn from_seed(seed: u64) -> Self {
        // Minecraft initializes Xoroshiro state from 64-bit seed using SplitMix64
        let mut sm = SplitMix64::new(seed);
        let s0 = sm.next();
        let s1 = sm.next();
        Self { s0, s1 }
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let s0 = self.s0;
        let mut s1 = self.s1;
        let result = (s0.wrapping_add(s1).rotate_left(17)).wrapping_add(s0);

        s1 ^= s0;
        self.s0 = s0.rotate_left(49) ^ s1 ^ (s1 << 21);
        self.s1 = s1.rotate_left(28);

        result
    }

    #[inline]
    pub fn next_int_bounded(&mut self, bound: i32) -> i32 {
        assert!(bound > 0);
        let bound = bound as u64;
        let mut r = self.next_u64() >> 32;
        let mut m = r * bound;
        let mut l = m as u32 as u64;
        if l < bound {
            let t = (0u64.wrapping_sub(bound)) % bound;
            while l < t {
                r = self.next_u64() >> 32;
                m = r * bound;
                l = m as u32 as u64;
            }
        }
        (m >> 32) as i32
    }
}

pub struct SplitMix64 {
    x: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self { x: seed }
    }

    pub fn next(&mut self) -> u64 {
        self.x = self.x.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.x;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}
