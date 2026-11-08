use serde::{Deserialize, Serialize};
use super::pos::{Axis, BlockPos};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LayerMode {
    #[default]
    All,
    SingleLayer,
    LayerRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerRange {
    pub mode: LayerMode,
    pub axis: Axis,
    pub current_layer: i32,
    pub min_layer: i32,
    pub max_layer: i32,
}

impl Default for LayerRange {
    fn default() -> Self {
        Self {
            mode: LayerMode::All,
            axis: Axis::Y,
            current_layer: 0,
            min_layer: 0,
            max_layer: 0,
        }
    }
}

impl LayerRange {
    pub fn new(axis: Axis) -> Self {
        Self {
            mode: LayerMode::All,
            axis,
            current_layer: 0,
            min_layer: 0,
            max_layer: 0,
        }
    }

    pub fn is_position_within_layer(&self, pos: BlockPos) -> bool {
        match self.mode {
            LayerMode::All => true,
            LayerMode::SingleLayer => {
                let coord = match self.axis {
                    Axis::X => pos.x,
                    Axis::Y => pos.y,
                    Axis::Z => pos.z,
                };
                coord == self.current_layer
            }
            LayerMode::LayerRange => {
                let coord = match self.axis {
                    Axis::X => pos.x,
                    Axis::Y => pos.y,
                    Axis::Z => pos.z,
                };
                let min = self.min_layer.min(self.max_layer);
                let max = self.min_layer.max(self.max_layer);
                coord >= min && coord <= max
            }
        }
    }

    pub fn next_mode(&mut self) {
        self.mode = match self.mode {
            LayerMode::All => LayerMode::SingleLayer,
            LayerMode::SingleLayer => LayerMode::LayerRange,
            LayerMode::LayerRange => LayerMode::All,
        };
    }

    pub fn prev_mode(&mut self) {
        self.mode = match self.mode {
            LayerMode::All => LayerMode::LayerRange,
            LayerMode::SingleLayer => LayerMode::All,
            LayerMode::LayerRange => LayerMode::SingleLayer,
        };
    }

    pub fn move_layer(&mut self, delta: i32) {
        self.current_layer += delta;
        self.min_layer += delta;
        self.max_layer += delta;
    }
}
