use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockType {
    Bedrock,
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BedrockBlock {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block_type: BlockType,
}

impl BedrockBlock {
    pub fn new(x: i32, y: i32, z: i32, block_type: BlockType) -> Self {
        Self {
            x,
            y,
            z,
            block_type,
        }
    }

    pub fn is_roof(&self) -> bool {
        self.y > 5
    }

    pub fn is_floor(&self) -> bool {
        self.y <= 5
    }
}
