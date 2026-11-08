use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use flate2::read::GzDecoder;
use serde::Deserialize;

use crate::util::pos::BlockPos;
use super::litematic::{LitematicaSchematic, SubRegion};
use super::palette::BlockState;

#[derive(Deserialize)]
struct VanillaStructure {
    size: Vec<i32>,
    #[serde(default)]
    palette: Vec<BlockState>,
    #[serde(default)]
    blocks: Vec<VanillaBlock>,
}

#[derive(Deserialize)]
struct VanillaBlock {
    pos: Vec<i32>,
    state: usize,
}

#[derive(Deserialize)]
struct SpongeSchematic {
    #[serde(rename = "Width")]
    width: i16,
    #[serde(rename = "Height")]
    height: i16,
    #[serde(rename = "Length")]
    length: i16,
    #[serde(rename = "Palette", default)]
    palette: HashMap<String, i32>,
    #[serde(rename = "BlockData", default)]
    block_data: Vec<u8>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct LegacySchematic {
    #[serde(rename = "Width")]
    width: i16,
    #[serde(rename = "Height")]
    height: i16,
    #[serde(rename = "Length")]
    length: i16,
    #[serde(rename = "Blocks", default)]
    blocks: Vec<u8>,
    #[serde(rename = "Data", default)]
    data: Vec<u8>,
}

pub struct SchematicConverter;

impl SchematicConverter {
    /// Unified schematic loader: detects format by file extension or content.
    /// Supports .litematic, .schem (WorldEdit Sponge), .schematic (Legacy MCEdit), and .nbt (Vanilla).
    pub fn load_any_schematic<P: AsRef<Path>>(path: P) -> Result<LitematicaSchematic, Box<dyn std::error::Error>> {
        let p = path.as_ref();
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

        match ext.as_str() {
            "litematic" => LitematicaSchematic::read_from_file(p),
            "schem" => Self::from_sponge_schem_file(p),
            "schematic" => Self::from_legacy_schematic_file(p),
            "nbt" => Self::from_vanilla_structure_file(p),
            _ => {
                // Try .litematic first, then .schem, then .nbt
                if let Ok(l) = LitematicaSchematic::read_from_file(p) {
                    return Ok(l);
                }
                if let Ok(s) = Self::from_sponge_schem_file(p) {
                    return Ok(s);
                }
                Self::from_vanilla_structure_file(p)
            }
        }
    }

    /// Imports a vanilla Minecraft structure template (.nbt) and converts it to a LitematicaSchematic
    pub fn from_vanilla_structure_file<P: AsRef<Path>>(path: P) -> Result<LitematicaSchematic, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let mut decoder = GzDecoder::new(file);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;

        let structure: VanillaStructure = fastnbt::from_bytes(&decompressed)?;

        let sx = structure.size.first().copied().unwrap_or(1).unsigned_abs() as usize;
        let sy = structure.size.get(1).copied().unwrap_or(1).unsigned_abs() as usize;
        let sz = structure.size.get(2).copied().unwrap_or(1).unsigned_abs() as usize;

        let mut schematic = LitematicaSchematic::new("ImportedStructure", "HorizonMatica");
        let mut sub_region = SubRegion::new(
            BlockPos::ZERO,
            BlockPos::new(sx as i32, sy as i32, sz as i32),
        );

        for block in structure.blocks {
            if block.pos.len() >= 3 {
                let x = block.pos[0] as usize;
                let y = block.pos[1] as usize;
                let z = block.pos[2] as usize;

                if let Some(state) = structure.palette.get(block.state) {
                    sub_region.container.set(x, y, z, state.clone());
                }
            }
        }

        schematic.add_region("Main", sub_region);
        Ok(schematic)
    }

    /// Imports a Sponge (.schem) WorldEdit schematic
    pub fn from_sponge_schem_file<P: AsRef<Path>>(path: P) -> Result<LitematicaSchematic, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let mut decoder = GzDecoder::new(file);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;

        let schem: SpongeSchematic = fastnbt::from_bytes(&decompressed)?;
        let sx = schem.width.max(1) as usize;
        let sy = schem.height.max(1) as usize;
        let sz = schem.length.max(1) as usize;

        let mut schematic = LitematicaSchematic::new("ImportedSponge", "HorizonMatica");
        let mut sub_region = SubRegion::new(
            BlockPos::ZERO,
            BlockPos::new(sx as i32, sy as i32, sz as i32),
        );

        // Invert palette mapping from ID to BlockState
        let mut id_to_state: HashMap<i32, BlockState> = HashMap::new();
        for (state_str, id) in &schem.palette {
            id_to_state.insert(*id, BlockState::from_string_repr(state_str));
        }

        // Decode varint/byte BlockData
        let mut idx = 0;
        let mut block_idx = 0;
        let total_blocks = sx * sy * sz;

        while idx < schem.block_data.len() && block_idx < total_blocks {
            let mut val: i32 = 0;
            let mut shift = 0;
            loop {
                if idx >= schem.block_data.len() { break; }
                let b = schem.block_data[idx];
                idx += 1;
                val |= ((b & 0x7F) as i32) << shift;
                if (b & 0x80) == 0 { break; }
                shift += 7;
            }

            let y = block_idx / (sx * sz);
            let rem = block_idx % (sx * sz);
            let z = rem / sx;
            let x = rem % sx;

            if let Some(state) = id_to_state.get(&val) {
                if !state.is_air() {
                    sub_region.container.set(x, y, z, state.clone());
                }
            }
            block_idx += 1;
        }

        schematic.add_region("Main", sub_region);
        Ok(schematic)
    }

    /// Imports a legacy MCEdit (.schematic) file
    pub fn from_legacy_schematic_file<P: AsRef<Path>>(path: P) -> Result<LitematicaSchematic, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let mut decoder = GzDecoder::new(file);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;

        let schem: LegacySchematic = fastnbt::from_bytes(&decompressed)?;
        let sx = schem.width.max(1) as usize;
        let sy = schem.height.max(1) as usize;
        let sz = schem.length.max(1) as usize;

        let mut schematic = LitematicaSchematic::new("ImportedLegacy", "HorizonMatica");
        let mut sub_region = SubRegion::new(
            BlockPos::ZERO,
            BlockPos::new(sx as i32, sy as i32, sz as i32),
        );

        for y in 0..sy {
            for z in 0..sz {
                for x in 0..sx {
                    let idx = (y * sz + z) * sx + x;
                    if idx < schem.blocks.len() {
                        let id = schem.blocks[idx];
                        if id != 0 {
                            let name = match id {
                                1 => "minecraft:stone",
                                2 => "minecraft:grass_block",
                                3 => "minecraft:dirt",
                                4 => "minecraft:cobblestone",
                                5 => "minecraft:oak_planks",
                                12 => "minecraft:sand",
                                13 => "minecraft:gravel",
                                14 => "minecraft:gold_ore",
                                15 => "minecraft:iron_ore",
                                16 => "minecraft:coal_ore",
                                17 => "minecraft:oak_log",
                                18 => "minecraft:oak_leaves",
                                20 => "minecraft:glass",
                                35 => "minecraft:white_wool",
                                41 => "minecraft:gold_block",
                                42 => "minecraft:iron_block",
                                45 => "minecraft:bricks",
                                46 => "minecraft:tnt",
                                47 => "minecraft:bookshelf",
                                48 => "minecraft:mossy_cobblestone",
                                49 => "minecraft:obsidian",
                                54 => "minecraft:chest",
                                56 => "minecraft:diamond_ore",
                                57 => "minecraft:diamond_block",
                                98 => "minecraft:stone_bricks",
                                _ => "minecraft:stone",
                            };
                            sub_region.container.set(x, y, z, BlockState::new(name));
                        }
                    }
                }
            }
        }

        schematic.add_region("Main", sub_region);
        Ok(schematic)
    }
}
