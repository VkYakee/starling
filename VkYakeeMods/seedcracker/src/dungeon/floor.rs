#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DungeonFloorBlock {
    Cobblestone = 0,
    MossyCobblestone = 1,
    Unknown = 2,
}

#[derive(Debug, Clone)]
pub struct DungeonFloor {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub block_x: i32,
    pub block_y: i32,
    pub block_z: i32,
    pub size_x: i32,
    pub size_z: i32,
    pub calls: Vec<DungeonFloorBlock>,
}

impl DungeonFloor {
    pub fn new(
        chunk_x: i32,
        chunk_z: i32,
        block_x: i32,
        block_y: i32,
        block_z: i32,
        size_x: i32,
        size_z: i32,
        calls: Vec<DungeonFloorBlock>,
    ) -> Self {
        Self {
            chunk_x,
            chunk_z,
            block_x,
            block_y,
            block_z,
            size_x,
            size_z,
            calls,
        }
    }

    pub fn from_string(pattern: &str) -> Vec<DungeonFloorBlock> {
        pattern
            .chars()
            .filter_map(|c| match c {
                '0' | 'c' | 'C' => Some(DungeonFloorBlock::Cobblestone),
                '1' | 'm' | 'M' => Some(DungeonFloorBlock::MossyCobblestone),
                '?' | '.' | '2' => Some(DungeonFloorBlock::Unknown),
                _ => None,
            })
            .collect()
    }
}
