//! VkMod implementation for Starling / Horizon Android integration

use crate::prelude::*;
use crate::config::VlaKeeMaticaConfig;
use crate::schematic::litematic::LitematicaSchematic;
use crate::schematic::placement::{SchematicPlacement, SchematicPlacementManager};
use crate::schematic::verifier::{SchematicVerifier, BlockMismatch, VerificationSummary};
use crate::schematic::projects::{SchematicProject, SchematicProjectsManager, ProjectVersion};
use crate::materials::{MaterialList, MaterialListExporter};
use crate::materials::inventory::{InventoryTracker, InventoryProgress};
use crate::selection::{SelectionManager, BoxRegion, AreaSelection};
use crate::tool::ToolMode;
use crate::util::easy_place::EasyPlaceProtocol;
use crate::util::layer::{LayerMode, LayerRange};
use crate::util::pos::{BlockPos, Rotation, Mirror, Axis};

use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;

/// Toast notification for UI
#[derive(Clone, Debug)]
pub struct ToastLog {
    pub text: String,
    pub rgb: u32,
    pub created_at: Instant,
}

/// HUD view state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VlaKeeHudView {
    #[default]
    Main,
    MiniButton,
    ToolsDrawer,
}

/// Main mod struct implementing VkMod
pub struct VlaKeeMaticaMod {
    // Storage
    storage: StorageManager,
    
    // Schematic state
    active_schematic: Option<LitematicaSchematic>,
    active_schematic_name: Option<String>,
    placement: Option<SchematicPlacement>,
    materials: Option<MaterialList>,
    
    // Selection
    selection_manager: SelectionManager,
    
    // Tool state
    tool_mode: ToolMode,
    
    // UI state
    active: bool,
    show_hud: bool,
    show_hologram: bool,
    show_mini_render: bool,
    hud_view: VlaKeeHudView,
    
    // Placement settings
    easy_place: bool,
    easy_place_protocol: EasyPlaceProtocol,
    layer_mode: LayerMode,
    layer_axis: Axis,
    current_layer: i32,
    
    // Player state
    player_pos: BlockPos,
    player_inventory: HashMap<String, usize>,
    
    // Visual settings
    opacity: f32,
    locked: bool,
    
    // Session state
    hologram_started: bool,
    pos1: Option<BlockPos>,
    pos2: Option<BlockPos>,
    
    // Projects
    projects_manager: SchematicProjectsManager,
    
    // Verification
    last_verification: Option<VerificationSummary>,
    recent_mismatches: Vec<BlockMismatch>,
    
    // Notifications
    toasts: VecDeque<ToastLog>,
    status_message: String,
}

impl Default for VlaKeeMaticaMod {
    fn default() -> Self {
        Self::new()
    }
}

impl VlaKeeMaticaMod {
    pub fn new() -> Self {
        let storage = StorageManager::new();
        
        Self {
            storage,
            active_schematic: None,
            active_schematic_name: None,
            placement: None,
            materials: None,
            selection_manager: SelectionManager::new(),
            tool_mode: ToolMode::SchematicPlacement,
            active: true,
            show_hud: true,
            show_hologram: false,
            show_mini_render: true,
            hud_view: VlaKeeHudView::Main,
            easy_place: true,
            easy_place_protocol: EasyPlaceProtocol::V2,
            layer_mode: LayerMode::All,
            layer_axis: Axis::Y,
            current_layer: 64,
            player_pos: BlockPos::new(0, 64, 0),
            player_inventory: HashMap::new(),
            opacity: 0.75,
            locked: false,
            hologram_started: false,
            pos1: None,
            pos2: None,
            projects_manager: SchematicProjectsManager::new(),
            last_verification: None,
            recent_mismatches: Vec::new(),
            toasts: VecDeque::new(),
            status_message: "VlaKeeMatica: Listo. Carga una esquemática.".to_string(),
        }
    }
    
    // --- Schematic Loading ---
    
    pub fn load_next_schematic(&mut self) {
        self.add_toast("🔄 Cargando siguiente esquemática...".to_string(), 0x00FF88);
    }
    
    pub fn import_from_downloads(&mut self) -> Result<(), String> {
        self.add_toast("📁 Importando desde descargas...".to_string(), 0x00FF88);
        // Would use StorageManager to find and load .litematic files
        Ok(())
    }
    
    pub fn load_schematic(&mut self, path: &str, name: &str) -> Result<(), String> {
        let schematic = LitematicaSchematic::read_from_file(path)
            .map_err(|e| format!("Error cargando esquemática: {}", e))?;
        
        let placement = SchematicPlacement::for_schematic(&schematic, self.player_pos);
        let materials = MaterialList::from_schematic(&schematic);
        
        self.active_schematic = Some(schematic);
        self.active_schematic_name = Some(name.to_string());
        self.placement = Some(placement);
        self.materials = Some(materials);
        self.hologram_started = false;
        self.show_hologram = false;
        
        self.add_toast(format!("✅ Cargado: {}", name), 0x00FF00);
        Ok(())
    }
    
    pub fn unload_schematic(&mut self) {
        self.active_schematic = None;
        self.active_schematic_name = None;
        self.placement = None;
        self.materials = None;
        self.show_hologram = false;
        self.hologram_started = false;
        self.add_toast("🗑️ Esquemática descargada".to_string(), 0xFFAA00);
    }
    
    // --- Schematic Placement ---
    
    pub fn toggle_hologram(&mut self) {
        self.show_hologram = !self.show_hologram;
        if self.show_hologram {
            self.hologram_started = true;
        }
        self.add_toast(format!("📐 Holograma: {}", if self.show_hologram { "ON" } else { "OFF" }), 0x00FFAA);
    }
    
    pub fn start_hologram_at_player(&mut self) {
        if let Some(ref mut placement) = self.placement {
            placement.origin = self.player_pos;
            self.show_hologram = true;
            self.hologram_started = true;
            self.add_toast("▶ Holograma iniciado en jugador".to_string(), 0x00FF00);
        }
    }
    
    pub fn rotate_schematic_cw(&mut self) {
        if let Some(ref mut placement) = self.placement {
            if !self.locked {
                placement.rotate_clockwise();
                self.add_toast("🔄 Rotado 90° CW".to_string(), 0x00FFAA);
            } else {
                self.add_toast("🔒 Bloqueado - desbloquea primero".to_string(), 0xFF8800);
            }
        }
    }
    
    pub fn flip_mirror(&mut self) {
        if let Some(ref mut placement) = self.placement {
            if !self.locked {
                placement.flip_mirror();
                self.add_toast("🪞 Espejo alternado".to_string(), 0x00FFAA);
            } else {
                self.add_toast("🔒 Bloqueado - desbloquea primero".to_string(), 0xFF8800);
            }
        }
    }
    
    pub fn lock_hologram(&mut self) {
        self.locked = !self.locked;
        self.add_toast(format!("🔒 Bloqueo: {}", if self.locked { "ON" } else { "OFF" }), 0xAA00FF);
    }
    
    pub fn nudge_up(&mut self) {
        if let Some(ref mut placement) = self.placement {
            if !self.locked {
                placement.nudge(0, 1, 0);
            }
        }
    }
    
    pub fn nudge_down(&mut self) {
        if let Some(ref mut placement) = self.placement {
            if !self.locked {
                placement.nudge(0, -1, 0);
            }
        }
    }
    
    pub fn nudge_north(&mut self) {
        if let Some(ref mut placement) = self.placement {
            if !self.locked {
                placement.nudge(0, 0, -1);
            }
        }
    }
    
    pub fn nudge_south(&mut self) {
        if let Some(ref mut placement) = self.placement {
            if !self.locked {
                placement.nudge(0, 0, 1);
            }
        }
    }
    
    pub fn nudge_east(&mut self) {
        if let Some(ref mut placement) = self.placement {
            if !self.locked {
                placement.nudge(1, 0, 0);
            }
        }
    }
    
    pub fn nudge_west(&mut self) {
        if let Some(ref mut placement) = self.placement {
            if !self.locked {
                placement.nudge(-1, 0, 0);
            }
        }
    }
    
    // --- Tool Modes ---
    
    pub fn cycle_tool_mode(&mut self) {
        self.tool_mode = self.tool_mode.cycle_next();
        self.add_toast(format!("🛠️ Herramienta: {:?}", self.tool_mode), 0xAAAAFF);
    }
    
    // --- Layer Slicing ---
    
    pub fn cycle_layer_axis(&mut self) {
        self.layer_axis = match self.layer_axis {
            Axis::Y => Axis::X,
            Axis::X => Axis::Z,
            Axis::Z => Axis::Y,
        };
        self.add_toast(format!("📐 Eje de capa: {:?}", self.layer_axis), 0x00FFAA);
    }
    
    pub fn layer_up(&mut self) {
        self.current_layer += 1;
        self.add_toast(format!("📏 Capa: {}", self.current_layer), 0x00FFAA);
    }
    
    pub fn layer_down(&mut self) {
        self.current_layer -= 1;
        self.add_toast(format!("📏 Capa: {}", self.current_layer), 0x00FFAA);
    }
    
    // --- Easy Place ---
    
    pub fn toggle_easy_place(&mut self) {
        self.easy_place = !self.easy_place;
        self.add_toast(format!("📋 EasyPlace: {}", if self.easy_place { "ON" } else { "OFF" }), 0x00FFAA);
    }
    
    pub fn cycle_easy_place_protocol(&mut self) {
        self.easy_place_protocol = self.easy_place_protocol.next();
        self.add_toast(format!("📋 Protocolo: {:?}", self.easy_place_protocol), 0x00FFAA);
    }
    
    // --- Materials ---
    
    pub fn show_materials(&mut self) {
        if let Some(ref materials) = self.materials {
            self.add_toast(format!("📦 Materiales: {} tipos, {} totales", 
                materials.entries.len(), materials.total_items), 0xFFAA00);
        }
    }
    
    pub fn export_materials_csv(&mut self) -> Result<(), String> {
        if let Some(ref materials) = self.materials {
            let csv = MaterialListExporter::to_csv(materials);
            let filename = format!("materials_{}.csv", 
                self.active_schematic_name.as_deref().unwrap_or("schematic"));
            self.storage.save_file(&filename, csv.as_bytes())?;
            self.add_toast(format!("💾 CSV exportado: {}", filename), 0x00FF00);
            Ok(())
        } else {
            Err("No hay esquemática cargada".to_string())
        }
    }
    
    // --- Verification ---
    
    pub fn verify_schematic(&mut self) {
        if let (Some(ref schematic), Some(ref placement)) = (&self.active_schematic, &self.placement) {
            // Use static verify method with world block lookup
            let (summary, mismatches) = SchematicVerifier::verify(schematic, placement, |_pos| {
                // Placeholder: would need actual world block lookup from game
                // For now, return air - this means all non-air blocks in schematic will show as missing
                BlockState::air()
            });
            self.last_verification = Some(summary.clone());
            self.recent_mismatches = mismatches;
            
            self.add_toast(format!(
                "🔍 Verificación: ✓{} ✗{} ⚠{} ⬜{}",
                summary.correct_count, summary.missing_count,
                summary.wrong_block_count, summary.extra_count
            ), if summary.missing_count > 0 { 0xFF0000 } else { 0x00FF00 });
        }
    }
    
    // --- Projects ---
    
    pub fn save_project(&mut self, name: &str) {
        if let Some(ref schematic) = self.active_schematic {
            if let Some(ref mut project) = self.projects_manager.get_active_project_mut() {
                project.commit_version(name, "Manual save");
                self.add_toast(format!("💾 Proyecto guardado: {}", name), 0x00FF00);
            } else {
                let new_project = SchematicProject::new(name, schematic.clone());
                self.projects_manager.set_active_project(new_project);
                self.add_toast(format!("💾 Proyecto creado: {}", name), 0x00FF00);
            }
        }
    }
    
    pub fn list_projects(&mut self) {
        let projects = self.projects_manager.list_versions();
        if projects.is_empty() {
            self.add_toast("📭 Sin proyectos guardados".to_string(), 0x888888);
        } else {
            for p in projects {
                self.add_toast(format!("📁 {}", p), 0xAAAAFF);
            }
        }
    }
    
    // --- Selection ---
    
    pub fn set_pos1(&mut self) {
        self.pos1 = Some(self.player_pos);
        self.add_toast(format!("📍 Pos1: {}", self.player_pos), 0x00FFFF);
    }
    
    pub fn set_pos2(&mut self) {
        self.pos2 = Some(self.player_pos);
        self.add_toast(format!("📍 Pos2: {}", self.player_pos), 0x00FFFF);
    }
    
    pub fn create_schematic_from_selection(&mut self) {
        if let (Some(pos1), Some(pos2)) = (self.pos1, self.pos2) {
            let mut region = AreaSelection::new("Region");
            region.add_box(BoxRegion::new("Box1", pos1, pos2));
            self.selection_manager.set_selection(region);
            self.add_toast("✅ Región seleccionada".to_string(), 0x00FF00);
        } else {
            self.add_toast("❌ Falta Pos1 o Pos2".to_string(), 0xFF0000);
        }
    }
    
    // --- Opacity ---
    
    pub fn increase_opacity(&mut self) {
        self.opacity = (self.opacity + 0.1).min(1.0);
        self.add_toast(format!("🔆 Opacidad: {:.0}%", self.opacity * 100.0), 0xFFAA00);
    }
    
    pub fn decrease_opacity(&mut self) {
        self.opacity = (self.opacity - 0.1).max(0.1);
        self.add_toast(format!("🔅 Opacidad: {:.0}%", self.opacity * 100.0), 0xFFAA00);
    }
    
    // --- Utility ---
    
    fn add_toast(&mut self, text: String, rgb: u32) {
        self.toasts.push_back(ToastLog {
            text,
            rgb,
            created_at: Instant::now(),
        });
        while self.toasts.len() > 10 {
            self.toasts.pop_front();
        }
    }
    
    // --- VkMod Interface Methods ---
    
    pub fn is_active(&self) -> bool {
        self.active
    }
    
    pub fn get_status(&self) -> String {
        self.status_message.clone()
    }
    
    pub fn get_hud_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        
        if self.active {
            lines.push("┌──── VlaKeeMatica ───────────────────────┐".to_string());
            lines.push(format!("│ Esquemática: {}", 
                self.active_schematic_name.as_deref().unwrap_or("Ninguna")));
            lines.push(format!("│ Holograma: {}  Mini: {}", 
                if self.show_hologram { "ON" } else { "OFF" },
                if self.show_mini_render { "ON" } else { "OFF" }));
            lines.push(format!("│ Herramienta: {:?}  Protocolo: {:?}", self.tool_mode, self.easy_place_protocol));
            lines.push(format!("│ Capa: {} (Eje: {:?})  Opacidad: {:.0}%", self.current_layer, self.layer_axis, self.opacity * 100.0));
            if let Some(ref v) = self.last_verification {
                lines.push(format!("│ Verif: ✓{} ✗{} ⚠{} ⬜{}",
                    v.correct_count, v.missing_count, v.wrong_block_count, v.extra_count));
            }
            lines.push("│ [L] Cargar  [H] Holograma  [T] Herramienta  [M] Menú".to_string());
            lines.push("└────────────────────────────────────────┘".to_string());
        } else {
            lines.push("VlaKeeMatica: Deshabilitado".to_string());
        }
        
        // Add recent toasts
        for toast in self.toasts.iter().rev().take(3) {
            lines.push(format!("  💬 {}", toast.text));
        }
        
        lines
    }
    
    pub fn render_bounding_boxes(&self) -> Vec<[f32; 10]> {
        if !self.show_hologram {
            return Vec::new();
        }
        
        let mut boxes = Vec::new();
        
        if let (Some(ref schematic), Some(ref placement)) = (&self.active_schematic, &self.placement) {
            for (region_name, region) in &schematic.regions {
                let sub_enabled = placement.sub_region_placements.get(region_name)
                    .map(|p| p.enabled)
                    .unwrap_or(true);
                
                if !sub_enabled {
                    continue;
                }
                
                let min = region.position;
                let max = min.offset(
                    region.size.x - 1,
                    region.size.y - 1,
                    region.size.z - 1,
                );
                
                let world_min = placement.get_transformed_position(region_name, min);
                let world_max = placement.get_transformed_position(region_name, max);
                
                // Main enclosing box
                boxes.push([
                    world_min.x as f32 - 0.5,
                    world_min.y as f32 - 0.5,
                    world_min.z as f32 - 0.5,
                    world_max.x as f32 + 0.5,
                    world_max.y as f32 + 0.5,
                    world_max.z as f32 + 0.5,
                    0.0, 1.0, 0.0, 0.5,
                ]);
            }
        }
        
        boxes
    }
    
    pub fn settings_buttons(&self) -> Vec<String> {
        vec![
            "📐 Cargar Esquemática".to_string(),
            "📁 Importar desde Descargas".to_string(),
            "🔄 Previsualizar 3D/2D".to_string(),
            "▶ Iniciar Holograma".to_string(),
            "🛠️ Modo Herramienta: Colocación".to_string(),
            "🔄 Rotar 90° CW".to_string(),
            "🪞 Espejo: Alternar".to_string(),
            "📦 Ver Materiales".to_string(),
            "📊 Exportar CSV".to_string(),
            "🔍 Verificar Bloques".to_string(),
            "⏹ Descargar Esquemática".to_string(),
            "⚙️ Configuración Avanzada".to_string(),
        ]
    }
    
    pub fn on_settings_click(&mut self, button_idx: usize) {
        match button_idx {
            0 => { self.load_next_schematic(); }
            1 => { let _ = self.import_from_downloads(); }
            2 => { self.show_mini_render = !self.show_mini_render; }
            3 => { self.toggle_hologram(); }
            4 => { self.cycle_tool_mode(); }
            5 => { self.rotate_schematic_cw(); }
            6 => { self.flip_mirror(); }
            7 => { self.show_materials(); }
            8 => { let _ = self.export_materials_csv(); }
            9 => { self.verify_schematic(); }
            10 => { self.unload_schematic(); }
            11 => { /* Advanced settings */ }
            _ => {}
        }
    }
    
    // --- Native Hook Handlers ---
    
    pub fn on_tick(&mut self, clock: &horizon_mods::Clock) {
        let _ = clock;
        // Update hologram position if following player
        if self.show_hologram && !self.locked {
            if let Some(ref mut _placement) = self.placement {
                // Could auto-follow player here
            }
        }
        
        // Update material progress if inventory changed
        if let Some(ref mut _mats) = self.materials {
            // This would be updated when player inventory changes
        }
    }
    
    pub fn on_player_pos(&mut self, x: i32, y: i32, z: i32) {
        self.player_pos = BlockPos::new(x, y, z);
    }
    
    pub fn on_block_update(&mut self, _x: i32, _y: i32, _z: i32, _block: Option<&str>) {
        // Real-time verification update
    }
    
    pub fn on_chunk_loaded(&mut self, _cx: i32, _cz: i32, _dimension: &str) {
        // Could auto-detect structures for schematic creation
    }
    
    pub fn on_command(&mut self, command: &str) -> bool {
        if command.starts_with("hm ") || command.starts_with("vlakeematica ") {
            let parts: Vec<&str> = command.split_whitespace().collect();
            if parts.len() < 2 {
                return true;
            }
            
            match parts[1] {
                "load" => { true }
                "unload" => {
                    self.unload_schematic();
                    true
                }
                "hologram" => {
                    self.toggle_hologram();
                    true
                }
                "verify" => {
                    self.verify_schematic();
                    true
                }
                "materials" => {
                    self.show_materials();
                    true
                }
                "pos1" => {
                    self.set_pos1();
                    true
                }
                "pos2" => {
                    self.set_pos2();
                    true
                }
                "lock" => {
                    self.lock_hologram();
                    true
                }
                "rotate" => {
                    self.rotate_schematic_cw();
                    true
                }
                "mirror" => {
                    self.flip_mirror();
                    true
                }
                "tool" => {
                    self.cycle_tool_mode();
                    true
                }
                "protocol" => {
                    self.cycle_easy_place_protocol();
                    true
                }
                "layer_up" => {
                    self.layer_up();
                    true
                }
                "layer_down" => {
                    self.layer_down();
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }
}

/// Storage manager for file operations
pub struct StorageManager {
    base_path: std::path::PathBuf,
}

impl StorageManager {
    pub fn new() -> Self {
        let base_path = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("Starling")
            .join("VlaKeeMatica");
        std::fs::create_dir_all(&base_path).ok();
        Self { base_path }
    }
    
    pub fn save_file(&self, name: &str, data: &[u8]) -> Result<std::path::PathBuf, String> {
        let path = self.base_path.join(name);
        std::fs::write(&path, data).map_err(|e| e.to_string())?;
        Ok(path)
    }
    
    pub fn load_file(&self, name: &str) -> Result<Vec<u8>, String> {
        let path = self.base_path.join(name);
        std::fs::read(&path).map_err(|e| e.to_string())
    }
    
    pub fn list_schematics(&self) -> Vec<std::path::PathBuf> {
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("litematic") {
                    files.push(path);
                }
            }
        }
        files.sort();
        files
    }
}

impl Default for StorageManager {
    fn default() -> Self {
        Self::new()
    }
}
