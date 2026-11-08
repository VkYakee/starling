use serde::{Deserialize, Serialize};
use crate::util::pos::BlockPos;

pub const DEFAULT_SCHEMATIC_VERSION: i32 = 7;
pub const DEFAULT_MINECRAFT_DATA_VERSION: i32 = 3700;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchematicMetadata {
    #[serde(rename = "Name", default = "default_name")]
    pub name: String,
    #[serde(rename = "Author", default = "default_author")]
    pub author: String,
    #[serde(rename = "Description", default)]
    pub description: String,
    #[serde(rename = "TimeCreated", default)]
    pub time_created: i64,
    #[serde(rename = "TimeModified", default)]
    pub time_modified: i64,
    #[serde(rename = "RegionCount", default)]
    pub region_count: i32,
    #[serde(rename = "TotalBlocks", default)]
    pub total_blocks: i32,
    #[serde(rename = "TotalVolume", default)]
    pub total_volume: i64,
    #[serde(rename = "EnclosingSize", default)]
    pub enclosing_size: BlockPos,
    #[serde(rename = "PreviewImageData", default, skip_serializing_if = "Option::is_none")]
    pub preview_image_data: Option<Vec<i32>>,
}

fn default_name() -> String {
    "Unnamed".to_string()
}

fn default_author() -> String {
    "Rustmatica".to_string()
}

impl Default for SchematicMetadata {
    fn default() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        Self {
            name: default_name(),
            author: default_author(),
            description: String::new(),
            time_created: now,
            time_modified: now,
            region_count: 1,
            total_blocks: 0,
            total_volume: 0,
            enclosing_size: BlockPos::ZERO,
            preview_image_data: None,
        }
    }
}
