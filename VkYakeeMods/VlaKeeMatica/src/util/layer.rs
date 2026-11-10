//! Layer slicing and range utilities

use serde::{Deserialize, Serialize};
use crate::util::pos::Axis;

/// Layer rendering mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerMode {
    /// Show all layers
    All,
    /// Show only a single layer
    SingleLayer,
    /// Show a range of layers
    LayerRange,
}

impl Default for LayerMode {
    fn default() -> Self {
        LayerMode::All
    }
}

/// Layer range for slicing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerRange {
    pub min: i32,
    pub max: i32,
    pub axis: Axis,
}

impl Default for LayerRange {
    fn default() -> Self {
        Self {
            min: i32::MIN,
            max: i32::MAX,
            axis: Axis::Y,
        }
    }
}

impl LayerRange {
    pub fn new(min: i32, max: i32, axis: Axis) -> Self {
        Self { min, max, axis }
    }

    pub fn single_layer(y: i32) -> Self {
        Self {
            min: y,
            max: y,
            axis: Axis::Y,
        }
    }

    pub fn is_position_within_layer(&self, pos: crate::util::pos::BlockPos) -> bool {
        let coord = match self.axis {
            Axis::X => pos.x,
            Axis::Y => pos.y,
            Axis::Z => pos.z,
        };
        coord >= self.min && coord <= self.max
    }

    pub fn expand_to_include(&mut self, pos: crate::util::pos::BlockPos) {
        let coord = match self.axis {
            Axis::X => pos.x,
            Axis::Y => pos.y,
            Axis::Z => pos.z,
        };
        self.min = self.min.min(coord);
        self.max = self.max.max(coord);
    }

    pub fn size(&self) -> i32 {
        self.max - self.min + 1
    }

    pub fn contains(&self, coord: i32) -> bool {
        coord >= self.min && coord <= self.max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_range() {
        let range = LayerRange::new(60, 70, Axis::Y);
        assert!(range.contains(65));
        assert!(!range.contains(55));
        assert!(!range.contains(75));
        assert_eq!(range.size(), 11);
    }

    #[test]
    fn test_single_layer() {
        let range = LayerRange::single_layer(64);
        assert_eq!(range.min, 64);
        assert_eq!(range.max, 64);
        assert!(range.contains(64));
        assert!(!range.contains(65));
    }
}
