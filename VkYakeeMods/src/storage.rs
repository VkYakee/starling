//! Storage and persistence engine for VkYakeeMods on Android and Starling.
//! Manages separate data directories for SeedCracker and HorizonMatica:
//!
//! /sdcard/VkYakeeMods (or ./VkYakeeMods_Data fallback):
//!   ├── SeedCracker/
//!   │     ├── worlds/    (world session JSONs)
//!   │     └── maps/      (exported CSV maps for navigation)
//!   └── HorizonMatica/
//!         ├── schematics/ (.litematic, .schem, .schematic, .nbt)
//!         └── exports/    (saved and exported schematics)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use horizonmatica::schematic::conversion::SchematicConverter;
use horizonmatica::schematic::litematic::{LitematicaSchematic, SubRegion};
use horizonmatica::schematic::palette::BlockState;
use horizonmatica::util::pos::BlockPos;

/// A recorded biome point in the world.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveredBiome {
    pub x: i32,
    pub z: i32,
    pub biome_id: i32,
    pub name: String,
}

/// A recorded structure found during exploration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveredStructure {
    pub structure_type: String,
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub block_x: i32,
    pub block_z: i32,
}

/// Complete saved session data for a world or multiplayer server.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorldSessionData {
    pub world_id: String,
    pub dimension: String,
    pub hashed_seed: Option<u64>,
    pub cracked_seed: Option<i64>,
    pub updated_at_utc: u64,
    /// Bedrock chunk masks: key is "cx,cz", value is hex-encoded 256-byte mask.
    pub bedrock_chunks: HashMap<String, String>,
    pub biomes: Vec<DiscoveredBiome>,
    pub structures: Vec<DiscoveredStructure>,
    pub total_scanned_chunks: usize,
}

#[derive(Clone)]
pub struct StorageManager {
    base_dir: PathBuf,
    seedcracker_dir: PathBuf,
    horizonmatica_dir: PathBuf,
    vkmap_dir: PathBuf,
}

impl Default for StorageManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageManager {
    /// Determines the best mobile storage directory:
    /// 1. Tries Android public external storage `/sdcard/VkYakeeMods`
    /// 2. Falls back to local directory `./VkYakeeMods_Data`
    pub fn new() -> Self {
        let sdcard_path = PathBuf::from("/sdcard/VkYakeeMods");
        let base_dir = if sdcard_path.exists() || fs::create_dir_all(&sdcard_path).is_ok() {
            sdcard_path
        } else {
            let local_path = PathBuf::from("VkYakeeMods_Data");
            let _ = fs::create_dir_all(&local_path);
            local_path
        };

        let seedcracker_dir = base_dir.join("SeedCracker");
        let horizonmatica_dir = base_dir.join("HorizonMatica");
        let vkmap_dir = base_dir.join("VkMap");

        // Create SeedCracker subdirectories
        let _ = fs::create_dir_all(seedcracker_dir.join("worlds"));
        let _ = fs::create_dir_all(seedcracker_dir.join("maps"));

        // Create HorizonMatica subdirectories
        let schematics_dir = horizonmatica_dir.join("schematics");
        let exports_dir = horizonmatica_dir.join("exports");
        let _ = fs::create_dir_all(&schematics_dir);
        let _ = fs::create_dir_all(&exports_dir);

        // Create VkMap subdirectories
        let _ = fs::create_dir_all(vkmap_dir.join("worlds"));
        let _ = fs::create_dir_all(vkmap_dir.join("exports"));

        let mgr = Self {
            base_dir,
            seedcracker_dir,
            horizonmatica_dir,
            vkmap_dir,
        };

        // Create sample schematic if none exist
        mgr.ensure_sample_schematic();

        mgr
    }

    pub fn get_base_path(&self) -> &Path {
        &self.base_dir
    }

    pub fn seedcracker_dir(&self) -> &Path {
        &self.seedcracker_dir
    }

    pub fn seedcracker_worlds_dir(&self) -> PathBuf {
        self.seedcracker_dir.join("worlds")
    }

    pub fn seedcracker_maps_dir(&self) -> PathBuf {
        self.seedcracker_dir.join("maps")
    }

    pub fn horizonmatica_dir(&self) -> &Path {
        &self.horizonmatica_dir
    }

    pub fn schematics_dir(&self) -> PathBuf {
        self.horizonmatica_dir.join("schematics")
    }

    pub fn exports_dir(&self) -> PathBuf {
        self.horizonmatica_dir.join("exports")
    }

    pub fn vkmap_dir(&self) -> &Path {
        &self.vkmap_dir
    }

    pub fn vkmap_worlds_dir(&self) -> PathBuf {
        self.vkmap_dir.join("worlds")
    }

    pub fn vkmap_world_dir(&self, world_id: &str) -> PathBuf {
        let safe_name: String = world_id
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
            .collect();
        let path = self.vkmap_worlds_dir().join(safe_name);
        let _ = fs::create_dir_all(&path);
        let _ = fs::create_dir_all(path.join("tiles"));
        path
    }

    pub fn vkmap_exports_dir(&self) -> PathBuf {
        self.vkmap_dir.join("exports")
    }

    /// Generates a sanitized filename for a world / server session
    fn world_file_path(&self, world_id: &str) -> PathBuf {
        let safe_name: String = world_id
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
            .collect();
        self.seedcracker_worlds_dir().join(format!("{}.json", safe_name))
    }

    /// Loads existing world session data or initializes a new one.
    /// Checks both the new SeedCracker/worlds/ path and the legacy root worlds/ path.
    pub fn load_or_create(&self, world_id: &str) -> WorldSessionData {
        let path = self.world_file_path(world_id);
        let legacy_path = self.base_dir.join("worlds").join(format!("{}.json", world_id));

        let candidate_path = if path.exists() {
            Some(path)
        } else if legacy_path.exists() {
            Some(legacy_path)
        } else {
            None
        };

        if let Some(p) = candidate_path {
            if let Ok(content) = fs::read_to_string(&p) {
                if let Ok(data) = serde_json::from_str::<WorldSessionData>(&content) {
                    log::info!(
                        "VkYakeeMods Storage: Datos cargados para '{}' ({} chunks, {} estructuras)",
                        world_id,
                        data.total_scanned_chunks,
                        data.structures.len()
                    );
                    return data;
                }
            }
        }

        WorldSessionData {
            world_id: world_id.to_string(),
            dimension: "minecraft:overworld".to_string(),
            hashed_seed: None,
            cracked_seed: None,
            updated_at_utc: 0,
            bedrock_chunks: HashMap::new(),
            biomes: Vec::new(),
            structures: Vec::new(),
            total_scanned_chunks: 0,
        }
    }

    /// Saves session data to disk inside `SeedCracker/worlds/`.
    pub fn save(&self, data: &WorldSessionData) -> Result<(), String> {
        let path = self.world_file_path(&data.world_id);
        let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Exports discovered map points into a clean CSV format for external mapping tools.
    pub fn export_map_csv(&self, data: &WorldSessionData) -> Result<PathBuf, String> {
        let path = self.seedcracker_maps_dir().join(format!("{}_map.csv", data.world_id));
        let mut lines = Vec::new();
        lines.push("type,name,x,z".to_string());

        for b in &data.biomes {
            lines.push(format!("biome,{},{},{}", b.name, b.x, b.z));
        }
        for s in &data.structures {
            lines.push(format!("structure,{},{},{}", s.structure_type, s.block_x, s.block_z));
        }

        fs::write(&path, lines.join("\n")).map_err(|e| e.to_string())?;
        Ok(path)
    }

    // ==========================================
    // HorizonMatica Storage Operations
    // ==========================================

    /// Scans Android storage for schematics (.litematic, .schem, .schematic, .nbt).
    /// Scans both `HorizonMatica/schematics/`, `HorizonMatica/exports/`, and `HorizonMatica/`.
    pub fn list_schematics(&self) -> Vec<PathBuf> {
        let mut list = Vec::new();
        let search_dirs = [
            self.schematics_dir(),
            self.exports_dir(),
            self.horizonmatica_dir().to_path_buf(),
        ];

        for dir in &search_dirs {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                            let ext_lower = ext.to_ascii_lowercase();
                            if matches!(ext_lower.as_str(), "litematic" | "schem" | "schematic" | "nbt") {
                                if !list.contains(&path) {
                                    list.push(path);
                                }
                            }
                        }
                    }
                }
            }
        }

        list.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
        list
    }

    /// Loads any schematic from Android local storage.
    /// Can be passed a filename (e.g. "house.litematic", "farm.schem") or a full Path.
    pub fn load_schematic(&self, name_or_path: &str) -> Result<LitematicaSchematic, String> {
        let direct_path = PathBuf::from(name_or_path);
        if direct_path.exists() && direct_path.is_file() {
            return SchematicConverter::load_any_schematic(&direct_path)
                .map_err(|e| format!("Error cargando esquemática '{:?}': {}", direct_path, e));
        }

        // Search in schematics_dir, exports_dir, horizonmatica_dir
        let candidate_dirs = [
            self.schematics_dir(),
            self.exports_dir(),
            self.horizonmatica_dir().to_path_buf(),
        ];

        for dir in &candidate_dirs {
            // Direct candidate
            let p = dir.join(name_or_path);
            if p.exists() && p.is_file() {
                return SchematicConverter::load_any_schematic(&p)
                    .map_err(|e| format!("Error cargando esquemática '{:?}': {}", p, e));
            }

            // Try common extensions if missing
            for ext in &["litematic", "schem", "schematic", "nbt"] {
                let p_ext = dir.join(format!("{}.{}", name_or_path, ext));
                if p_ext.exists() && p_ext.is_file() {
                    return SchematicConverter::load_any_schematic(&p_ext)
                        .map_err(|e| format!("Error cargando esquemática '{:?}': {}", p_ext, e));
                }
            }
        }

        Err(format!(
            "No se encontró el archivo de esquemática '{}' en {}",
            name_or_path,
            self.schematics_dir().display()
        ))
    }

    /// Saves or exports a schematic into `HorizonMatica/exports/`.
    pub fn export_schematic(&self, name: &str, schematic: &mut LitematicaSchematic) -> Result<PathBuf, String> {
        let safe_name = if name.ends_with(".litematic") {
            name.to_string()
        } else {
            format!("{}.litematic", name)
        };
        let dest = self.exports_dir().join(safe_name);
        schematic
            .write_to_file(&dest)
            .map_err(|e| format!("Fallo al exportar esquemática a {:?}: {}", dest, e))?;
        Ok(dest)
    }

    /// Exports material list CSV into `HorizonMatica/exports/`.
    pub fn export_materials_csv(&self, name: &str, csv: &str) -> Result<PathBuf, String> {
        let safe_name = if name.ends_with(".csv") {
            name.to_string()
        } else {
            format!("{}_materials.csv", name)
        };
        let dest = self.exports_dir().join(safe_name);
        fs::write(&dest, csv).map_err(|e| format!("Fallo al exportar CSV a {:?}: {}", dest, e))?;
        Ok(dest)
    }

    /// Exports material list JSON into `HorizonMatica/exports/`.
    pub fn export_materials_json(&self, name: &str, json: &str) -> Result<PathBuf, String> {
        let safe_name = if name.ends_with(".json") {
            name.to_string()
        } else {
            format!("{}_materials.json", name)
        };
        let dest = self.exports_dir().join(safe_name);
        fs::write(&dest, json).map_err(|e| format!("Fallo al exportar JSON a {:?}: {}", dest, e))?;
        Ok(dest)
    }

    /// Saves a schematic into `HorizonMatica/schematics/`.
    pub fn save_schematic(&self, name: &str, schematic: &mut LitematicaSchematic) -> Result<PathBuf, String> {
        let safe_name = if name.ends_with(".litematic") {
            name.to_string()
        } else {
            format!("{}.litematic", name)
        };
        let dest = self.schematics_dir().join(safe_name);
        schematic
            .write_to_file(&dest)
            .map_err(|e| format!("Fallo al guardar esquemática en {:?}: {}", dest, e))?;
        Ok(dest)
    }

    /// Imports an external schematic file and copies it directly into `HorizonMatica/schematics/`.
    /// Searches typical Android download directories (/sdcard/Download, /storage/emulated/0/Download)
    /// if a relative or simple file name is provided.
    pub fn import_schematic(&self, source_path_or_filename: &str) -> Result<PathBuf, String> {
        let direct = PathBuf::from(source_path_or_filename);
        let source_path = if direct.exists() && direct.is_file() {
            direct
        } else {
            let search_paths = [
                PathBuf::from("/sdcard/Download"),
                PathBuf::from("/sdcard/Downloads"),
                PathBuf::from("/storage/emulated/0/Download"),
                PathBuf::from("/storage/emulated/0/Downloads"),
                PathBuf::from("/sdcard/Documents"),
                PathBuf::from("."),
            ];

            let mut found = None;
            for base in &search_paths {
                let candidate = base.join(source_path_or_filename);
                if candidate.exists() && candidate.is_file() {
                    found = Some(candidate);
                    break;
                }
                for ext in &["litematic", "schem", "schematic", "nbt"] {
                    let c_ext = base.join(format!("{}.{}", source_path_or_filename, ext));
                    if c_ext.exists() && c_ext.is_file() {
                        found = Some(c_ext);
                        break;
                    }
                }
                if found.is_some() {
                    break;
                }
            }

            match found {
                Some(p) => p,
                None => {
                    return Err(format!(
                        "No se encontró el archivo '{}' en /sdcard/Download ni en rutas locales.",
                        source_path_or_filename
                    ));
                }
            }
        };

        // Validate schematic before copying
        let _ = SchematicConverter::load_any_schematic(&source_path)
            .map_err(|e| format!("El archivo '{:?}' no es una esquemática válida: {}", source_path, e))?;

        let file_name = source_path
            .file_name()
            .ok_or_else(|| "Nombre de archivo inválido".to_string())?;

        let dest_path = self.schematics_dir().join(file_name);

        fs::copy(&source_path, &dest_path)
            .map_err(|e| format!("Error al copiar esquemática a {:?}: {}", dest_path, e))?;

        log::info!(
            "VkYakeeMods: Esquemática importada exitosamente de {:?} a {:?}",
            source_path,
            dest_path
        );

        Ok(dest_path)
    }

    /// Automatically scans /sdcard/Download/ for the latest .litematic, .schem, or .nbt and imports it.
    pub fn import_latest_from_downloads(&self) -> Result<PathBuf, String> {
        let download_dirs = [
            PathBuf::from("/sdcard/Download"),
            PathBuf::from("/sdcard/Downloads"),
            PathBuf::from("/storage/emulated/0/Download"),
            PathBuf::from("."),
        ];

        let mut newest_file: Option<(PathBuf, std::time::SystemTime)> = None;

        for dir in &download_dirs {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                            let ext_lower = ext.to_ascii_lowercase();
                            if matches!(ext_lower.as_str(), "litematic" | "schem" | "schematic" | "nbt") {
                                if let Ok(meta) = path.metadata() {
                                    if let Ok(modified) = meta.modified() {
                                        if newest_file.as_ref().map_or(true, |(_, m)| modified > *m) {
                                            newest_file = Some((path, modified));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some((path, _)) = newest_file {
            self.import_schematic(&path.to_string_lossy())
        } else {
            Err("No se encontraron esquemáticas (.litematic, .schem, .nbt) en Descargas.".to_string())
        }
    }

    /// Creates a sample starter schematic in `HorizonMatica/schematics/` if the folder is empty.
    fn ensure_sample_schematic(&self) {
        let sample_file = self.schematics_dir().join("sample_shelter.litematic");
        if !sample_file.exists() {
            let mut schem = LitematicaSchematic::new("Sample Shelter", "VkYakee");
            let mut sub = SubRegion::new(BlockPos::ZERO, BlockPos::new(5, 4, 5));

            let oak_planks = BlockState::from_string_repr("minecraft:oak_planks");
            let glass = BlockState::from_string_repr("minecraft:glass");
            let oak_log = BlockState::from_string_repr("minecraft:oak_log[axis=y]");

            // Build a small 5x4x5 shelter structure
            for x in 0..5 {
                for z in 0..5 {
                    // Floor
                    sub.container.set(x, 0, z, oak_planks.clone());
                    // Roof
                    sub.container.set(x, 3, z, oak_planks.clone());

                    // Walls and corners
                    if (x == 0 || x == 4) && (z == 0 || z == 4) {
                        sub.container.set(x, 1, z, oak_log.clone());
                        sub.container.set(x, 2, z, oak_log.clone());
                    } else if x == 0 || x == 4 || z == 0 || z == 4 {
                        if (x == 2 || z == 2) && !(x == 2 && z == 0) {
                            sub.container.set(x, 1, z, glass.clone());
                            sub.container.set(x, 2, z, glass.clone());
                        } else if !(x == 2 && z == 0) {
                            sub.container.set(x, 1, z, oak_planks.clone());
                            sub.container.set(x, 2, z, oak_planks.clone());
                        }
                    }
                }
            }

            schem.regions.insert("Main".to_string(), sub);
            let _ = schem.write_to_file(&sample_file);
        }
    }

    // ==========================================
    // VkMap Storage & World Separation Operations
    // ==========================================

    /// Saves all VkMap world data (waypoints, structures, annotations) into `/sdcard/VkYakeeMods/VkMap/worlds/{world_id}/`.
    pub fn save_vkmap_world_data(
        &self,
        world_id: &str,
        waypoints: &vkmap::WaypointManager,
        structures: &vkmap::StructureBridge,
        annotations: &vkmap::AnnotationManager,
    ) -> Result<(), String> {
        let world_dir = self.vkmap_world_dir(world_id);

        let wp_path = world_dir.join("waypoints.json");
        let wp_json = serde_json::to_string_pretty(waypoints).map_err(|e| e.to_string())?;
        fs::write(&wp_path, wp_json).map_err(|e| e.to_string())?;

        let st_path = world_dir.join("structures.json");
        let st_json = serde_json::to_string_pretty(structures).map_err(|e| e.to_string())?;
        fs::write(&st_path, st_json).map_err(|e| e.to_string())?;

        let an_path = world_dir.join("annotations.json");
        let an_json = serde_json::to_string_pretty(annotations).map_err(|e| e.to_string())?;
        fs::write(&an_path, an_json).map_err(|e| e.to_string())?;

        log::info!("VkYakeeMods: VkMap datos guardados para mundo '{}'", world_id);
        Ok(())
    }

    /// Loads world data for a specific world_id if exists.
    pub fn load_vkmap_world_data(
        &self,
        world_id: &str,
    ) -> (Option<vkmap::WaypointManager>, Option<vkmap::StructureBridge>, Option<vkmap::AnnotationManager>) {
        let world_dir = self.vkmap_world_dir(world_id);

        let waypoints = if let Ok(s) = fs::read_to_string(world_dir.join("waypoints.json")) {
            serde_json::from_str::<vkmap::WaypointManager>(&s).ok()
        } else {
            None
        };

        let structures = if let Ok(s) = fs::read_to_string(world_dir.join("structures.json")) {
            serde_json::from_str::<vkmap::StructureBridge>(&s).ok()
        } else {
            None
        };

        let annotations = if let Ok(s) = fs::read_to_string(world_dir.join("annotations.json")) {
            serde_json::from_str::<vkmap::AnnotationManager>(&s).ok()
        } else {
            None
        };

        (waypoints, structures, annotations)
    }

    /// Exports full high-resolution PNG map to `/sdcard/VkYakeeMods/VkMap/exports/`.
    pub fn export_vkmap_png(
        &self,
        world_id: &str,
        dimension: &vkmap::Dimension,
        image: &vkmap::ImageBuffer,
    ) -> Result<PathBuf, String> {
        vkmap::MapExporter::export_png(self.vkmap_exports_dir(), world_id, dimension, image)
            .map_err(|e| format!("Error exportando mapa PNG: {}", e))
    }

    /// Exports structured JSON dataset to `/sdcard/VkYakeeMods/VkMap/exports/`.
    pub fn export_vkmap_json(
        &self,
        world_id: &str,
        dimension: &vkmap::Dimension,
        structures: &vkmap::StructureBridge,
        waypoints: &vkmap::WaypointManager,
        annotations: &vkmap::AnnotationManager,
    ) -> Result<PathBuf, String> {
        vkmap::MapExporter::export_json(
            self.vkmap_exports_dir(),
            world_id,
            dimension,
            structures,
            waypoints,
            annotations,
        )
        .map_err(|e| format!("Error exportando mapa JSON: {}", e))
    }

    /// Writes a mod-generated export into the shared VkYakeeMods exports directory.
    pub fn save_file(&self, file_name: &str, bytes: &[u8]) -> Result<PathBuf, String> {
        let safe_name = Path::new(file_name).file_name()
            .ok_or_else(|| "Nombre de archivo inválido".to_string())?;
        let path = self.base_dir.join("exports").join(safe_name);
        if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
        fs::write(&path, bytes).map_err(|e| format!("No se pudo guardar {:?}: {}", path, e))?;
        Ok(path)
    }

}
