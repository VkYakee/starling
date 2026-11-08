// Copyright (c) 2026 HorizonMatica Authors / Starling Android
//! Universal schematic storage manager for Android and desktop environments.
//! Resolves /sdcard/Documents/Horizonmatica/schematics/ with fallback to app data.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use super::container::{LitematicaBlockStateContainer, SchematicBlockState};
use super::format::{self, LoadedSchematic, LoadedSubRegion};

#[derive(Clone, Debug)]
pub struct SchematicFileInfo {
    pub filename: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub modified_secs: u64,
}

impl SchematicFileInfo {
    pub fn formatted_size(&self) -> String {
        if self.size_bytes < 1024 {
            format!("{} B", self.size_bytes)
        } else if self.size_bytes < 1024 * 1024 {
            format!("{:.1} KB", self.size_bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", self.size_bytes as f64 / (1024.0 * 1024.0))
        }
    }
}

/// Resolves the primary directory for storing and browsing schematics on Android.
pub fn get_schematics_dir() -> PathBuf {
    // 1. Android public user documents path
    let sdcard_docs = PathBuf::from("/sdcard/Documents/Horizonmatica/schematics");
    if sdcard_docs.exists() || fs::create_dir_all(&sdcard_docs).is_ok() {
        return sdcard_docs;
    }

    let emulated_docs = PathBuf::from("/storage/emulated/0/Documents/Horizonmatica/schematics");
    if emulated_docs.exists() || fs::create_dir_all(&emulated_docs).is_ok() {
        return emulated_docs;
    }

    // 2. HORIZON_HOME environment or local share fallback
    if let Ok(horizon_home) = std::env::var("HORIZON_HOME") {
        let p = PathBuf::from(horizon_home).join("schematics");
        let _ = fs::create_dir_all(&p);
        return p;
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let fallback = PathBuf::from(home).join(".local/share/horizon/schematics");
    let _ = fs::create_dir_all(&fallback);
    fallback
}

/// Ensures the directory exists and provisions a sample starter schematic if empty.
pub fn ensure_schematics_dir() -> io::Result<PathBuf> {
    let dir = get_schematics_dir();
    fs::create_dir_all(&dir)?;

    let existing = list_schematics_in_dir(&dir);
    if existing.is_empty() {
        // Create demo sample schematic
        let sample = generate_demo_house_schematic();
        if let Ok(bytes) = format::save_to_litematic(&sample) {
            let sample_path = dir.join("starter_cottage.litematic");
            let _ = fs::write(&sample_path, bytes);
        }
    }

    Ok(dir)
}

/// Lists all valid schematic files (.litematic, .schem, .nbt) in the schematics folder.
pub fn list_schematics() -> Vec<SchematicFileInfo> {
    let dir = get_schematics_dir();
    list_schematics_in_dir(&dir)
}

fn list_schematics_in_dir(dir: &Path) -> Vec<SchematicFileInfo> {
    let mut results = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return results;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();

        let lower = name.to_lowercase();
        if lower.ends_with(".litematic") || lower.ends_with(".schem") || lower.ends_with(".nbt") {
            let (size_bytes, modified_secs) = if let Ok(meta) = entry.metadata() {
                let size = meta.len();
                let mod_time = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                (size, mod_time)
            } else {
                (0, 0)
            };

            results.push(SchematicFileInfo {
                filename: name,
                path,
                size_bytes,
                modified_secs,
            });
        }
    }

    // Sort alphabetically by filename
    results.sort_by(|a, b| a.filename.to_lowercase().cmp(&b.filename.to_lowercase()));
    results
}

/// Loads raw schematic bytes by either filename (in default dir) or full path.
pub fn load_schematic_file(name_or_path: &str) -> io::Result<(PathBuf, Vec<u8>)> {
    let dir = get_schematics_dir();
    let target_path = if name_or_path.contains('/') || name_or_path.contains('\\') {
        PathBuf::from(name_or_path)
    } else {
        let mut p = dir.join(name_or_path);
        if !p.exists() && !name_or_path.ends_with(".litematic") && !name_or_path.ends_with(".schem") && !name_or_path.ends_with(".nbt") {
            p = dir.join(format!("{name_or_path}.litematic"));
        }
        p
    };

    let bytes = fs::read(&target_path)?;
    Ok((target_path, bytes))
}

/// Builds a 5x4x5 cozy cottage as an out-of-the-box working sample schematic.
fn generate_demo_house_schematic() -> LoadedSchematic {
    let sx = 5;
    let sy = 4;
    let sz = 5;
    let mut container = LitematicaBlockStateContainer::new(sx, sy, sz);

    let stone_bricks = SchematicBlockState::new("minecraft:stone_bricks");
    let cobblestone = SchematicBlockState::new("minecraft:cobblestone");
    let oak_planks = SchematicBlockState::new("minecraft:oak_planks");
    let glass = SchematicBlockState::new("minecraft:glass");
    let torch = SchematicBlockState::new("minecraft:torch");

    // Floor (y=0)
    for x in 0..sx {
        for z in 0..sz {
            container.set(x, 0, z, stone_bricks.clone());
        }
    }

    // Walls (y=1..2)
    for y in 1..=2 {
        for x in 0..sx {
            for z in 0..sz {
                let is_corner = (x == 0 || x == sx - 1) && (z == 0 || z == sz - 1);
                let is_edge = x == 0 || x == sx - 1 || z == 0 || z == sz - 1;

                if is_corner {
                    container.set(x, y, z, cobblestone.clone());
                } else if is_edge {
                    // Window openings or door opening
                    if x == 2 && z == 0 && y == 1 {
                        // Doorway
                    } else if (x == 2 && (z == 0 || z == sz - 1)) || (z == 2 && (x == 0 || x == sx - 1)) {
                        container.set(x, y, z, glass.clone());
                    } else {
                        container.set(x, y, z, oak_planks.clone());
                    }
                }
            }
        }
    }

    // Interior torch (y=2, inside corner)
    container.set(1, 2, 3, torch.clone());

    // Roof (y=3)
    for x in 0..sx {
        for z in 0..sz {
            container.set(x, 3, z, oak_planks.clone());
        }
    }

    let sub = LoadedSubRegion {
        name: "Cottage".to_string(),
        pos: [0, 0, 0],
        size: [sx, sy, sz],
        container,
    };

    LoadedSchematic {
        name: "Starter Cottage".to_string(),
        author: "HorizonMatica".to_string(),
        description: "A compact 5x4x5 starter house for testing HorizonMatica.".to_string(),
        time_created: 1731000000,
        time_modified: 1731000000,
        total_volume: (sx * sy * sz) as u64,
        total_blocks: 54,
        enclosing_size: [sx, sy, sz],
        regions: vec![sub],
    }
}
