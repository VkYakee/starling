// Copyright (c) 2026 HorizonMatica Authors / Starling Android
//! Universal schematic parser and serializer for .litematic, Sponge .schem,
//! and Vanilla structure .nbt formats.

use std::collections::BTreeMap;
use std::io::Cursor;
use horizon_world::compression::Compression;
use horizon_world::nbt::{self, Compound, JavaString, Limits, NamedTag, Tag};

use super::container::{LitematicaBlockStateContainer, SchematicBlockState};

/// A named subregion inside a schematic.
#[derive(Clone, Debug)]
pub struct LoadedSubRegion {
    pub name: String,
    pub pos: [i32; 3],
    pub size: [i32; 3],
    pub container: LitematicaBlockStateContainer,
}

/// In-memory representation of any loaded schematic.
#[derive(Clone, Debug)]
pub struct LoadedSchematic {
    pub name: String,
    pub author: String,
    pub description: String,
    pub time_created: u64,
    pub time_modified: u64,
    pub total_volume: u64,
    pub total_blocks: u64,
    pub enclosing_size: [i32; 3],
    pub regions: Vec<LoadedSubRegion>,
}

impl LoadedSchematic {
    pub fn empty() -> Self {
        Self {
            name: "Empty".to_string(),
            author: "None".to_string(),
            description: String::new(),
            time_created: 0,
            time_modified: 0,
            total_volume: 0,
            total_blocks: 0,
            enclosing_size: [0, 0, 0],
            regions: Vec::new(),
        }
    }

    pub fn primary_region(&self) -> Option<&LoadedSubRegion> {
        self.regions.first()
    }

    pub fn total_non_air_blocks(&self) -> usize {
        let mut count = 0;
        for r in &self.regions {
            for y in 0..r.container.size_y {
                for z in 0..r.container.size_z {
                    for x in 0..r.container.size_x {
                        if let Some(st) = r.container.get(x, y, z) {
                            if !st.is_air() {
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
        count
    }
}

// =========================================================================
// Helpers for NBT inspection
// =========================================================================

fn nbt_get<'a>(compound: &'a Compound, key: &str) -> Option<&'a Tag> {
    compound.get(&JavaString::from(key))
}

fn nbt_int(tag: Option<&Tag>) -> Option<i32> {
    match tag? {
        Tag::Int(v) => Some(*v),
        Tag::Short(v) => Some(*v as i32),
        Tag::Byte(v) => Some(*v as i32),
        Tag::Long(v) => Some(*v as i32),
        _ => None,
    }
}

fn nbt_long(tag: Option<&Tag>) -> Option<i64> {
    match tag? {
        Tag::Long(v) => Some(*v),
        Tag::Int(v) => Some(*v as i64),
        Tag::Short(v) => Some(*v as i64),
        Tag::Byte(v) => Some(*v as i64),
        _ => None,
    }
}

fn nbt_string(tag: Option<&Tag>) -> Option<String> {
    match tag? {
        Tag::String(js) => Some(js.to_string_lossy()),
        _ => None,
    }
}

fn nbt_compound<'a>(tag: Option<&'a Tag>) -> Option<&'a Compound> {
    match tag? {
        Tag::Compound(c) => Some(c),
        _ => None,
    }
}

fn nbt_list<'a>(tag: Option<&'a Tag>) -> Option<&'a Vec<Tag>> {
    match tag? {
        Tag::List(l) => Some(l),
        _ => None,
    }
}

fn nbt_pos(compound: &Compound, key: &str) -> [i32; 3] {
    if let Some(pos_c) = nbt_compound(nbt_get(compound, key)) {
        let x = nbt_int(nbt_get(pos_c, "x")).unwrap_or(0);
        let y = nbt_int(nbt_get(pos_c, "y")).unwrap_or(0);
        let z = nbt_int(nbt_get(pos_c, "z")).unwrap_or(0);
        [x, y, z]
    } else {
        [0, 0, 0]
    }
}

// =========================================================================
// Universal Load Dispatcher
// =========================================================================

pub fn load_from_bytes(raw_bytes: &[u8], filename: &str) -> Result<LoadedSchematic, String> {
    if raw_bytes.is_empty() {
        return Err("Schematic data is empty".to_string());
    }

    // Attempt GZIP decompression first if header matches 0x1f 0x8b
    let decompressed = if raw_bytes.starts_with(&[0x1f, 0x8b]) {
        Compression::Gzip
            .decode(raw_bytes, 128 * 1024 * 1024)
            .map_err(|e| format!("GZIP decompression error: {e}"))?
    } else if raw_bytes.starts_with(&[0x78, 0x9c]) || raw_bytes.starts_with(&[0x78, 0x01]) || raw_bytes.starts_with(&[0x78, 0xda]) {
        Compression::Zlib
            .decode(raw_bytes, 128 * 1024 * 1024)
            .map_err(|e| format!("ZLIB decompression error: {e}"))?
    } else {
        raw_bytes.to_vec()
    };

    let limits = Limits {
        max_bytes: 256 * 1024 * 1024,
        max_depth: 512,
    };
    let (root_named, _) = nbt::read_named(&decompressed, limits)
        .map_err(|e| format!("NBT parse failure: {e}"))?;

    let root = match root_named.tag {
        Tag::Compound(c) => c,
        _ => return Err("Root NBT tag is not a Compound".to_string()),
    };

    // Detect format:
    if nbt_get(&root, "Regions").is_some() {
        load_litematic(&root, filename)
    } else if nbt_get(&root, "Width").is_some() && nbt_get(&root, "BlockData").is_some() {
        load_sponge_schematic(&root, filename)
    } else if nbt_get(&root, "size").is_some() && nbt_get(&root, "blocks").is_some() {
        load_vanilla_structure(&root, filename)
    } else {
        Err("Unrecognized schematic format (missing Regions, Width/BlockData, or size/blocks)".to_string())
    }
}

// =========================================================================
// 1. .litematic Format Parser
// =========================================================================

fn load_litematic(root: &Compound, filename: &str) -> Result<LoadedSchematic, String> {
    let meta_tag = nbt_compound(nbt_get(root, "Metadata"));
    let schem_name = meta_tag
        .and_then(|m| nbt_string(nbt_get(m, "Name")))
        .unwrap_or_else(|| filename.trim_end_matches(".litematic").to_string());
    let author = meta_tag
        .and_then(|m| nbt_string(nbt_get(m, "Author")))
        .unwrap_or_else(|| "Unknown".to_string());
    let description = meta_tag
        .and_then(|m| nbt_string(nbt_get(m, "Description")))
        .unwrap_or_default();
    let time_created = meta_tag
        .and_then(|m| nbt_long(nbt_get(m, "TimeCreated")))
        .unwrap_or(0) as u64;
    let time_modified = meta_tag
        .and_then(|m| nbt_long(nbt_get(m, "TimeModified")))
        .unwrap_or(0) as u64;
    let total_volume = meta_tag
        .and_then(|m| nbt_long(nbt_get(m, "TotalVolume")))
        .unwrap_or(0) as u64;
    let total_blocks = meta_tag
        .and_then(|m| nbt_long(nbt_get(m, "TotalBlocks")))
        .unwrap_or(0) as u64;
    let enclosing_size = meta_tag
        .map(|m| nbt_pos(m, "EnclosingSize"))
        .unwrap_or([0, 0, 0]);

    let regions_comp = nbt_compound(nbt_get(root, "Regions"))
        .ok_or_else(|| "Missing 'Regions' tag in .litematic".to_string())?;

    let mut regions = Vec::new();

    for (r_name_js, r_tag) in regions_comp {
        let r_name = r_name_js.to_string_lossy();
        let Some(r_comp) = nbt_compound(Some(r_tag)) else {
            continue;
        };

        let raw_pos = nbt_pos(r_comp, "Position");
        let raw_size = nbt_pos(r_comp, "Size");

        // Parse Palette
        let mut palette = Vec::new();
        if let Some(pal_list) = nbt_list(nbt_get(r_comp, "BlockStatePalette")) {
            for entry in pal_list {
                if let Some(entry_comp) = nbt_compound(Some(entry)) {
                    let name = nbt_string(nbt_get(entry_comp, "Name"))
                        .unwrap_or_else(|| "minecraft:air".to_string());
                    let mut props = BTreeMap::new();
                    if let Some(props_comp) = nbt_compound(nbt_get(entry_comp, "Properties")) {
                        for (pk, pv) in props_comp {
                            if let Some(val_str) = nbt_string(Some(pv)) {
                                props.insert(pk.to_string_lossy(), val_str);
                            }
                        }
                    }
                    palette.push(SchematicBlockState {
                        name,
                        properties: props,
                    });
                }
            }
        }
        if palette.is_empty() {
            palette.push(SchematicBlockState::air());
        }

        // Parse BlockStates LongArray
        let backing_longs = match nbt_get(r_comp, "BlockStates") {
            Some(Tag::LongArray(arr)) => arr.clone(),
            _ => Vec::new(),
        };

        // Compute normalized min pos and dimensions
        let end_x = raw_pos[0] + if raw_size[0] >= 0 { raw_size[0] - 1 } else { raw_size[0] + 1 };
        let end_y = raw_pos[1] + if raw_size[1] >= 0 { raw_size[1] - 1 } else { raw_size[1] + 1 };
        let end_z = raw_pos[2] + if raw_size[2] >= 0 { raw_size[2] - 1 } else { raw_size[2] + 1 };

        let min_x = raw_pos[0].min(end_x);
        let min_y = raw_pos[1].min(end_y);
        let min_z = raw_pos[2].min(end_z);

        let max_x = raw_pos[0].max(end_x);
        let max_y = raw_pos[1].max(end_y);
        let max_z = raw_pos[2].max(end_z);

        let norm_size_x = (max_x - min_x + 1).max(1);
        let norm_size_y = (max_y - min_y + 1).max(1);
        let norm_size_z = (max_z - min_z + 1).max(1);

        let container = LitematicaBlockStateContainer::from_existing(
            norm_size_x,
            norm_size_y,
            norm_size_z,
            palette,
            backing_longs,
        );

        regions.push(LoadedSubRegion {
            name: r_name,
            pos: [min_x, min_y, min_z],
            size: [norm_size_x, norm_size_y, norm_size_z],
            container,
        });
    }

    Ok(LoadedSchematic {
        name: schem_name,
        author,
        description,
        time_created,
        time_modified,
        total_volume,
        total_blocks,
        enclosing_size,
        regions,
    })
}

// =========================================================================
// 2. Sponge .schem Format Parser
// =========================================================================

fn load_sponge_schematic(root: &Compound, filename: &str) -> Result<LoadedSchematic, String> {
    let width = nbt_int(nbt_get(root, "Width")).unwrap_or(1).abs();
    let height = nbt_int(nbt_get(root, "Height")).unwrap_or(1).abs();
    let length = nbt_int(nbt_get(root, "Length")).unwrap_or(1).abs();

    let offset = match nbt_get(root, "Offset") {
        Some(Tag::IntArray(arr)) if arr.len() >= 3 => [arr[0], arr[1], arr[2]],
        _ => [0, 0, 0],
    };

    // Parse Palette: BlockState string -> ID
    let mut palette_entries: BTreeMap<i32, SchematicBlockState> = BTreeMap::new();
    if let Some(pal_comp) = nbt_compound(nbt_get(root, "Palette")) {
        for (state_str_js, id_tag) in pal_comp {
            let id = nbt_int(Some(id_tag)).unwrap_or(0);
            let state_str = state_str_js.to_string_lossy();
            let parsed_state = parse_blockstate_string(&state_str);
            palette_entries.insert(id, parsed_state);
        }
    }

    let mut palette = Vec::new();
    let max_id = palette_entries.keys().copied().max().unwrap_or(0);
    for id in 0..=max_id {
        if let Some(st) = palette_entries.get(&id) {
            palette.push(st.clone());
        } else {
            palette.push(SchematicBlockState::air());
        }
    }
    if palette.is_empty() {
        palette.push(SchematicBlockState::air());
    }

    let mut container = LitematicaBlockStateContainer::new(width, height, length);
    container.palette = palette;

    // Decode VarInt byte buffer
    if let Some(Tag::ByteArray(bytes)) = nbt_get(root, "BlockData") {
        let mut cursor = 0;
        let total_vol = (width as usize) * (height as usize) * (length as usize);
        for index in 0..total_vol {
            if cursor >= bytes.len() {
                break;
            }
            let id = read_varint(bytes, &mut cursor);
            let layer_size = (width as usize) * (length as usize);
            let y = (index / layer_size) as i32;
            let rem = index % layer_size;
            let z = (rem / (width as usize)) as i32;
            let x = (rem % (width as usize)) as i32;

            if let Some(st) = container.palette.get(id as usize).cloned() {
                container.set(x, y, z, st);
            }
        }
    }

    let region = LoadedSubRegion {
        name: "Main".to_string(),
        pos: offset,
        size: [width, height, length],
        container,
    };

    Ok(LoadedSchematic {
        name: filename.trim_end_matches(".schem").to_string(),
        author: "Sponge".to_string(),
        description: String::new(),
        time_created: 0,
        time_modified: 0,
        total_volume: (width as u64) * (height as u64) * (length as u64),
        total_blocks: 0,
        enclosing_size: [width, height, length],
        regions: vec![region],
    })
}

// =========================================================================
// 3. Vanilla Structure Template .nbt Parser
// =========================================================================

fn load_vanilla_structure(root: &Compound, filename: &str) -> Result<LoadedSchematic, String> {
    let size = match nbt_get(root, "size") {
        Some(Tag::List(l)) if l.len() >= 3 => [
            nbt_int(l.get(0)).unwrap_or(1),
            nbt_int(l.get(1)).unwrap_or(1),
            nbt_int(l.get(2)).unwrap_or(1),
        ],
        _ => [1, 1, 1],
    };

    let mut palette = Vec::new();
    if let Some(pal_list) = nbt_list(nbt_get(root, "palette")) {
        for entry in pal_list {
            if let Some(entry_comp) = nbt_compound(Some(entry)) {
                let name = nbt_string(nbt_get(entry_comp, "Name"))
                    .unwrap_or_else(|| "minecraft:air".to_string());
                let mut props = BTreeMap::new();
                if let Some(props_comp) = nbt_compound(nbt_get(entry_comp, "Properties")) {
                    for (pk, pv) in props_comp {
                        if let Some(val_str) = nbt_string(Some(pv)) {
                            props.insert(pk.to_string_lossy(), val_str);
                        }
                    }
                }
                palette.push(SchematicBlockState {
                    name,
                    properties: props,
                });
            }
        }
    }
    if palette.is_empty() {
        palette.push(SchematicBlockState::air());
    }

    let mut container = LitematicaBlockStateContainer::new(size[0], size[1], size[2]);
    container.palette = palette;

    if let Some(blocks_list) = nbt_list(nbt_get(root, "blocks")) {
        for b_tag in blocks_list {
            if let Some(b_comp) = nbt_compound(Some(b_tag)) {
                let state_id = nbt_int(nbt_get(b_comp, "state")).unwrap_or(0) as usize;
                let pos = match nbt_get(b_comp, "pos") {
                    Some(Tag::List(pl)) if pl.len() >= 3 => [
                        nbt_int(pl.get(0)).unwrap_or(0),
                        nbt_int(pl.get(1)).unwrap_or(0),
                        nbt_int(pl.get(2)).unwrap_or(0),
                    ],
                    _ => [0, 0, 0],
                };
                if let Some(st) = container.palette.get(state_id).cloned() {
                    container.set(pos[0], pos[1], pos[2], st);
                }
            }
        }
    }

    let region = LoadedSubRegion {
        name: "Main".to_string(),
        pos: [0, 0, 0],
        size,
        container,
    };

    Ok(LoadedSchematic {
        name: filename.trim_end_matches(".nbt").to_string(),
        author: "Structure".to_string(),
        description: String::new(),
        time_created: 0,
        time_modified: 0,
        total_volume: (size[0] as u64) * (size[1] as u64) * (size[2] as u64),
        total_blocks: 0,
        enclosing_size: size,
        regions: vec![region],
    })
}

// =========================================================================
// .litematic Serializer (Write out)
// =========================================================================

pub fn save_to_litematic(schematic: &LoadedSchematic) -> Result<Vec<u8>, String> {
    let mut root = Compound::new();

    root.insert(JavaString::from("MinecraftDataVersion"), Tag::Int(3837));
    root.insert(JavaString::from("Version"), Tag::Int(6));
    root.insert(JavaString::from("SubVersion"), Tag::Int(1));

    // Metadata
    let mut meta = Compound::new();
    meta.insert(JavaString::from("Name"), Tag::String(JavaString::from(schematic.name.as_str())));
    meta.insert(JavaString::from("Author"), Tag::String(JavaString::from(schematic.author.as_str())));
    meta.insert(JavaString::from("Description"), Tag::String(JavaString::from(schematic.description.as_str())));
    meta.insert(JavaString::from("TimeCreated"), Tag::Long(schematic.time_created as i64));
    meta.insert(JavaString::from("TimeModified"), Tag::Long(schematic.time_modified as i64));
    meta.insert(JavaString::from("TotalVolume"), Tag::Long(schematic.total_volume as i64));
    meta.insert(JavaString::from("TotalBlocks"), Tag::Long(schematic.total_blocks as i64));
    meta.insert(JavaString::from("RegionCount"), Tag::Int(schematic.regions.len() as i32));

    let mut enc_size = Compound::new();
    enc_size.insert(JavaString::from("x"), Tag::Int(schematic.enclosing_size[0]));
    enc_size.insert(JavaString::from("y"), Tag::Int(schematic.enclosing_size[1]));
    enc_size.insert(JavaString::from("z"), Tag::Int(schematic.enclosing_size[2]));
    meta.insert(JavaString::from("EnclosingSize"), Tag::Compound(enc_size));

    root.insert(JavaString::from("Metadata"), Tag::Compound(meta));

    // Regions
    let mut regions_comp = Compound::new();
    for r in &schematic.regions {
        let mut r_comp = Compound::new();

        let mut pos_comp = Compound::new();
        pos_comp.insert(JavaString::from("x"), Tag::Int(r.pos[0]));
        pos_comp.insert(JavaString::from("y"), Tag::Int(r.pos[1]));
        pos_comp.insert(JavaString::from("z"), Tag::Int(r.pos[2]));
        r_comp.insert(JavaString::from("Position"), Tag::Compound(pos_comp));

        let mut size_comp = Compound::new();
        size_comp.insert(JavaString::from("x"), Tag::Int(r.size[0]));
        size_comp.insert(JavaString::from("y"), Tag::Int(r.size[1]));
        size_comp.insert(JavaString::from("z"), Tag::Int(r.size[2]));
        r_comp.insert(JavaString::from("Size"), Tag::Compound(size_comp));

        // Palette
        let mut pal_list = Vec::new();
        for s in &r.container.palette {
            let mut s_comp = Compound::new();
            s_comp.insert(JavaString::from("Name"), Tag::String(JavaString::from(s.name.as_str())));
            if !s.properties.is_empty() {
                let mut props_comp = Compound::new();
                for (k, v) in &s.properties {
                    props_comp.insert(JavaString::from(k.as_str()), Tag::String(JavaString::from(v.as_str())));
                }
                s_comp.insert(JavaString::from("Properties"), Tag::Compound(props_comp));
            }
            pal_list.push(Tag::Compound(s_comp));
        }
        r_comp.insert(JavaString::from("BlockStatePalette"), Tag::List(pal_list));
        r_comp.insert(JavaString::from("BlockStates"), Tag::LongArray(r.container.storage.long_array.clone()));

        regions_comp.insert(JavaString::from(r.name.as_str()), Tag::Compound(r_comp));
    }
    root.insert(JavaString::from("Regions"), Tag::Compound(regions_comp));

    let named = NamedTag {
        name: JavaString::from(""),
        tag: Tag::Compound(root),
    };
    let uncompressed = nbt::write_named(&named, Limits::saved())
        .map_err(|e| format!("Failed to serialize NBT: {e}"))?;

    let compressed = Compression::Gzip
        .encode(&uncompressed)
        .map_err(|e| format!("Failed to compress GZIP: {e}"))?;

    Ok(compressed)
}

// =========================================================================
// Parsers & ByteBuf utilities
// =========================================================================

fn parse_blockstate_string(s: &str) -> SchematicBlockState {
    if let Some((name, props_str)) = s.split_once('[') {
        let clean_props = props_str.trim_end_matches(']');
        let mut props = BTreeMap::new();
        for kv in clean_props.split(',') {
            if let Some((k, v)) = kv.split_once('=') {
                props.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
        SchematicBlockState {
            name: name.trim().to_string(),
            properties: props,
        }
    } else {
        SchematicBlockState::new(s.trim())
    }
}

fn read_varint(bytes: &[u8], cursor: &mut usize) -> i32 {
    let mut num_read = 0;
    let mut result = 0;
    while *cursor < bytes.len() {
        let read = bytes[*cursor];
        *cursor += 1;
        let value = (read & 0x7F) as i32;
        result |= value << (7 * num_read);
        num_read += 1;
        if num_read > 5 {
            break;
        }
        if (read & 0x80) == 0 {
            break;
        }
    }
    result
}
