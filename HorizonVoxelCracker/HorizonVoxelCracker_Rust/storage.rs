// Copyright (c) 2026 VkYakee
// VkYakee VoxelMap x seedcrackerX
//! Universal Android Device Storage Resolver.
//! Works across all Android manufacturers (Samsung, Xiaomi/MIUI, Pixel, Motorola, etc.)
//! and Android versions (7 through 15) using Scoped Storage-compliant public folders (Documents/Download).
//! Automatically creates and isolates folders per world and server.

use super::cracker::{CollectedFeature, MCVersion, StructureType};
use super::hud::HudSettings;
use super::waypoints::Waypoint;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

static RESOLVED_ROOT: OnceLock<PathBuf> = OnceLock::new();

/// Test if a path is writable by creating and removing a test file
fn is_writable(dir: &std::path::Path) -> bool {
    if fs::create_dir_all(dir).is_err() {
        return false;
    }
    let test_file = dir.join(".probe_write_test");
    if fs::write(&test_file, b"ok").is_ok() {
        let _ = fs::remove_file(test_file);
        true
    } else {
        false
    }
}

/// Automatically find the best accessible public storage path on ANY Android device
pub fn device_storage_root() -> PathBuf {
    RESOLVED_ROOT
        .get_or_init(|| {
            // Candidates list in order of preference for Android compatibility
            let candidate_paths = [
                // 1. Android Documents (Standard SAF/MediaStore public location, visible in Files app)
                PathBuf::from("/sdcard/Documents/HorizonVoxelCracker"),
                PathBuf::from("/storage/emulated/0/Documents/HorizonVoxelCracker"),
                // 2. Android Download (Always accessible on modern Android)
                PathBuf::from("/sdcard/Download/HorizonVoxelCracker"),
                PathBuf::from("/storage/emulated/0/Download/HorizonVoxelCracker"),
                // 3. Root SDCard folder (Android 9 and below, or with full storage rights)
                PathBuf::from("/sdcard/HorizonVoxelCracker"),
                PathBuf::from("/storage/emulated/0/HorizonVoxelCracker"),
            ];

            for path in &candidate_paths {
                if is_writable(path) {
                    let readme = path.join("LEEME_DATOS.txt");
                    let _ = fs::write(
                        readme,
                        "HorizonVoxelCracker - Almacenamiento de Mapas y Semillas en Android\n\
                         ==================================================================\n\
                         Esta carpeta guarda tus waypoints y el progreso del cracker de forma\n\
                         independiente para cada mundo y servidor.\n\n\
                         Estructura:\n\
                         - global_config.json                 : Opciones tactiles y visuales\n\
                         - worlds/<nombre_mundo>/waypoints.json    : Tus waypoints en formato JSON editable\n\
                         - worlds/<nombre_mundo>/seedcracker.json  : Semillas y estructuras descubiertas\n\n\
                         Puedes respaldar, copiar o transferir estas carpetas a otros dispositivos.\n",
                    );
                    return path.clone();
                }
            }

            // Fallback for Termux environment
            if let Ok(home) = std::env::var("HOME") {
                let termux_path = PathBuf::from(&home).join("storage").join("shared").join("HorizonVoxelCracker");
                if is_writable(&termux_path) {
                    return termux_path;
                }
            }

            // Fallback for HORIZON_HOME
            if let Ok(h) = std::env::var("HORIZON_HOME") {
                let p = PathBuf::from(h).join("horizon_voxel_cracker");
                if fs::create_dir_all(&p).is_ok() {
                    return p;
                }
            }

            // Ultimate fallback in app-private folder
            let local_p = PathBuf::from(".local").join("horizon_voxel_cracker");
            let _ = fs::create_dir_all(&local_p);
            local_p
        })
        .clone()
}

/// Sanitize world or server names for safe folder creation across Android filesystems (FAT32, ext4, F2FS)
pub fn sanitize_folder_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == '.' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "mundo_default".into()
    } else {
        out
    }
}

/// Get the isolated folder for a specific world or server
pub fn world_storage_directory(world_id: &str) -> PathBuf {
    let root = device_storage_root();
    let sanitized = sanitize_folder_name(world_id);
    let world_dir = root.join("worlds").join(sanitized);
    let _ = fs::create_dir_all(&world_dir);
    world_dir
}

// ---------------------------------------------------------------- World Metadata

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldInfo {
    pub world_id: String,
    pub display_name: String,
    pub detected_version: String,
    pub last_played: String,
}

pub fn save_world_info(world_id: &str, display_name: &str, version: MCVersion) {
    let dir = world_storage_directory(world_id);
    let file = dir.join("info.json");
    let info = WorldInfo {
        world_id: world_id.to_string(),
        display_name: display_name.to_string(),
        detected_version: version.name().to_string(),
        last_played: format!("{:?}", std::time::SystemTime::now()),
    };
    if let Ok(json_str) = serde_json::to_string_pretty(&info) {
        let _ = fs::write(file, json_str);
    }
}

// ---------------------------------------------------------------- World-specific Waypoints

pub fn save_world_waypoints(world_id: &str, waypoints: &[Waypoint]) {
    let dir = world_storage_directory(world_id);
    let file = dir.join("waypoints.json");
    if let Ok(json_str) = serde_json::to_string_pretty(waypoints) {
        let _ = fs::write(file, json_str);
    }
}

pub fn load_world_waypoints(world_id: &str) -> Vec<Waypoint> {
    let dir = world_storage_directory(world_id);
    let file = dir.join("waypoints.json");
    if let Ok(content) = fs::read_to_string(file) {
        if let Ok(list) = serde_json::from_str::<Vec<Waypoint>>(&content) {
            return list;
        }
    }
    Vec::new()
}

// ---------------------------------------------------------------- World-specific Cracker Data

#[derive(Serialize, Deserialize)]
struct StoredFeature {
    pub kind_name: String,
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub block_y: i32,
}

#[derive(Serialize, Deserialize)]
struct StoredCracker {
    pub version: String,
    pub features: Vec<StoredFeature>,
    pub structure_seed: Option<i64>,
    pub world_seed: Option<i64>,
}

pub fn save_world_cracker_data(
    world_id: &str,
    version: MCVersion,
    features: &[CollectedFeature],
    struct_seed: Option<i64>,
    world_seed: Option<i64>,
) {
    let dir = world_storage_directory(world_id);
    let file = dir.join("seedcracker.json");
    let stored_feats: Vec<StoredFeature> = features
        .iter()
        .map(|f| StoredFeature {
            kind_name: f.kind.name().into(),
            chunk_x: f.chunk_x,
            chunk_z: f.chunk_z,
            block_y: f.block_y,
        })
        .collect();

    let data = StoredCracker {
        version: version.name().into(),
        features: stored_feats,
        structure_seed: struct_seed,
        world_seed,
    };

    if let Ok(json_str) = serde_json::to_string_pretty(&data) {
        let _ = fs::write(file, json_str);
    }
}

pub fn load_world_cracker_data(world_id: &str) -> (MCVersion, Vec<CollectedFeature>, Option<i64>, Option<i64>) {
    let dir = world_storage_directory(world_id);
    let file = dir.join("seedcracker.json");
    if let Ok(content) = fs::read_to_string(file) {
        if let Ok(data) = serde_json::from_str::<StoredCracker>(&content) {
            let ver = match data.version.as_str() {
                "Pre-1.18 (1.14-1.17)" => MCVersion::Pre1_18,
                "1.18+ (Caves & Cliffs)" => MCVersion::V1_18_Plus,
                "1.21+ (Modern)" => MCVersion::V1_21_Plus,
                _ => MCVersion::Auto,
            };
            let mut list = Vec::new();
            for sf in data.features {
                let kind = match sf.kind_name.as_str() {
                    "Desert Pyramid" => StructureType::DesertPyramid,
                    "Jungle Temple" => StructureType::JungleTemple,
                    "Swamp Hut" => StructureType::SwampHut,
                    "Igloo" => StructureType::Igloo,
                    "Shipwreck" => StructureType::Shipwreck,
                    "Pillager Outpost" => StructureType::PillagerOutpost,
                    "Ocean Monument" => StructureType::OceanMonument,
                    "Buried Treasure" => StructureType::BuriedTreasure,
                    "End Pillars" => StructureType::EndPillars,
                    _ => StructureType::Dungeon,
                };
                list.push(CollectedFeature::new(kind, sf.chunk_x, sf.chunk_z, sf.block_y));
            }
            return (ver, list, data.structure_seed, data.world_seed);
        }
    }
    (MCVersion::Auto, Vec::new(), None, None)
}

// ---------------------------------------------------------------- Global Settings

#[derive(Serialize, Deserialize)]
struct GlobalConfig {
    pub show_minimap: bool,
    pub show_radar: bool,
    pub show_coords: bool,
    pub show_waypoints: bool,
    pub show_cracker_status: bool,
    pub cave_mode: bool,
    pub minimap_size: f32,
    pub map_mode: u8,
}

pub fn save_global_config(settings: &HudSettings, map_mode: u8) {
    let dir = device_storage_root();
    let file = dir.join("global_config.json");
    let data = GlobalConfig {
        show_minimap: settings.show_minimap,
        show_radar: settings.show_radar,
        show_coords: settings.show_coords,
        show_waypoints: settings.show_waypoints,
        show_cracker_status: settings.show_cracker_status,
        cave_mode: settings.cave_mode,
        minimap_size: settings.minimap_size,
        map_mode,
    };
    if let Ok(json_str) = serde_json::to_string_pretty(&data) {
        let _ = fs::write(file, json_str);
    }
}

pub fn load_global_config() -> (HudSettings, u8) {
    let dir = device_storage_root();
    let file = dir.join("global_config.json");
    if let Ok(content) = fs::read_to_string(file) {
        if let Ok(data) = serde_json::from_str::<GlobalConfig>(&content) {
            let mut s = HudSettings::default();
            s.show_minimap = data.show_minimap;
            s.show_radar = data.show_radar;
            s.show_coords = data.show_coords;
            s.show_waypoints = data.show_waypoints;
            s.show_cracker_status = data.show_cracker_status;
            s.cave_mode = data.cave_mode;
            s.minimap_size = data.minimap_size;
            return (s, data.map_mode);
        }
    }
    (HudSettings::default(), 0)
}
