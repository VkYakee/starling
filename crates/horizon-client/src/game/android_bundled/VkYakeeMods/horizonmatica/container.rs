// Copyright (c) 2026 HorizonMatica Authors / Starling Android
//! Litematica bit-packed 3D volume container and dynamic blockstate palette.
//! Matches fi.dy.masa.litematica.schematic.container.LitematicaBitArray and
//! LitematicaBlockStateContainer with exact spanning 64-bit boundary packing.

use std::collections::BTreeMap;

/// Single block state entry stored in the palette (block name + optional properties).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SchematicBlockState {
    pub name: String,
    pub properties: BTreeMap<String, String>,
}

impl SchematicBlockState {
    pub fn air() -> Self {
        Self {
            name: "minecraft:air".to_string(),
            properties: BTreeMap::new(),
        }
    }

    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            properties: BTreeMap::new(),
        }
    }

    pub fn is_air(&self) -> bool {
        self.name == "minecraft:air" || self.name == "air" || self.name == "minecraft:cave_air" || self.name == "minecraft:void_air"
    }

    pub fn short_name(&self) -> &str {
        self.name.strip_prefix("minecraft:").unwrap_or(&self.name)
    }

    pub fn formatted_properties(&self) -> String {
        if self.properties.is_empty() {
            String::new()
        } else {
            let mut parts = Vec::new();
            for (k, v) in &self.properties {
                parts.push(format!("{k}={v}"));
            }
            format!("[{}]", parts.join(","))
        }
    }
}

/// Bit-packed storage array that continuously spans 64-bit boundaries.
#[derive(Clone, Debug)]
pub struct LitematicaBitArray {
    pub bits_per_entry: usize,
    pub array_size: usize,
    pub long_array: Vec<i64>,
    max_entry_value: u64,
}

impl LitematicaBitArray {
    pub fn new(bits_per_entry: usize, array_size: usize, backing_data: Option<Vec<i64>>) -> Self {
        let bits = bits_per_entry.clamp(1, 32);
        let max_val = if bits == 64 { u64::MAX } else { (1u64 << bits) - 1 };
        let long_count = if array_size == 0 {
            0
        } else {
            let total_bits = (array_size as u64) * (bits as u64);
            ((total_bits + 63) / 64) as usize
        };

        let long_array = match backing_data {
            Some(vec) => {
                if vec.len() >= long_count {
                    vec
                } else {
                    let mut extended = vec;
                    extended.resize(long_count, 0);
                    extended
                }
            }
            None => vec![0i64; long_count],
        };

        Self {
            bits_per_entry: bits,
            array_size,
            long_array,
            max_entry_value: max_val,
        }
    }

    #[inline]
    pub fn get_at(&self, index: usize) -> u32 {
        if index >= self.array_size || self.long_array.is_empty() {
            return 0;
        }
        let start_offset = (index as u64) * (self.bits_per_entry as u64);
        let start_arr = (start_offset >> 6) as usize;
        let end_arr = (((index as u64 + 1) * (self.bits_per_entry as u64) - 1) >> 6) as usize;
        let start_bit = (start_offset & 0x3F) as usize;

        if start_arr >= self.long_array.len() {
            return 0;
        }

        let u_start = self.long_array[start_arr] as u64;

        if start_arr == end_arr {
            ((u_start >> start_bit) & self.max_entry_value) as u32
        } else {
            let end_offset = 64 - start_bit;
            let u_end = if end_arr < self.long_array.len() {
                self.long_array[end_arr] as u64
            } else {
                0
            };
            (((u_start >> start_bit) | (u_end << end_offset)) & self.max_entry_value) as u32
        }
    }

    pub fn set_at(&mut self, index: usize, value: u32) {
        if index >= self.array_size || self.long_array.is_empty() {
            return;
        }
        let v = (value as u64) & self.max_entry_value;
        let start_offset = (index as u64) * (self.bits_per_entry as u64);
        let start_arr = (start_offset >> 6) as usize;
        let end_arr = (((index as u64 + 1) * (self.bits_per_entry as u64) - 1) >> 6) as usize;
        let start_bit = (start_offset & 0x3F) as usize;

        if start_arr >= self.long_array.len() {
            return;
        }

        let u_start = self.long_array[start_arr] as u64;
        let mask1 = !(self.max_entry_value << start_bit);
        self.long_array[start_arr] = ((u_start & mask1) | (v << start_bit)) as i64;

        if start_arr != end_arr && end_arr < self.long_array.len() {
            let end_offset = 64 - start_bit;
            let j1 = self.bits_per_entry - end_offset;
            let u_end = self.long_array[end_arr] as u64;
            let mask2 = !((1u64 << j1) - 1);
            self.long_array[end_arr] = ((u_end & mask2) | (v >> end_offset)) as i64;
        }
    }
}

/// 3D discrete block volume containing palette and bitpacked indices.
#[derive(Clone, Debug)]
pub struct LitematicaBlockStateContainer {
    pub size_x: i32,
    pub size_y: i32,
    pub size_z: i32,
    pub palette: Vec<SchematicBlockState>,
    pub storage: LitematicaBitArray,
}

impl LitematicaBlockStateContainer {
    pub fn new(size_x: i32, size_y: i32, size_z: i32) -> Self {
        let sx = size_x.abs().max(1);
        let sy = size_y.abs().max(1);
        let sz = size_z.abs().max(1);
        let total_volume = (sx as usize) * (sy as usize) * (sz as usize);

        let mut palette = Vec::new();
        palette.push(SchematicBlockState::air());
        let bits = 2;
        let storage = LitematicaBitArray::new(bits, total_volume, None);

        Self {
            size_x: sx,
            size_y: sy,
            size_z: sz,
            palette,
            storage,
        }
    }

    pub fn from_existing(
        size_x: i32,
        size_y: i32,
        size_z: i32,
        palette: Vec<SchematicBlockState>,
        backing_longs: Vec<i64>,
    ) -> Self {
        let sx = size_x.abs().max(1);
        let sy = size_y.abs().max(1);
        let sz = size_z.abs().max(1);
        let total_volume = (sx as usize) * (sy as usize) * (sz as usize);

        let pal_len = palette.len().max(1);
        let bits = if pal_len <= 1 {
            2
        } else {
            std::cmp::max(2, 32 - ((pal_len as u32) - 1).leading_zeros() as usize)
        };

        let storage = LitematicaBitArray::new(bits, total_volume, Some(backing_longs));

        Self {
            size_x: sx,
            size_y: sy,
            size_z: sz,
            palette,
            storage,
        }
    }

    #[inline]
    pub fn get_index(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        if x < 0 || x >= self.size_x || y < 0 || y >= self.size_y || z < 0 || z >= self.size_z {
            return None;
        }
        let layer_size = (self.size_x as usize) * (self.size_z as usize);
        let idx = (y as usize) * layer_size + (z as usize) * (self.size_x as usize) + (x as usize);
        Some(idx)
    }

    pub fn get(&self, x: i32, y: i32, z: i32) -> Option<&SchematicBlockState> {
        let idx = self.get_index(x, y, z)?;
        let palette_id = self.storage.get_at(idx) as usize;
        self.palette.get(palette_id).or_else(|| self.palette.first())
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, state: SchematicBlockState) {
        let Some(idx) = self.get_index(x, y, z) else {
            return;
        };

        let palette_id = if let Some(found) = self.palette.iter().position(|s| s == &state) {
            found
        } else {
            self.palette.push(state);
            let new_len = self.palette.len();
            let needed_bits = std::cmp::max(2, 32 - ((new_len as u32) - 1).leading_zeros() as usize);
            if needed_bits > self.storage.bits_per_entry {
                self.reallocate_storage(needed_bits);
            }
            new_len - 1
        };

        self.storage.set_at(idx, palette_id as u32);
    }

    fn reallocate_storage(&mut self, new_bits: usize) {
        let mut new_storage = LitematicaBitArray::new(new_bits, self.storage.array_size, None);
        for i in 0..self.storage.array_size {
            let val = self.storage.get_at(i);
            new_storage.set_at(i, val);
        }
        self.storage = new_storage;
    }

    pub fn total_volume(&self) -> usize {
        (self.size_x as usize) * (self.size_y as usize) * (self.size_z as usize)
    }
}
