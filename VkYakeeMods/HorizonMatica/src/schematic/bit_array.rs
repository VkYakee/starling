use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LitematicaBitArray {
    bits_per_entry: usize,
    array_size: usize,
    max_entry_value: u64,
    long_array: Vec<i64>,
}

impl LitematicaBitArray {
    pub fn new(bits_per_entry: usize, array_size: usize, existing_data: Option<Vec<i64>>) -> Self {
        assert!(bits_per_entry >= 1 && bits_per_entry <= 32, "bits_per_entry must be between 1 and 32");
        let max_entry_value = (1u64 << bits_per_entry) - 1;
        let total_bits = array_size * bits_per_entry;
        let required_longs = (total_bits + 63) / 64;

        let long_array = match existing_data {
            Some(vec) => vec,
            None => vec![0i64; required_longs],
        };

        Self {
            bits_per_entry,
            array_size,
            max_entry_value,
            long_array,
        }
    }

    pub fn bits_per_entry(&self) -> usize {
        self.bits_per_entry
    }

    pub fn size(&self) -> usize {
        self.array_size
    }

    pub fn backing_long_array(&self) -> &[i64] {
        &self.long_array
    }

    pub fn into_backing_long_array(self) -> Vec<i64> {
        self.long_array
    }

    pub fn get_at(&self, index: usize) -> u32 {
        if index >= self.array_size {
            return 0;
        }

        let start_offset = index * self.bits_per_entry;
        let start_arr_idx = start_offset >> 6;
        let end_arr_idx = ((index + 1) * self.bits_per_entry - 1) >> 6;
        let start_bit_offset = start_offset & 0x3F;

        if start_arr_idx >= self.long_array.len() {
            return 0;
        }

        let start_val = self.long_array[start_arr_idx] as u64;

        if start_arr_idx == end_arr_idx {
            ((start_val >> start_bit_offset) & self.max_entry_value) as u32
        } else {
            let end_val = if end_arr_idx < self.long_array.len() {
                self.long_array[end_arr_idx] as u64
            } else {
                0
            };
            let end_offset = 64 - start_bit_offset;
            (((start_val >> start_bit_offset) | (end_val << end_offset)) & self.max_entry_value) as u32
        }
    }

    pub fn set_at(&mut self, index: usize, value: u32) {
        if index >= self.array_size {
            return;
        }

        let val = (value as u64) & self.max_entry_value;
        let start_offset = index * self.bits_per_entry;
        let start_arr_idx = start_offset >> 6;
        let end_arr_idx = ((index + 1) * self.bits_per_entry - 1) >> 6;
        let start_bit_offset = start_offset & 0x3F;

        if start_arr_idx >= self.long_array.len() {
            return;
        }

        let mut start_val = self.long_array[start_arr_idx] as u64;
        start_val = (start_val & !(self.max_entry_value << start_bit_offset)) | (val << start_bit_offset);
        self.long_array[start_arr_idx] = start_val as i64;

        if start_arr_idx != end_arr_idx && end_arr_idx < self.long_array.len() {
            let end_offset = 64 - start_bit_offset;
            let j1 = self.bits_per_entry - end_offset;
            let mut end_val = self.long_array[end_arr_idx] as u64;
            end_val = ((end_val >> j1) << j1) | (val >> end_offset);
            self.long_array[end_arr_idx] = end_val as i64;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_array_roundtrip() {
        let size = 1000;
        for bits in 1..=16 {
            let mut arr = LitematicaBitArray::new(bits, size, None);
            let mask = (1u32 << bits) - 1;
            for i in 0..size {
                let v = ((i * 37 + 13) as u32) & mask;
                arr.set_at(i, v);
            }
            for i in 0..size {
                let v = ((i * 37 + 13) as u32) & mask;
                assert_eq!(arr.get_at(i), v, "Mismatch at index {} with bits {}", i, bits);
            }
        }
    }
}
