use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};

use crate::util::pos::BlockPos;
use super::container::LitematicaBlockStateContainer;
use super::metadata::{SchematicMetadata, DEFAULT_MINECRAFT_DATA_VERSION, DEFAULT_SCHEMATIC_VERSION};
use super::palette::BlockState;

#[derive(Debug, Clone, PartialEq)]
pub struct SubRegion {
    pub position: BlockPos,
    pub size: BlockPos,
    pub container: LitematicaBlockStateContainer,
    pub tile_entities: Vec<fastnbt::Value>,
    pub entities: Vec<fastnbt::Value>,
    pub pending_block_ticks: Vec<fastnbt::Value>,
    pub pending_fluid_ticks: Vec<fastnbt::Value>,
}

impl SubRegion {
    pub fn new(position: BlockPos, size: BlockPos) -> Self {
        let sx = size.x.unsigned_abs() as usize;
        let sy = size.y.unsigned_abs() as usize;
        let sz = size.z.unsigned_abs() as usize;
        let container = LitematicaBlockStateContainer::new(sx, sy, sz);

        Self {
            position,
            size,
            container,
            tile_entities: Vec::new(),
            entities: Vec::new(),
            pending_block_ticks: Vec::new(),
            pending_fluid_ticks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LitematicaSchematic {
    pub version: i32,
    pub sub_version: i32,
    pub minecraft_data_version: i32,
    pub metadata: SchematicMetadata,
    pub regions: HashMap<String, SubRegion>,
}

#[derive(Serialize, Deserialize)]
struct NbtSchematicRoot {
    #[serde(rename = "Version")]
    version: i32,
    #[serde(rename = "SubVersion", default = "default_sub_version")]
    sub_version: i32,
    #[serde(rename = "MinecraftDataVersion")]
    minecraft_data_version: i32,
    #[serde(rename = "Metadata")]
    metadata: SchematicMetadata,
    #[serde(rename = "Regions")]
    regions: HashMap<String, NbtRegion>,
}

fn default_sub_version() -> i32 {
    1
}

#[derive(Serialize, Deserialize)]
struct NbtRegion {
    #[serde(rename = "Position")]
    position: BlockPos,
    #[serde(rename = "Size")]
    size: BlockPos,
    #[serde(rename = "BlockStatePalette")]
    block_state_palette: Vec<BlockState>,
    #[serde(rename = "BlockStates")]
    block_states: Vec<i64>,
    #[serde(rename = "TileEntities", default)]
    tile_entities: Vec<fastnbt::Value>,
    #[serde(rename = "Entities", default)]
    entities: Vec<fastnbt::Value>,
    #[serde(rename = "PendingBlockTicks", default, skip_serializing_if = "Vec::is_empty")]
    pending_block_ticks: Vec<fastnbt::Value>,
    #[serde(rename = "PendingFluidTicks", default, skip_serializing_if = "Vec::is_empty")]
    pending_fluid_ticks: Vec<fastnbt::Value>,
}

impl Default for LitematicaSchematic {
    fn default() -> Self {
        Self {
            version: DEFAULT_SCHEMATIC_VERSION,
            sub_version: 1,
            minecraft_data_version: DEFAULT_MINECRAFT_DATA_VERSION,
            metadata: SchematicMetadata::default(),
            regions: HashMap::new(),
        }
    }
}

impl LitematicaSchematic {
    pub fn new(name: impl Into<String>, author: impl Into<String>) -> Self {
        let mut metadata = SchematicMetadata::default();
        metadata.name = name.into();
        metadata.author = author.into();

        Self {
            version: DEFAULT_SCHEMATIC_VERSION,
            sub_version: 1,
            minecraft_data_version: DEFAULT_MINECRAFT_DATA_VERSION,
            metadata,
            regions: HashMap::new(),
        }
    }

    pub fn add_region(&mut self, name: impl Into<String>, region: SubRegion) {
        self.regions.insert(name.into(), region);
        self.update_metadata_totals();
    }

    pub fn get_region(&self, name: &str) -> Option<&SubRegion> {
        self.regions.get(name)
    }

    pub fn get_region_mut(&mut self, name: &str) -> Option<&mut SubRegion> {
        self.regions.get_mut(name)
    }

    pub fn update_metadata_totals(&mut self) {
        let mut total_blocks = 0;
        let mut total_volume = 0i64;
        let mut min_corner = BlockPos::new(i32::MAX, i32::MAX, i32::MAX);
        let mut max_corner = BlockPos::new(i32::MIN, i32::MIN, i32::MIN);

        for region in self.regions.values() {
            total_blocks += region.container.total_non_air_blocks() as i32;
            total_volume += region.container.total_volume as i64;

            let pos1 = region.position;
            let end_rel_x = if region.size.x >= 0 { region.size.x - 1 } else { region.size.x + 1 };
            let end_rel_y = if region.size.y >= 0 { region.size.y - 1 } else { region.size.y + 1 };
            let end_rel_z = if region.size.z >= 0 { region.size.z - 1 } else { region.size.z + 1 };
            let pos2 = pos1.offset(end_rel_x, end_rel_y, end_rel_z);

            min_corner = BlockPos::min(min_corner, BlockPos::min(pos1, pos2));
            max_corner = BlockPos::max(max_corner, BlockPos::max(pos1, pos2));
        }

        if self.regions.is_empty() {
            min_corner = BlockPos::ZERO;
            max_corner = BlockPos::ZERO;
        }

        let enclosing_size = BlockPos::new(
            (max_corner.x - min_corner.x + 1).max(0),
            (max_corner.y - min_corner.y + 1).max(0),
            (max_corner.z - min_corner.z + 1).max(0),
        );

        self.metadata.region_count = self.regions.len() as i32;
        self.metadata.total_blocks = total_blocks;
        self.metadata.total_volume = total_volume;
        self.metadata.enclosing_size = enclosing_size;
    }

    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let mut decoder = GzDecoder::new(file);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;

        let nbt_root: NbtSchematicRoot = fastnbt::from_bytes(&decompressed)?;

        let mut regions = HashMap::new();
        for (name, nbt_region) in nbt_root.regions {
            let sx = nbt_region.size.x.unsigned_abs() as usize;
            let sy = nbt_region.size.y.unsigned_abs() as usize;
            let sz = nbt_region.size.z.unsigned_abs() as usize;

            let container = LitematicaBlockStateContainer::from_raw(
                sx,
                sy,
                sz,
                nbt_region.block_state_palette,
                nbt_region.block_states,
            );

            regions.insert(
                name,
                SubRegion {
                    position: nbt_region.position,
                    size: nbt_region.size,
                    container,
                    tile_entities: nbt_region.tile_entities,
                    entities: nbt_region.entities,
                    pending_block_ticks: nbt_region.pending_block_ticks,
                    pending_fluid_ticks: nbt_region.pending_fluid_ticks,
                },
            );
        }

        Ok(Self {
            version: nbt_root.version,
            sub_version: nbt_root.sub_version,
            minecraft_data_version: nbt_root.minecraft_data_version,
            metadata: nbt_root.metadata,
            regions,
        })
    }

    pub fn write_to_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        self.update_metadata_totals();

        let mut nbt_regions = HashMap::new();
        for (name, region) in &self.regions {
            let palette = region.container.palette.as_slice().to_vec();
            let block_states = region.container.storage.backing_long_array().to_vec();

            nbt_regions.insert(
                name.clone(),
                NbtRegion {
                    position: region.position,
                    size: region.size,
                    block_state_palette: palette,
                    block_states,
                    tile_entities: region.tile_entities.clone(),
                    entities: region.entities.clone(),
                    pending_block_ticks: region.pending_block_ticks.clone(),
                    pending_fluid_ticks: region.pending_fluid_ticks.clone(),
                },
            );
        }

        let nbt_root = NbtSchematicRoot {
            version: self.version,
            sub_version: self.sub_version,
            minecraft_data_version: self.minecraft_data_version,
            metadata: self.metadata.clone(),
            regions: nbt_regions,
        };

        let encoded = fastnbt::to_bytes(&nbt_root)?;

        let file = File::create(path)?;
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(&encoded)?;
        encoder.finish()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schematic_save_and_load() {
        let mut schem = LitematicaSchematic::new("TestSchematic", "TestAuthor");
        let mut sub = SubRegion::new(BlockPos::new(0, 0, 0), BlockPos::new(3, 3, 3));
        let stone = BlockState::new("minecraft:stone");
        let glass = BlockState::new("minecraft:glass");

        sub.container.set(0, 0, 0, stone.clone());
        sub.container.set(1, 1, 1, glass.clone());
        sub.container.set(2, 2, 2, stone.clone());

        schem.add_region("Main", sub);

        let tmp_path = std::env::temp_dir().join("test_schem.litematic");
        schem.write_to_file(&tmp_path).unwrap();

        let loaded = LitematicaSchematic::read_from_file(&tmp_path).unwrap();
        assert_eq!(loaded.metadata.name, "TestSchematic");
        assert_eq!(loaded.metadata.author, "TestAuthor");
        assert_eq!(loaded.metadata.total_blocks, 3);

        let loaded_sub = loaded.get_region("Main").unwrap();
        assert_eq!(loaded_sub.container.get(0, 0, 0), stone);
        assert_eq!(loaded_sub.container.get(1, 1, 1), glass);
        assert_eq!(loaded_sub.container.get(2, 2, 2), stone);
        assert_eq!(loaded_sub.container.get(0, 1, 0), BlockState::air());

        let _ = std::fs::remove_file(tmp_path);
    }
}
