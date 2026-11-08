use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const ZERO: BlockPos = BlockPos { x: 0, y: 0, z: 0 };

    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn offset(self, dx: i32, dy: i32, dz: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            z: self.z + dz,
        }
    }

    pub fn add(self, other: BlockPos) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    pub fn subtract(self, other: BlockPos) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    pub fn min(a: BlockPos, b: BlockPos) -> BlockPos {
        BlockPos::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z))
    }

    pub fn max(a: BlockPos, b: BlockPos) -> BlockPos {
        BlockPos::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z))
    }

    pub fn is_inside_area(&self, min: BlockPos, max: BlockPos) -> bool {
        self.x >= min.x && self.x <= max.x
            && self.y >= min.y && self.y <= max.y
            && self.z >= min.z && self.z <= max.z
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3i {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn to_block_pos(self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    pub fn axis(&self) -> Axis {
        match self {
            Direction::Down | Direction::Up => Axis::Y,
            Direction::North | Direction::South => Axis::Z,
            Direction::West | Direction::East => Axis::X,
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Direction::Down => Direction::Up,
            Direction::Up => Direction::Down,
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Rotation {
    #[default]
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Mirror {
    #[default]
    None,
    LeftRight, // Z axis inversion (North <-> South)
    FrontBack, // X axis inversion (East <-> West)
}

/// Mirrors and then rotates the given position around the origin
pub fn get_transformed_block_pos(pos: BlockPos, mirror: Mirror, rotation: Rotation) -> BlockPos {
    let mut x = pos.x;
    let y = pos.y;
    let mut z = pos.z;
    let mut is_mirrored = true;

    match mirror {
        Mirror::LeftRight => z = -z,
        Mirror::FrontBack => x = -x,
        Mirror::None => is_mirrored = false,
    }

    match rotation {
        Rotation::Clockwise90 => BlockPos::new(-z, y, x),
        Rotation::CounterClockwise90 => BlockPos::new(z, y, -x),
        Rotation::Clockwise180 => BlockPos::new(-x, y, -z),
        Rotation::None => {
            if is_mirrored {
                BlockPos::new(x, y, z)
            } else {
                pos
            }
        }
    }
}

pub fn get_reverse_transformed_block_pos(pos: BlockPos, mirror: Mirror, rotation: Rotation) -> BlockPos {
    let mut x = pos.x;
    let y = pos.y;
    let mut z = pos.z;
    let mut is_rotated = true;
    let tmp = x;

    match rotation {
        Rotation::Clockwise90 => {
            x = z;
            z = -tmp;
        }
        Rotation::CounterClockwise90 => {
            x = -z;
            z = tmp;
        }
        Rotation::Clockwise180 => {
            x = -x;
            z = -z;
        }
        Rotation::None => is_rotated = false,
    }

    match mirror {
        Mirror::LeftRight => z = -z,
        Mirror::FrontBack => x = -x,
        Mirror::None => {
            if !is_rotated {
                return pos;
            }
        }
    }

    BlockPos::new(x, y, z)
}

pub fn get_relative_end_position_from_area_size(size: Vec3i) -> BlockPos {
    let x = if size.x >= 0 { size.x - 1 } else { size.x + 1 };
    let y = if size.y >= 0 { size.y - 1 } else { size.y + 1 };
    let z = if size.z >= 0 { size.z - 1 } else { size.z + 1 };
    BlockPos::new(x, y, z)
}

pub fn get_area_size_from_relative_end_position(pos: BlockPos) -> Vec3i {
    let x = if pos.x >= 0 { pos.x + 1 } else { pos.x - 1 };
    let y = if pos.y >= 0 { pos.y + 1 } else { pos.y - 1 };
    let z = if pos.z >= 0 { pos.z + 1 } else { pos.z - 1 };
    Vec3i::new(x, y, z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transformations() {
        let pos = BlockPos::new(10, 5, 20);
        let transformed = get_transformed_block_pos(pos, Mirror::None, Rotation::Clockwise90);
        assert_eq!(transformed, BlockPos::new(-20, 5, 10));

        let reverse = get_reverse_transformed_block_pos(transformed, Mirror::None, Rotation::Clockwise90);
        assert_eq!(reverse, pos);
    }
}
