use std::ffi::CStr;

pub const MC_1_12: i32 = 24;
pub const MC_1_13: i32 = 25;
pub const MC_1_14: i32 = 26;
pub const MC_1_15: i32 = 27;
pub const MC_1_16: i32 = 29;
pub const MC_1_17: i32 = 30;
pub const MC_1_18: i32 = 31;
pub const MC_1_19: i32 = 33;
pub const MC_1_20: i32 = 34;
pub const MC_1_21: i32 = 38;

pub const DIM_OVERWORLD: i32 = 0;
pub const DIM_NETHER: i32 = -1;
pub const DIM_END: i32 = 1;

#[repr(C, align(16))]
pub struct CGenerator {
    _data: [u8; 32768],
}

impl Default for CGenerator {
    fn default() -> Self {
        Self {
            _data: [0u8; 32768],
        }
    }
}

unsafe extern "C" {
    fn setupGenerator(g: *mut CGenerator, mc: i32, flags: u32);
    fn applySeed(g: *mut CGenerator, dim: i32, seed: u64);
    fn getBiomeAt(g: *const CGenerator, scale: i32, x: i32, y: i32, z: i32) -> i32;
    fn biome2str(mc: i32, id: i32) -> *const std::os::raw::c_char;
}

pub struct BiomeGenerator {
    raw: Box<CGenerator>,
    pub version: i32,
    pub seed: u64,
}

impl BiomeGenerator {
    pub fn new(version: i32) -> Self {
        let mut raw = Box::new(CGenerator::default());
        unsafe {
            setupGenerator(&mut *raw, version, 0);
        }
        Self {
            raw,
            version,
            seed: 0,
        }
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
        unsafe {
            applySeed(&mut *self.raw, DIM_OVERWORLD, seed);
        }
    }

    pub fn as_raw(&self) -> *const CGenerator {
        &*self.raw as *const CGenerator
    }

    pub fn as_raw_mut(&mut self) -> *mut CGenerator {
        &mut *self.raw as *mut CGenerator
    }

    pub fn get_biome_at_block(&self, x: i32, y: i32, z: i32) -> i32 {
        unsafe { getBiomeAt(&*self.raw, 1, x, y, z) }
    }

    pub fn get_biome_name(&self, biome_id: i32) -> String {
        unsafe {
            let ptr = biome2str(self.version, biome_id);
            if ptr.is_null() {
                format!("unknown_{biome_id}")
            } else {
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            }
        }
    }
}
