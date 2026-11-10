//! Position and transformation utilities

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub};

/// 3D integer position (block coordinates)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const ZERO: Self = BlockPos { x: 0, y: 0, z: 0 };

    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn offset(self, dx: i32, dy: i32, dz: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            z: self.z + dz,
        }
    }

    pub fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    pub fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    pub fn min(self, other: Self) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
        }
    }

    pub fn max(self, other: Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
        }
    }

    pub fn distance_to(&self, other: &BlockPos) -> f64 {
        let dx = (self.x - other.x) as f64;
        let dy = (self.y - other.y) as f64;
        let dz = (self.z - other.z) as f64;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn manhattan_distance(&self, other: &BlockPos) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()
    }

    pub fn within_bounds(&self, min: BlockPos, max: BlockPos) -> bool {
        self.x >= min.x && self.x <= max.x &&
        self.y >= min.y && self.y <= max.y &&
        self.z >= min.z && self.z <= max.z
    }
}

impl Add for BlockPos {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        self.add(other)
    }
}

impl Sub for BlockPos {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        self.sub(other)
    }
}

impl fmt::Display for BlockPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

/// 3D integer vector (for sizes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3i {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

impl From<BlockPos> for Vec3i {
    fn from(pos: BlockPos) -> Self {
        Vec3i::new(pos.x, pos.y, pos.z)
    }
}

/// Rotation in 90-degree increments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    CounterClockwise90,
}

impl Rotation {
    pub fn rotate_cw(self) -> Self {
        match self {
            Rotation::None => Rotation::Clockwise90,
            Rotation::Clockwise90 => Rotation::Clockwise180,
            Rotation::Clockwise180 => Rotation::CounterClockwise90,
            Rotation::CounterClockwise90 => Rotation::None,
        }
    }

    pub fn rotate_ccw(self) -> Self {
        match self {
            Rotation::None => Rotation::CounterClockwise90,
            Rotation::CounterClockwise90 => Rotation::Clockwise180,
            Rotation::Clockwise180 => Rotation::Clockwise90,
            Rotation::Clockwise90 => Rotation::None,
        }
    }

    pub fn apply(self, x: i32, z: i32, size_x: i32, size_z: i32) -> (i32, i32) {
        match self {
            Rotation::None => (x, z),
            Rotation::Clockwise90 => (z, size_x - 1 - x),
            Rotation::Clockwise180 => (size_x - 1 - x, size_z - 1 - z),
            Rotation::CounterClockwise90 => (size_z - 1 - z, x),
        }
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Rotation::None
    }
}

/// Mirror transformation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mirror {
    None,
    LeftRight,  // Mirror across X axis (front-back)
    FrontBack,  // Mirror across Z axis (left-right)
}

impl Mirror {
    pub fn cycle(self) -> Self {
        match self {
            Mirror::None => Mirror::FrontBack,
            Mirror::FrontBack => Mirror::LeftRight,
            Mirror::LeftRight => Mirror::None,
        }
    }

    pub fn apply(self, x: i32, z: i32, size_x: i32, size_z: i32) -> (i32, i32) {
        match self {
            Mirror::None => (x, z),
            Mirror::LeftRight => (size_x - 1 - x, z),
            Mirror::FrontBack => (x, size_z - 1 - z),
        }
    }
}

impl Default for Mirror {
    fn default() -> Self {
        Mirror::None
    }
}

/// Cardinal axes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub fn all() -> [Axis; 3] {
        [Axis::X, Axis::Y, Axis::Z]
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "x" => Some(Axis::X),
            "y" => Some(Axis::Y),
            "z" => Some(Axis::Z),
            _ => None,
        }
    }
}

impl Default for Axis {
    fn default() -> Self {
        Axis::Y
    }
}

/// Get transformed block position applying rotation and mirror
pub fn get_transformed_block_pos(
    pos: BlockPos,
    mirror: Mirror,
    rotation: Rotation,
) -> BlockPos {
    let (rx, rz) = rotation.apply(pos.x, pos.z, 16, 16); // placeholder sizes
    let (mx, mz) = mirror.apply(rx, rz, 16, 16);
    BlockPos::new(mx, pos.y, mz)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_pos_operations() {
        let p1 = BlockPos::new(10, 64, 20);
        let p2 = BlockPos::new(5, 60, 15);
        assert_eq!(p1.add(p2), BlockPos::new(15, 124, 35));
        assert_eq!(p1.sub(p2), BlockPos::new(5, 4, 5));
    }

    #[test]
    fn test_rotation() {
        let r = Rotation::None;
        assert_eq!(r.rotate_cw(), Rotation::Clockwise90);
        assert_eq!(r.rotate_cw().rotate_cw(), Rotation::Clockwise180);
        assert_eq!(r.rotate_cw().rotate_cw().rotate_cw(), Rotation::CounterClockwise90);
        assert_eq!(r.rotate_cw().rotate_cw().rotate_cw().rotate_cw(), Rotation::None);
    }

    #[test]
    fn test_mirror() {
        let m = Mirror::None;
        assert_eq!(m.cycle(), Mirror::FrontBack);
        assert_eq!(m.cycle().cycle(), Mirror::LeftRight);
        assert_eq!(m.cycle().cycle().cycle(), Mirror::None);
    }
}
