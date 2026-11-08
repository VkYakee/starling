use std::collections::HashMap;
use crate::util::pos::Vec3i;
use super::bit_array::LitematicaBitArray;
use super::palette::{BlockState, BlockStatePalette};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LitematicaBlockStateContainer {
    pub size_x: usize,
    pub size_y: usize,
    pub size_z: usize,
    pub size_layer: usize,
    pub total_volume: usize,
    pub storage: LitematicaBitArray,
    pub palette: BlockStatePalette,
}

impl LitematicaBlockStateContainer {
    pub fn new(size_x: usize, size_y: usize, size_z: usize) -> Self {
        let size_x = size_x.max(1);
        let size_y = size_y.max(1);
        let size_z = size_z.max(1);
        let size_layer = size_x * size_z;
        let total_volume = size_x * size_y * size_z;

        let palette = BlockStatePalette::new();
        // Initial 2 bits per entry
        let storage = LitematicaBitArray::new(2, total_volume, None);

        Self {
            size_x,
            size_y,
            size_z,
            size_layer,
            total_volume,
            storage,
            palette,
        }
    }

    pub fn from_raw(
        size_x: usize,
        size_y: usize,
        size_z: usize,
        palette_states: Vec<BlockState>,
        raw_longs: Vec<i64>,
    ) -> Self {
        let size_x = size_x.max(1);
        let size_y = size_y.max(1);
        let size_z = size_z.max(1);
        let size_layer = size_x * size_z;
        let total_volume = size_x * size_y * size_z;

        let palette_len = palette_states.len().max(2);
        let bits = Self::calculate_bits_for_palette_size(palette_len);

        let palette = BlockStatePalette::from_states(palette_states);
        let storage = LitematicaBitArray::new(bits, total_volume, Some(raw_longs));

        Self {
            size_x,
            size_y,
            size_z,
            size_layer,
            total_volume,
            storage,
            palette,
        }
    }

    pub fn calculate_bits_for_palette_size(palette_len: usize) -> usize {
        let mut bits = 2;
        while (1usize << bits) < palette_len && bits < 32 {
            bits += 1;
        }
        bits
    }

    pub fn size(&self) -> Vec3i {
        Vec3i::new(self.size_x as i32, self.size_y as i32, self.size_z as i32)
    }

    pub fn get_index(&self, x: usize, y: usize, z: usize) -> usize {
        (y * self.size_layer) + z * self.size_x + x
    }

    pub fn get_coords(&self, index: usize) -> (usize, usize, usize) {
        let y = index / self.size_layer;
        let rem = index % self.size_layer;
        let z = rem / self.size_x;
        let x = rem % self.size_x;
        (x, y, z)
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockState {
        if x >= self.size_x || y >= self.size_y || z >= self.size_z {
            return BlockState::air();
        }
        let idx = self.get_index(x, y, z);
        let state_id = self.storage.get_at(idx);
        self.palette.get_state_or_air(state_id)
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, state: BlockState) {
        if x >= self.size_x || y >= self.size_y || z >= self.size_z {
            return;
        }

        let state_id = self.palette.id_for(&state);
        self.ensure_capacity_for_id(state_id);

        let idx = self.get_index(x, y, z);
        self.storage.set_at(idx, state_id);
    }

    fn ensure_capacity_for_id(&mut self, id: u32) {
        let needed_bits = Self::calculate_bits_for_palette_size((id + 1) as usize);
        if needed_bits > self.storage.bits_per_entry() {
            let mut new_storage = LitematicaBitArray::new(needed_bits, self.total_volume, None);
            for i in 0..self.total_volume {
                let val = self.storage.get_at(i);
                new_storage.set_at(i, val);
            }
            self.storage = new_storage;
        }
    }

    pub fn count_blocks(&self) -> HashMap<BlockState, usize> {
        let mut map = HashMap::new();
        for i in 0..self.total_volume {
            let state_id = self.storage.get_at(i);
            let state = self.palette.get_state_or_air(state_id);
            if !state.is_air() {
                *map.entry(state).or_insert(0) += 1;
            }
        }
        map
    }

    pub fn total_non_air_blocks(&self) -> usize {
        let mut count = 0;
        for i in 0..self.total_volume {
            let state_id = self.storage.get_at(i);
            let state = self.palette.get_state_or_air(state_id);
            if !state.is_air() {
                count += 1;
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_get_set() {
        let mut container = LitematicaBlockStateContainer::new(4, 4, 4);
        let stone = BlockState::new("minecraft:stone");
        let dirt = BlockState::new("minecraft:dirt");

        container.set(1, 2, 3, stone.clone());
        container.set(0, 0, 0, dirt.clone());

        assert_eq!(container.get(1, 2, 3), stone);
        assert_eq!(container.get(0, 0, 0), dirt);
        assert_eq!(container.get(2, 2, 2), BlockState::air());
        assert_eq!(container.total_non_air_blocks(), 2);
    }
}
