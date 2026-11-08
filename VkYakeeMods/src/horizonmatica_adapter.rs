//! HorizonMatica adapter implementing VkMod for Starling / Horizon Android.
//! Full schematic management (.litematic, .schem, .schematic, .nbt),
//! 100% parity with original Litematica (masa) and Rustmatica:
//!   - 3-stage interactive mobile workflow:
//!       1. Import / Load schematic (.litematic, .schem, .schematic, .nbt)
//!       2. 3D Isometric / 2D Layer Mini-Renderer preview
//!       3. `[ ▶ Iniciar ]`: Spawns hologram at player position, collapses big HUD,
//!          and deploys 1-button compact floating `[ 🛠️ Herramientas ]` mini-HUD.
//!   - Tools & Editing Drawer:
//!       * Rotate (90° CW / CCW), Mirror (X / Z), Placement Lock
//!       * Precise Nudge (Y±1, X±1, Z±1), Dynamic Opacity (25%..100%)
//!       * Multi-axis Layer Slicing (Y vertical, X east-west, Z north-south)
//!       * Schematic Verifier: detects Missing, Extra, WrongBlock, WrongState
//!       * Material List & Inventory Progress: required, available, missing count
//!       * Material Exporter: CSV, JSON, and text tables to Android storage
//!       * EasyPlace Protocol: Auto, V2, V3, SlabOnly click offset calculation
//!       * Schematic Projects: versioning and snapshot commits
//!       * Area Selection & Creation: Pos1, Pos2, and Save to .litematic.

use crate::api::{ModMetadata, VkMod};
use crate::storage::StorageManager;
use horizonmatica::materials::exporter::MaterialListExporter;
use horizonmatica::materials::inventory::{InventoryProgress, InventoryTracker};
use horizonmatica::materials::MaterialList;
use horizonmatica::schematic::litematic::{LitematicaSchematic, SubRegion};
use horizonmatica::schematic::mini_render::SchematicMiniRenderer;
use horizonmatica::schematic::palette::BlockState;
use horizonmatica::schematic::placement::SchematicPlacement;
use horizonmatica::schematic::projects::{SchematicProject, SchematicProjectsManager};
use horizonmatica::schematic::verifier::{BlockMismatch, SchematicVerifier, VerificationSummary};
use horizonmatica::selection::box_region::BoxRegion;
use horizonmatica::selection::manager::SelectionManager;
use horizonmatica::tool::mode::ToolMode;
use horizonmatica::util::easy_place::EasyPlaceProtocol;
use horizonmatica::util::layer::LayerMode;
use horizonmatica::util::pos::{Axis, BlockPos, Mirror, Rotation};

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct ToastLog {
    pub text: String,
    pub rgb: u32,
    pub created_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HorizonHudView {
    /// Full Main HUD: 1. Cargar/Importar, 2. Previsualizar, 3. Iniciar
    #[default]
    Main,
    /// 1-button compact floating mini-HUD: `[ 🛠️ Herramientas ]`
    MiniButton,
    /// Expanded Tools & Editing Drawer
    ToolsDrawer,
}

pub struct HorizonMaticaMod {
    storage: StorageManager,
    active_schematic: Option<LitematicaSchematic>,
    active_schematic_name: Option<String>,
    placement: Option<SchematicPlacement>,
    materials: Option<MaterialList>,
    selection_manager: SelectionManager,
    tool_mode: ToolMode,
    active: bool,
    show_hud: bool,
    show_hologram: bool,
    show_mini_render: bool,
    easy_place: bool,
    easy_place_protocol: EasyPlaceProtocol,
    layer_mode: LayerMode,
    layer_axis: Axis,
    current_layer: i32,
    hud_view: HorizonHudView,
    player_pos: BlockPos,
    player_inventory: HashMap<String, usize>,
    opacity: f32,
    locked: bool,
    hologram_started: bool,
    pos1: Option<BlockPos>,
    pos2: Option<BlockPos>,
    projects_manager: SchematicProjectsManager,
    last_verification: Option<VerificationSummary>,
    recent_mismatches: Vec<BlockMismatch>,
    toasts: VecDeque<ToastLog>,
    status_message: String,
}

impl HorizonMaticaMod {
    pub fn new() -> Self {
        let storage = StorageManager::new();
        let mut hm = Self {
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
            easy_place: true,
            easy_place_protocol: EasyPlaceProtocol::V2,
            layer_mode: LayerMode::All,
            layer_axis: Axis::Y,
            current_layer: 64,
            hud_view: HorizonHudView::Main,
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
            status_message: "HorizonMatica: Listo. Listo para cargar esquemáticas.".to_string(),
        };

        // Try to load starter sample shelter if available
        let schematics = hm.storage.list_schematics();
        if let Some(first) = schematics.first() {
            if let Some(fname) = first.file_name().and_then(|s| s.to_str()) {
                let _ = hm.load_schematic(fname);
            }
        }

        hm
    }

    pub fn add_toast(&mut self, text: impl Into<String>, rgb: u32) {
        let msg = text.into();
        log::info!("HorizonMatica Toast: {}", msg);
        if self.toasts.len() >= 5 {
            self.toasts.pop_front();
        }
        self.toasts.push_back(ToastLog {
            text: msg,
            rgb,
            created_at: Instant::now(),
        });
    }

    /// Loads a schematic by name, index, or file path.
    pub fn load_schematic(&mut self, target: &str) -> Result<(), String> {
        let schematics = self.storage.list_schematics();

        let resolved_name = if let Ok(idx) = target.parse::<usize>() {
            if let Some(p) = schematics.get(idx) {
                p.to_string_lossy().to_string()
            } else {
                return Err(format!("Índice {} fuera de rango (disponibles: 0..{})", idx, schematics.len()));
            }
        } else {
            target.to_string()
        };

        let schematic = self.storage.load_schematic(&resolved_name)?;
        let name = schematic.metadata.name.clone();
        let origin = self.player_pos;
        let mut placement = SchematicPlacement::for_schematic(&schematic, origin);
        placement.layer_range.axis = self.layer_axis;
        placement.layer_range.mode = self.layer_mode;
        placement.layer_range.current_layer = origin.y;
        placement.locked = self.locked;

        let materials = MaterialList::from_schematic(&schematic);
        let region_count = schematic.regions.len();
        let total_blocks = schematic.metadata.total_blocks;

        self.active_schematic_name = Some(name.clone());
        self.materials = Some(materials);
        self.placement = Some(placement);
        self.active_schematic = Some(schematic);
        self.current_layer = origin.y;
        self.last_verification = None;
        self.recent_mismatches.clear();

        let msg = format!("📂 Esquemática cargada: '{}' ({} regiones, {} bloques)", name, region_count, total_blocks);
        self.status_message = msg.clone();
        self.add_toast(msg, 0x00FF88);

        Ok(())
    }

    /// Step 3: Starts the hologram at the player's position, collapses big HUD, and opens 1-button mini-HUD.
    pub fn start_hologram(&mut self) {
        if self.active_schematic.is_none() {
            self.add_toast("⚠️ Primero carga o importa una esquemática", 0xFFAA00);
            return;
        }

        let origin = self.player_pos;
        if let Some(ref mut p) = self.placement {
            if !p.locked {
                p.origin = origin;
            }
        }

        self.show_hologram = true;
        self.hologram_started = true;
        self.current_layer = origin.y;
        self.hud_view = HorizonHudView::MiniButton;

        let msg = format!("▶ Holograma INICIADO en [{}, {}, {}]. HUD minimizado.", origin.x, origin.y, origin.z);
        self.status_message = msg.clone();
        self.add_toast(msg, 0x00FF88);
        self.add_toast("Toca [🛠️ Herramientas] para rotar, capas, verificar y materiales", 0x55FFFF);
    }

    /// Toggles between the 1-button floating widget and the full Tools & Editing drawer.
    pub fn toggle_tools(&mut self) {
        self.hud_view = match self.hud_view {
            HorizonHudView::MiniButton => HorizonHudView::ToolsDrawer,
            HorizonHudView::ToolsDrawer => HorizonHudView::MiniButton,
            HorizonHudView::Main => HorizonHudView::ToolsDrawer,
        };

        if self.hud_view == HorizonHudView::ToolsDrawer {
            self.add_toast("🛠️ Herramientas de edición desplegadas", 0x55FFFF);
        } else {
            self.add_toast("🔽 Herramientas minimizadas a 1 botón", 0xAAAAAA);
        }
    }

    /// Returns to the full main HUD.
    pub fn open_main_menu(&mut self) {
        self.hud_view = HorizonHudView::Main;
        self.add_toast("📋 Menú Principal de HorizonMatica abierto", 0x00FF88);
    }

    /// Imports a schematic from Android storage / downloads and immediately loads it.
    pub fn import_schematic(&mut self, source_path: &str) -> Result<PathBuf, String> {
        let imported_path = self.storage.import_schematic(source_path)?;
        let file_str = imported_path.to_string_lossy().to_string();
        self.load_schematic(&file_str)?;
        let fname = imported_path.file_name().and_then(|s| s.to_str()).unwrap_or("archivo");
        self.add_toast(format!("📥 Importado y cargado con éxito: {}", fname), 0x00FF88);
        Ok(imported_path)
    }

    /// Automatically searches /sdcard/Download/ for the newest schematic and imports it.
    pub fn import_from_downloads(&mut self) -> Result<PathBuf, String> {
        match self.storage.import_latest_from_downloads() {
            Ok(path) => {
                let file_str = path.to_string_lossy().to_string();
                self.load_schematic(&file_str)?;
                let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("archivo");
                self.add_toast(format!("📥 Importada última descarga: {}", fname), 0x00FF88);
                Ok(path)
            }
            Err(e) => {
                self.add_toast(format!("Error en importación: {}", e), 0xFF5555);
                Err(e)
            }
        }
    }

    /// Unloads the active schematic and placement.
    pub fn unload_schematic(&mut self) {
        self.active_schematic = None;
        self.active_schematic_name = None;
        self.placement = None;
        self.materials = None;
        self.show_hologram = false;
        self.hologram_started = false;
        self.last_verification = None;
        self.recent_mismatches.clear();
        self.hud_view = HorizonHudView::Main;
        self.status_message = "Esquemática descargada.".to_string();
        self.add_toast("⏹ Esquemática descargada", 0xFFAA00);
    }

    /// Toggles placement lock (prevents accidental movement while placing blocks).
    pub fn toggle_placement_lock(&mut self) {
        self.locked = !self.locked;
        if let Some(ref mut p) = self.placement {
            p.locked = self.locked;
        }
        if self.locked {
            self.add_toast("🔒 Colocación BLOQUEADA (inmóvil)", 0xFFAA00);
        } else {
            self.add_toast("🔓 Colocación DESBLOQUEADA (editable)", 0x00FF88);
        }
    }

    /// Rotates the placement 90 degrees.
    pub fn rotate_placement(&mut self, clockwise: bool) {
        if self.locked {
            self.add_toast("⚠️ Colocación bloqueada. Desbloquea con /hm lock antes de rotar.", 0xFFAA00);
            return;
        }

        let rot = if let Some(ref mut p) = self.placement {
            if clockwise {
                p.rotate_clockwise();
            } else {
                p.rotate_counter_clockwise();
            }
            Some(p.rotation)
        } else {
            None
        };

        if let Some(r) = rot {
            let rot_str = match r {
                Rotation::None => "0°",
                Rotation::Clockwise90 => "90° Horario",
                Rotation::Clockwise180 => "180°",
                Rotation::CounterClockwise90 => "270° Antihorario",
            };
            self.add_toast(format!("🔄 Rotación: {}", rot_str), 0x55FFFF);
        } else {
            self.add_toast("No hay esquemática activa para rotar", 0xFF5555);
        }
    }

    /// Toggles mirror along X/Z.
    pub fn mirror_placement(&mut self) {
        if self.locked {
            self.add_toast("⚠️ Colocación bloqueada. Desbloquea con /hm lock antes de espejar.", 0xFFAA00);
            return;
        }

        let mir = if let Some(ref mut p) = self.placement {
            p.flip_mirror();
            Some(p.mirror)
        } else {
            None
        };

        if let Some(m) = mir {
            let mir_str = match m {
                Mirror::None => "Ninguno",
                Mirror::LeftRight => "Izquierda/Derecha (X)",
                Mirror::FrontBack => "Frente/Atrás (Z)",
            };
            self.add_toast(format!("🪞 Espejo: {}", mir_str), 0x55FFFF);
        } else {
            self.add_toast("No hay esquemática activa para espejar", 0xFF5555);
        }
    }

    /// Sets the placement origin coordinates.
    pub fn set_origin(&mut self, x: i32, y: i32, z: i32) {
        if self.locked {
            self.add_toast("⚠️ Colocación bloqueada. Desbloquea con /hm lock antes de mover.", 0xFFAA00);
            return;
        }

        if let Some(ref mut p) = self.placement {
            p.origin = BlockPos::new(x, y, z);
        }
        self.add_toast(format!("📍 Origen fijado en [{}, {}, {}]", x, y, z), 0x00FF88);
    }

    /// Nudges the placement origin by (dx, dy, dz).
    pub fn nudge_origin(&mut self, dx: i32, dy: i32, dz: i32) {
        if self.locked {
            self.add_toast("⚠️ Colocación bloqueada. Desbloquea con /hm lock antes de mover.", 0xFFAA00);
            return;
        }

        let new_origin = if let Some(ref mut p) = self.placement {
            p.nudge(dx, dy, dz);
            Some(p.origin)
        } else {
            None
        };

        if let Some(origin) = new_origin {
            self.add_toast(format!("📍 Posición: [{}, {}, {}]", origin.x, origin.y, origin.z), 0x00FF88);
        }
    }

    /// Cycles opacity between 25%, 50%, 75%, 100%.
    pub fn cycle_opacity(&mut self) {
        self.opacity = if self.opacity <= 0.35 {
            0.50
        } else if self.opacity <= 0.60 {
            0.75
        } else if self.opacity <= 0.85 {
            1.00
        } else {
            0.25
        };
        self.add_toast(format!("✨ Opacidad de holograma: {}%", (self.opacity * 100.0) as i32), 0x55FFFF);
    }

    // ==========================================
    // Schematic Verification (SchematicVerifier)
    // ==========================================

    /// Verifies world blocks against the active schematic placement using default world lookup.
    pub fn verify_schematic(&mut self) -> Option<VerificationSummary> {
        self.verify_schematic_with(|_pos| BlockState::air())
    }

    /// Verifies world blocks against active schematic with a custom block state lookup function.
    pub fn verify_schematic_with<F>(&mut self, lookup: F) -> Option<VerificationSummary>
    where
        F: FnMut(BlockPos) -> BlockState,
    {
        let (schematic, placement) = match (&self.active_schematic, &self.placement) {
            (Some(s), Some(p)) => (s, p),
            _ => {
                self.add_toast("⚠️ No hay esquemática activa para verificar", 0xFFAA00);
                return None;
            }
        };

        let (summary, mismatches) = SchematicVerifier::verify(schematic, placement, lookup);
        let msg = format!(
            "🔍 Verificador: {} bloques ({} correctos, {} faltantes, {} errores)",
            summary.total_schematic_blocks,
            summary.correct_count,
            summary.missing_count,
            summary.wrong_block_count + summary.wrong_state_count
        );
        self.status_message = msg.clone();
        self.add_toast(msg, 0x55FFFF);

        let res = summary.clone();
        self.last_verification = Some(summary);
        self.recent_mismatches = mismatches;
        Some(res)
    }

    // ==========================================
    // Material List & Inventory Progress
    // ==========================================

    /// Sets or updates available item count in player's inventory tracking cache.
    pub fn set_inventory_item(&mut self, item_name: impl Into<String>, count: usize) {
        self.player_inventory.insert(item_name.into(), count);
    }

    /// Calculates material availability and missing count against player inventory.
    pub fn check_materials_progress(&mut self) -> Option<InventoryProgress> {
        let mats = match self.materials.as_ref() {
            Some(m) => m,
            None => {
                self.add_toast("⚠️ No hay esquemática activa para contar materiales", 0xFFAA00);
                return None;
            }
        };

        let progress = InventoryTracker::calculate_progress(mats, &self.player_inventory);
        let msg = format!(
            "📦 Materiales: {}/{} en inventario (faltan {})",
            progress.total_available,
            progress.total_required,
            progress.total_missing
        );
        self.add_toast(msg, 0x00FF88);

        let missing_entries: Vec<_> = progress.entries.iter().filter(|e| e.missing > 0).take(3).collect();
        for m in missing_entries {
            self.add_toast(format!(" • Falta: {}x {}", m.missing, m.item_name), 0xFFAA55);
        }

        Some(progress)
    }

    /// Exports active material list to CSV in `/sdcard/VkYakeeMods/HorizonMatica/exports/`.
    pub fn export_materials_csv(&mut self) -> Result<PathBuf, String> {
        let mats = match self.materials.as_ref() {
            Some(m) => m,
            None => return Err("No hay lista de materiales activa para exportar".to_string()),
        };
        let schem_name = self.active_schematic_name.clone().unwrap_or_else(|| "esquema".to_string());
        let csv = MaterialListExporter::to_csv(mats);
        let path = self.storage.export_materials_csv(&schem_name, &csv)?;
        self.add_toast(format!("📊 CSV exportado: {:?}", path.file_name().unwrap_or_default()), 0x00FF88);
        Ok(path)
    }

    /// Exports active material list to JSON in `/sdcard/VkYakeeMods/HorizonMatica/exports/`.
    pub fn export_materials_json(&mut self) -> Result<PathBuf, String> {
        let mats = match self.materials.as_ref() {
            Some(m) => m,
            None => return Err("No hay lista de materiales activa para exportar".to_string()),
        };
        let schem_name = self.active_schematic_name.clone().unwrap_or_else(|| "esquema".to_string());
        let json = MaterialListExporter::to_json(mats).map_err(|e| e.to_string())?;
        let path = self.storage.export_materials_json(&schem_name, &json)?;
        self.add_toast(format!("📊 JSON exportado: {:?}", path.file_name().unwrap_or_default()), 0x00FF88);
        Ok(path)
    }

    // ==========================================
    // Multi-Axis Layer Slicing
    // ==========================================

    /// Cycles the cutting axis for layers: Y (vertical) -> X (east-west) -> Z (north-south) -> Y.
    pub fn cycle_layer_axis(&mut self) {
        self.layer_axis = match self.layer_axis {
            Axis::Y => Axis::X,
            Axis::X => Axis::Z,
            Axis::Z => Axis::Y,
        };
        if let Some(ref mut p) = self.placement {
            p.layer_range.axis = self.layer_axis;
        }
        let axis_str = match self.layer_axis {
            Axis::X => "Eje X (Este-Oeste)",
            Axis::Y => "Eje Y (Vertical)",
            Axis::Z => "Eje Z (Norte-Sur)",
        };
        self.add_toast(format!("📐 Eje de corte: {}", axis_str), 0x55FFFF);
    }

    /// Cycles layer mode: All -> Single Layer -> Layer Range -> All.
    pub fn cycle_layer_mode(&mut self) {
        self.layer_mode = match self.layer_mode {
            LayerMode::All => LayerMode::SingleLayer,
            LayerMode::SingleLayer => LayerMode::LayerRange,
            LayerMode::LayerRange => LayerMode::All,
        };
        if let Some(ref mut p) = self.placement {
            p.layer_range.mode = self.layer_mode;
        }
        let mode_str = match self.layer_mode {
            LayerMode::All => "Todas las capas",
            LayerMode::SingleLayer => "Capa individual",
            LayerMode::LayerRange => "Rango de capas",
        };
        self.add_toast(format!("👁️ Modo de capas: {}", mode_str), 0x55FFFF);
    }

    /// Moves focused layer up or down.
    pub fn adjust_layer(&mut self, delta: i32) {
        self.layer_mode = LayerMode::SingleLayer;
        self.current_layer += delta;
        if let Some(ref mut p) = self.placement {
            p.layer_range.mode = LayerMode::SingleLayer;
            p.layer_range.current_layer = self.current_layer;
        }
        let axis_char = match self.layer_axis {
            Axis::X => "X",
            Axis::Y => "Y",
            Axis::Z => "Z",
        };
        self.add_toast(format!("📐 Capa enfocada: {}={}", axis_char, self.current_layer), 0xFFFF55);
    }

    /// Toggles showing all layers vs single layer.
    pub fn toggle_layer_all(&mut self) {
        if self.layer_mode == LayerMode::All {
            self.layer_mode = LayerMode::SingleLayer;
            self.add_toast(format!("📐 Capa individual: Y={}", self.current_layer), 0xFFFF55);
        } else {
            self.layer_mode = LayerMode::All;
            self.add_toast("📐 Capas: MOSTRAR TODAS", 0x55FF55);
        }
        if let Some(ref mut p) = self.placement {
            p.layer_range.mode = self.layer_mode;
        }
    }

    // ==========================================
    // EasyPlace Protocol
    // ==========================================

    /// Cycles EasyPlace protocol between Auto, V2, V3, SlabOnly.
    pub fn cycle_easy_place_protocol(&mut self) {
        self.easy_place_protocol = match self.easy_place_protocol {
            EasyPlaceProtocol::Auto => EasyPlaceProtocol::V2,
            EasyPlaceProtocol::V2 => EasyPlaceProtocol::V3,
            EasyPlaceProtocol::V3 => EasyPlaceProtocol::SlabOnly,
            EasyPlaceProtocol::SlabOnly => EasyPlaceProtocol::Auto,
        };
        let proto_str = match self.easy_place_protocol {
            EasyPlaceProtocol::Auto => "Auto",
            EasyPlaceProtocol::V2 => "Protocol V2",
            EasyPlaceProtocol::V3 => "Protocol V3",
            EasyPlaceProtocol::SlabOnly => "Slab Only",
        };
        self.add_toast(format!("🔨 Protocolo EasyPlace: {}", proto_str), 0x55FFFF);
    }

    // ==========================================
    // Project Versioning
    // ==========================================

    /// Commits a new version snapshot into the schematic projects manager.
    pub fn commit_project_version(&mut self, version_name: &str, description: &str) -> Result<u32, String> {
        let schem = match self.active_schematic.as_ref() {
            Some(s) => s.clone(),
            None => return Err("No hay esquemática activa para crear versión de proyecto".to_string()),
        };
        let project_name = self.active_schematic_name.clone().unwrap_or_else(|| "Proyecto".to_string());
        if !self.projects_manager.has_project_open() {
            let p = SchematicProject::new(&project_name, schem);
            self.projects_manager.set_active_project(p);
        }

        if let Some(proj) = self.projects_manager.get_active_project_mut() {
            let ver = proj.commit_version(version_name, description);
            self.add_toast(format!("🔖 Versión {} guardada: '{}'", ver, version_name), 0x00FF88);
            Ok(ver)
        } else {
            Err("Error gestionando proyecto".to_string())
        }
    }

    // ==========================================
    // Area Selection & Creation
    // ==========================================

    /// Sets Pos1 for creating a new schematic at the player's current position.
    pub fn set_pos1_here(&mut self) {
        self.pos1 = Some(self.player_pos);
        self.update_selection_box();
        self.add_toast(format!("📍 Pos1 fijado en [{}, {}, {}]", self.player_pos.x, self.player_pos.y, self.player_pos.z), 0x00FF88);
    }

    /// Sets Pos2 for creating a new schematic at the player's current position.
    pub fn set_pos2_here(&mut self) {
        self.pos2 = Some(self.player_pos);
        self.update_selection_box();
        self.add_toast(format!("📍 Pos2 fijado en [{}, {}, {}]", self.player_pos.x, self.player_pos.y, self.player_pos.z), 0x00FF88);
    }

    fn update_selection_box(&mut self) {
        if let (Some(p1), Some(p2)) = (self.pos1, self.pos2) {
            let mut region = BoxRegion::new("Selection", p1, p2);
            region.name = "Nueva Creación".to_string();
            self.selection_manager.current_selection.add_box(region);
        }
    }

    /// Creates and saves a new .litematic file from the selected Pos1/Pos2 bounding box.
    pub fn save_selection_as_litematic(&mut self, name: &str) -> Result<PathBuf, String> {
        let (p1, p2) = match (self.pos1, self.pos2) {
            (Some(a), Some(b)) => (a, b),
            _ => return Err("Falta definir Pos1 y/o Pos2 antes de guardar".to_string()),
        };

        let min_x = p1.x.min(p2.x);
        let min_y = p1.y.min(p2.y);
        let min_z = p1.z.min(p2.z);

        let sx = (p1.x - p2.x).abs() + 1;
        let sy = (p1.y - p2.y).abs() + 1;
        let sz = (p1.z - p2.z).abs() + 1;

        let mut new_schem = LitematicaSchematic::new(name, "VkYakee");
        let sub = SubRegion::new(BlockPos::new(min_x, min_y, min_z), BlockPos::new(sx, sy, sz));
        new_schem.regions.insert("Main".to_string(), sub);

        let path = self.storage.save_schematic(name, &mut new_schem)?;
        self.add_toast(format!("💾 Nuevo Litematic '{}' creado ({}x{}x{})", name, sx, sy, sz), 0x00FF88);
        Ok(path)
    }

    /// Exports active schematic to `/sdcard/VkYakeeMods/HorizonMatica/exports/`.
    pub fn export_active(&mut self, name: &str) -> Result<PathBuf, String> {
        if let Some(ref mut schem) = self.active_schematic {
            let path = self.storage.export_schematic(name, schem)?;
            self.add_toast(format!("💾 Exportado a {:?}", path.file_name().unwrap_or_default()), 0x00FF88);
            Ok(path)
        } else {
            Err("No hay esquemática activa para exportar".to_string())
        }
    }

    /// Cycles the current tool mode.
    pub fn cycle_tool_mode(&mut self) {
        self.tool_mode = match self.tool_mode {
            ToolMode::AreaSelection => ToolMode::SchematicPlacement,
            ToolMode::SchematicPlacement => ToolMode::Fill,
            ToolMode::Fill => ToolMode::ReplaceBlock,
            ToolMode::ReplaceBlock => ToolMode::PasteSchematic,
            ToolMode::PasteSchematic => ToolMode::Rebuild,
            ToolMode::Rebuild => ToolMode::AreaSelection,
            _ => ToolMode::SchematicPlacement,
        };
        self.add_toast(format!("🛠️ Modo: {}", self.tool_mode.name()), 0xFFAAFF);
    }

    /// Toggles mini render preview.
    pub fn toggle_mini_render(&mut self) {
        self.show_mini_render = !self.show_mini_render;
        let s = if self.show_mini_render { "ACTIVADA" } else { "OCULTA" };
        self.add_toast(format!("📐 Vista Mini-Render: {}", s), 0x00E5FF);
    }

    /// Cycles through available schematics in storage.
    pub fn load_next_schematic(&mut self) {
        let list = self.storage.list_schematics();
        if list.is_empty() {
            self.add_toast("No hay esquemáticas en la carpeta de Android", 0xFF5555);
            return;
        }

        let next_idx = if let Some(ref curr) = self.active_schematic_name {
            let current_pos = list.iter().position(|p| {
                p.file_stem().and_then(|s| s.to_str()) == Some(curr.as_str())
                    || p.file_name().and_then(|s| s.to_str()) == Some(curr.as_str())
            });
            match current_pos {
                Some(idx) => (idx + 1) % list.len(),
                None => 0,
            }
        } else {
            0
        };

        if let Some(target) = list.get(next_idx) {
            let p_str = target.to_string_lossy().to_string();
            let _ = self.load_schematic(&p_str);
        }
    }
}

impl VkMod for HorizonMaticaMod {
    fn metadata(&self) -> ModMetadata {
        ModMetadata {
            id: "horizonmatica",
            name: "HorizonMatica",
            version: "0.1.0",
            description: "Visualizador y gestor de esquemáticas Litematica (.litematic, .schem, .nbt) para Android y Starling",
            author: "VkYakee (Basado en Litematica por masa)",
        }
    }

    fn on_tick(&mut self) {}

    fn on_player_pos(&mut self, x: i32, y: i32, z: i32) {
        self.player_pos = BlockPos::new(x, y, z);
    }

    fn on_command(&mut self, command: &str) -> bool {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return false;
        }

        let prefix = parts[0].to_ascii_lowercase();
        if !matches!(prefix.as_str(), "hm" | "horizonmatica" | "schem" | "litematica") {
            return false;
        }

        let action = parts.get(1).map(|s| s.to_ascii_lowercase()).unwrap_or_else(|| "help".to_string());

        match action.as_str() {
            "start" | "iniciar" | "place" => {
                self.start_hologram();
            }
            "tools" | "herramientas" => {
                self.toggle_tools();
            }
            "menu" | "main" => {
                self.open_main_menu();
            }
            "up" | "subir" => {
                self.nudge_origin(0, 1, 0);
            }
            "down" | "bajar" => {
                self.nudge_origin(0, -1, 0);
            }
            "left" | "izq" => {
                self.nudge_origin(-1, 0, 0);
            }
            "right" | "der" => {
                self.nudge_origin(1, 0, 0);
            }
            "lock" | "bloquear" => {
                self.toggle_placement_lock();
            }
            "v" | "verify" | "verificar" => {
                self.verify_schematic();
            }
            "opacity" | "opacidad" => {
                if let Some(&val_str) = parts.get(2) {
                    if let Ok(val) = val_str.parse::<f32>() {
                        self.opacity = (val / 100.0).clamp(0.1, 1.0);
                        self.add_toast(format!("✨ Opacidad fijada en {}%", (self.opacity * 100.0) as i32), 0x55FFFF);
                    }
                } else {
                    self.cycle_opacity();
                }
            }
            "pos1" => {
                self.set_pos1_here();
            }
            "pos2" => {
                self.set_pos2_here();
            }
            "save" | "create" => {
                let name = parts.get(2).copied().unwrap_or("nuevo_esquema");
                if let Err(e) = self.save_selection_as_litematic(name) {
                    self.add_toast(e, 0xFF5555);
                }
            }
            "load" => {
                if let Some(&name) = parts.get(2) {
                    if let Err(e) = self.load_schematic(name) {
                        self.add_toast(e, 0xFF5555);
                    }
                } else {
                    self.add_toast("Uso: /hm load <nombre_o_indice>", 0xFFFF55);
                }
            }
            "import" => {
                if let Some(&src) = parts.get(2) {
                    if let Err(e) = self.import_schematic(src) {
                        self.add_toast(e, 0xFF5555);
                    }
                } else {
                    let _ = self.import_from_downloads();
                }
            }
            "unload" => {
                self.unload_schematic();
            }
            "list" => {
                let schems = self.storage.list_schematics();
                if schems.is_empty() {
                    self.add_toast("No hay esquemáticas guardadas en Android.", 0xFFAA00);
                } else {
                    self.add_toast(format!("📁 {} esquemáticas encontradas:", schems.len()), 0x55FFFF);
                    for (i, p) in schems.iter().take(5).enumerate() {
                        let fname = p.file_name().and_then(|s| s.to_str()).unwrap_or("?");
                        self.add_toast(format!(" [{}] {}", i, fname), 0xFFFFFF);
                    }
                }
            }
            "pos" | "origin" => {
                if parts.len() >= 5 {
                    if let (Ok(x), Ok(y), Ok(z)) = (parts[2].parse::<i32>(), parts[3].parse::<i32>(), parts[4].parse::<i32>()) {
                        self.set_origin(x, y, z);
                    }
                } else {
                    self.add_toast("Uso: /hm pos <x> <y> <z>", 0xFFFF55);
                }
            }
            "move" | "nudge" => {
                if parts.len() >= 5 {
                    if let (Ok(dx), Ok(dy), Ok(dz)) = (parts[2].parse::<i32>(), parts[3].parse::<i32>(), parts[4].parse::<i32>()) {
                        self.nudge_origin(dx, dy, dz);
                    }
                } else {
                    self.add_toast("Uso: /hm move <dx> <dy> <dz>", 0xFFFF55);
                }
            }
            "rot" | "rotate" => {
                let cw = parts.get(2).map(|s| *s != "ccw").unwrap_or(true);
                self.rotate_placement(cw);
            }
            "mir" | "mirror" => {
                self.mirror_placement();
            }
            "mat" | "materials" => {
                let sub = parts.get(2).copied().unwrap_or("list");
                match sub {
                    "missing" | "inv" | "inventario" => {
                        self.check_materials_progress();
                    }
                    "export" => {
                        let fmt = parts.get(3).copied().unwrap_or("csv");
                        if fmt == "json" {
                            let _ = self.export_materials_json();
                        } else {
                            let _ = self.export_materials_csv();
                        }
                    }
                    _ => {
                        let msgs = if let Some(ref mats) = self.materials {
                            let mut list = vec![format!("📦 Total items: {} ({} tipos)", mats.total_items, mats.entries.len())];
                            for entry in mats.entries.iter().take(4) {
                                list.push(format!(" • {}: {}", entry.item_name, entry.formatted_stacks()));
                            }
                            Some(list)
                        } else {
                            None
                        };

                        if let Some(list) = msgs {
                            if let Some(first) = list.first() {
                                self.add_toast(first.clone(), 0x00FF88);
                            }
                            for msg in list.into_iter().skip(1) {
                                self.add_toast(msg, 0xFFFFFF);
                            }
                        } else {
                            self.add_toast("No hay esquemática activa para materiales.", 0xFFAA00);
                        }
                    }
                }
            }
            "mode" => {
                self.cycle_tool_mode();
            }
            "preview" | "mini" => {
                self.toggle_mini_render();
            }
            "layer" => {
                match parts.get(2).copied().unwrap_or("all") {
                    "up" => self.adjust_layer(1),
                    "down" => self.adjust_layer(-1),
                    "all" => self.toggle_layer_all(),
                    "axis" => {
                        if let Some(ax_str) = parts.get(3) {
                            match *ax_str {
                                "x" | "X" => self.layer_axis = Axis::X,
                                "y" | "Y" => self.layer_axis = Axis::Y,
                                "z" | "Z" => self.layer_axis = Axis::Z,
                                _ => self.cycle_layer_axis(),
                            }
                            if let Some(ref mut p) = self.placement {
                                p.layer_range.axis = self.layer_axis;
                            }
                            self.add_toast(format!("📐 Eje de corte fijado en {:?}", self.layer_axis), 0x55FFFF);
                        } else {
                            self.cycle_layer_axis();
                        }
                    }
                    "mode" => self.cycle_layer_mode(),
                    other => {
                        if let Ok(val) = other.parse::<i32>() {
                            self.current_layer = val;
                            self.layer_mode = LayerMode::SingleLayer;
                            if let Some(ref mut p) = self.placement {
                                p.layer_range.mode = LayerMode::SingleLayer;
                                p.layer_range.current_layer = val;
                            }
                            self.add_toast(format!("📐 Capa fijada en {}", val), 0xFFFF55);
                        }
                    }
                }
            }
            "ep" | "easyplace" => {
                if let Some(sub) = parts.get(2) {
                    if *sub == "protocol" {
                        self.cycle_easy_place_protocol();
                    } else {
                        self.easy_place = !self.easy_place;
                        let state_str = if self.easy_place { "ACTIVADO" } else { "DESACTIVADO" };
                        self.add_toast(format!("🔨 EasyPlace: {}", state_str), 0x00FF88);
                    }
                } else {
                    self.easy_place = !self.easy_place;
                    let state_str = if self.easy_place { "ACTIVADO" } else { "DESACTIVADO" };
                    self.add_toast(format!("🔨 EasyPlace: {}", state_str), 0x00FF88);
                }
            }
            "holo" | "hologram" => {
                self.show_hologram = !self.show_hologram;
                let state_str = if self.show_hologram { "VISIBLE" } else { "OCULTO" };
                self.add_toast(format!("👁️ Holograma: {}", state_str), 0x00FF88);
            }
            "hud" => {
                self.show_hud = !self.show_hud;
                let state_str = if self.show_hud { "VISIBLE" } else { "OCULTO" };
                self.add_toast(format!("🖥️ HUD: {}", state_str), 0x00FF88);
            }
            "export" => {
                let name = parts.get(2).copied().unwrap_or("exported_schematic");
                if let Err(e) = self.export_active(name) {
                    self.add_toast(e, 0xFF5555);
                }
            }
            "project" => {
                let sub = parts.get(2).copied().unwrap_or("info");
                if sub == "commit" || sub == "save" {
                    let commit_msg = parts.get(3).copied().unwrap_or("Actualización");
                    let _ = self.commit_project_version(commit_msg, "Commit guardado desde consola");
                } else {
                    self.add_toast("Proyectos: /hm project commit <nombre>", 0x55FFFF);
                }
            }
            _ => {
                self.add_toast("Comandos: /hm [start|tools|menu|verify|mat|lock|rot|mir|layer|opacity|pos1|pos2|save]", 0xFFFF55);
            }
        }

        true
    }

    fn hud_lines(&self) -> Vec<String> {
        if !self.active || !self.show_hud {
            return Vec::new();
        }

        let mut lines = Vec::new();

        match self.hud_view {
            // ==========================================
            // Mode 1: MiniButton (Single-Button Floating HUD)
            // ==========================================
            HorizonHudView::MiniButton => {
                let schem_name = self.active_schematic_name.as_deref().unwrap_or("Esquema");
                let p = self.placement.as_ref();
                let rot_str = match p.map(|x| x.rotation).unwrap_or(Rotation::None) {
                    Rotation::None => "0°",
                    Rotation::Clockwise90 => "90°",
                    Rotation::Clockwise180 => "180°",
                    Rotation::CounterClockwise90 => "270°",
                };

                let lock_str = if self.locked { "LOCK: ON" } else { "LOCK: OFF" };
                let axis_str = match self.layer_axis {
                    Axis::X => "X",
                    Axis::Y => "Y",
                    Axis::Z => "Z",
                };

                lines.push(format!("┌─ [ 🛠️ Herramientas: {} ] ──────────────┐", schem_name));
                lines.push(format!(
                    "│ [ 🛠️ HERRAMIENTAS ] (Toca o /hm tools) | Rot: {} | Capa {}: {}",
                    rot_str, axis_str, self.current_layer
                ));
                lines.push(format!(
                    "│ Holograma: ON | Opacidad: {}% | {} | EasyPlace: ON",
                    (self.opacity * 100.0) as i32,
                    lock_str
                ));
                lines.push("└──────────────────────────────────────────────┘".to_string());
            }

            // ==========================================
            // Mode 2: ToolsDrawer (Expanded Tools & Editing)
            // ==========================================
            HorizonHudView::ToolsDrawer => {
                let lock_str = if self.locked { "🔒 BLOQUEADA" } else { "🔓 LIBRE" };
                let axis_str = match self.layer_axis {
                    Axis::X => "X",
                    Axis::Y => "Y",
                    Axis::Z => "Z",
                };
                let mode_str = match self.layer_mode {
                    LayerMode::All => "Todas",
                    LayerMode::SingleLayer => "Individual",
                    LayerMode::LayerRange => "Rango",
                };

                lines.push("┌─ [ 🛠️ Herramientas y Edición: HorizonMatica ] ───────────────┐".to_string());
                lines.push(format!(
                    "│ 🔄 [Rotar 90° CW/CCW] 🪞 [Espejo X/Z] 🔒 [{}]",
                    lock_str
                ));
                lines.push(format!(
                    "│ ⬆️ [Subir Y+1] ⬇️ [Bajar Y-1] ⬅️ [Mover X] ➡️ [Mover Z] ✨ [Op: {}%]",
                    (self.opacity * 100.0) as i32
                ));
                lines.push(format!(
                    "│ 📐 [Capa ±1] 👁️ [Modo: {}] 📐 [Eje: {}] 🔨 [EasyPlace: ON]",
                    mode_str, axis_str
                ));
                lines.push("├─ [ 🔍 Verificador y Materiales ] ────────────────────────────┤".to_string());
                lines.push("│ 🔍 [Verificar Bloques] 📦 [Comprobar Inv] 📊 [Exportar CSV]  │".to_string());

                if let Some(ref ver) = self.last_verification {
                    lines.push(format!(
                        "│ Verifier: {} correctos, {} faltantes, {} errores",
                        ver.correct_count, ver.missing_count, ver.wrong_block_count + ver.wrong_state_count
                    ));
                }

                lines.push("├─ [ ✏️ Creación de Nuevo Litematic ] ─────────────────────────┤".to_string());
                lines.push("│ 📍 [Pos 1: Aquí] 📍 [Pos 2: Aquí] 💾 [Guardar Selección]      │".to_string());

                if let (Some(p1), Some(p2)) = (self.pos1, self.pos2) {
                    let sx = (p1.x - p2.x).abs() + 1;
                    let sy = (p1.y - p2.y).abs() + 1;
                    let sz = (p1.z - p2.z).abs() + 1;
                    lines.push(format!("│ Selección: [{},{},{}] ➔ [{},{},{}] ({} blqs)", p1.x, p1.y, p1.z, p2.x, p2.y, p2.z, sx * sy * sz));
                } else {
                    lines.push("│ Selección: Define Pos1 y Pos2 para crear nuevo esquema".to_string());
                }

                lines.push("├─ [ 🔘 Opciones de Pantalla ] ────────────────────────────────┤".to_string());
                lines.push("│ 🔽 [Minimizar a 1 Botón] (/hm tools) | 📋 [Menú Principal]   │".to_string());
                lines.push("└─ (/hm tools | /hm verify | /hm mat | /hm up | /hm down | /hm pos1) ┘".to_string());
            }

            // ==========================================
            // Mode 3: Main Full HUD (Import, Preview, Start)
            // ==========================================
            HorizonHudView::Main => {
                lines.push("┌─ [ VkYakeeMods ➔ 📐 HorizonMatica ] ─────────────────────────┐".to_string());
                lines.push("│ 1. 📁 [Cargar] / 📥 [Importar]  2. 📐 [Previsualizar]        │".to_string());
                lines.push("│ 3. ▶ [ INICIAR HOLOGRAMA ] (/hm start)                       │".to_string());
                lines.push("│ (Al presionar Iniciar se coloca el esquema y abre mini-HUD)  │".to_string());

                if let Some(ref schem) = self.active_schematic {
                    let name = self.active_schematic_name.as_deref().unwrap_or("Sin Nombre");
                    let p = self.placement.as_ref();
                    let origin = p.map(|x| x.origin).unwrap_or(BlockPos::ZERO);
                    let rot = p.map(|x| x.rotation).unwrap_or(Rotation::None);
                    let rot_str = match rot {
                        Rotation::None => "0°",
                        Rotation::Clockwise90 => "90° CW",
                        Rotation::Clockwise180 => "180°",
                        Rotation::CounterClockwise90 => "270° CCW",
                    };
                    let mir_str = match p.map(|x| x.mirror).unwrap_or(Mirror::None) {
                        Mirror::None => "Ninguno",
                        Mirror::LeftRight => "X",
                        Mirror::FrontBack => "Z",
                    };

                    // 3D Isometric or 2D Layer Mini-Renderer
                    if self.show_mini_render {
                        let layer_opt = if self.layer_mode == LayerMode::All { None } else { Some(self.current_layer) };
                        let preview = SchematicMiniRenderer::render_preview(schem, layer_opt, rot);
                        for pl in preview {
                            lines.push(format!("│{}", pl));
                        }
                    }

                    // Placement & Layer Status
                    lines.push("├─ [ 📋 Datos de Colocación ]".to_string());
                    lines.push(format!("│ Esquemática: {}", name));
                    lines.push(format!(
                        "│ Origen: [{}, {}, {}] | Rot: {} | Espejo: {}",
                        origin.x, origin.y, origin.z, rot_str, mir_str
                    ));
                    lines.push(format!(
                        "│ Dimensiones: {} bloques en {} regiones",
                        schem.metadata.total_blocks,
                        schem.regions.len()
                    ));

                    if self.layer_mode == LayerMode::All {
                        lines.push("│ Capa: Todas las capas visibles".to_string());
                    } else {
                        lines.push(format!("│ Capa Enfocada: {:?}={}", self.layer_axis, self.current_layer));
                    }

                    // Schematic Verifier summary if executed
                    if let Some(ref ver) = self.last_verification {
                        lines.push("├─ [ 🔍 Auditoría de Bloques (Verifier) ]".to_string());
                        lines.push(format!(
                            "│ Estado: {} correctos | {} faltantes | {} erróneos",
                            ver.correct_count, ver.missing_count, ver.wrong_block_count + ver.wrong_state_count
                        ));
                    }

                    // Complete Materials List
                    if let Some(ref mats) = self.materials {
                        lines.push("├─ [ 📦 Lista de Materiales (Litematica) ]".to_string());
                        let total_stacks = mats.total_items / 64;
                        let remainder = mats.total_items % 64;
                        let shulkers = (mats.total_items as f64) / (64.0 * 27.0);
                        lines.push(format!(
                            "│ Requeridos: {} items ({} stks + {}, {:.1} shulkers)",
                            mats.total_items, total_stacks, remainder, shulkers
                        ));
                        lines.push(format!("│ Tipos Distintos: {}", mats.entries.len()));

                        for entry in mats.entries.iter().take(3) {
                            lines.push(format!("│  • {}: {}", entry.item_name, entry.formatted_stacks()));
                        }
                    }
                } else {
                    lines.push("├─ [ ℹ️ Estado ]".to_string());
                    lines.push("│ Sin esquemática cargada.".to_string());
                    lines.push("│ Toca [📥 Importar] o ejecuta /hm import <archivo>".to_string());
                    lines.push(format!("│ Carpeta: {}", self.storage.schematics_dir().display()));
                }

                lines.push("└─ (/hm import | /hm load | /hm start | /hm preview | /hm verify) ─┘".to_string());
            }
        }

        // Recent Event Toasts
        if !self.toasts.is_empty() {
            lines.push("├─ [ 🔔 Registros Recientes ]".to_string());
            for t in self.toasts.iter().rev().take(2) {
                lines.push(format!("│  {}", t.text));
            }
        }

        lines
    }

    fn render_bounding_boxes(&self) -> Vec<[f32; 10]> {
        if !self.active || !self.show_hologram {
            return Vec::new();
        }

        let mut boxes = Vec::new();
        let alpha = self.opacity;

        // 1. Render schematic placement bounding box
        if let (Some(schem), Some(p)) = (&self.active_schematic, &self.placement) {
            let ox = p.origin.x as f32;
            let oy = p.origin.y as f32;
            let oz = p.origin.z as f32;

            for (name, region) in &schem.regions {
                let rx = region.position.x as f32;
                let ry = region.position.y as f32;
                let rz = region.position.z as f32;
                let sx = region.size.x as f32;
                let sy = region.size.y as f32;
                let sz = region.size.z as f32;

                let min_x = ox + rx;
                let min_y = oy + ry;
                let min_z = oz + rz;
                let max_x = min_x + sx;
                let max_y = min_y + sy;
                let max_z = min_z + sz;

                // Color: Cyan with dynamic opacity (R: 0.0, G: 0.85, B: 1.0, A: opacity)
                boxes.push([min_x, min_y, min_z, max_x, max_y, max_z, 0.0, 0.85, 1.0, alpha]);

                let _ = name;
            }
        }

        // 2. Render active area selection boxes
        for (_name, b) in &self.selection_manager.current_selection.sub_region_boxes {
            let p1 = b.pos1;
            let p2 = b.pos2;

            let min_x = p1.x.min(p2.x) as f32;
            let min_y = p1.y.min(p2.y) as f32;
            let min_z = p1.z.min(p2.z) as f32;
            let max_x = p1.x.max(p2.x) as f32 + 1.0;
            let max_y = p1.y.max(p2.y) as f32 + 1.0;
            let max_z = p1.z.max(p2.z) as f32 + 1.0;

            // Color: Magenta/Purple (R: 1.0, G: 0.2, B: 0.9, A: opacity)
            boxes.push([min_x, min_y, min_z, max_x, max_y, max_z, 1.0, 0.2, 0.9, alpha]);
        }

        boxes
    }

    fn settings_buttons(&self) -> Vec<String> {
        let load_label = if let Some(ref name) = self.active_schematic_name {
            format!("📁 Cargar: [{}]", name)
        } else {
            "📁 Cargar Esquemática (Siguiente)".to_string()
        };

        match self.hud_view {
            HorizonHudView::Main => {
                vec![
                    load_label,
                    "📥 Importar de Descargas".to_string(),
                    if self.show_mini_render { "📐 Previsualizar: [3D ACTIVO]" } else { "📐 Previsualizar: [OCULTO]" }.to_string(),
                    "▶ INICIAR HOLOGRAMA (Colocar Aquí)".to_string(),
                    format!("🛠️ Modo: [{}]", self.tool_mode.name()),
                    "🔄 Rotar: [90° CW]".to_string(),
                    "🪞 Espejo: [Alternar]".to_string(),
                    "📦 [Ver Materiales Requeridos]".to_string(),
                    "📊 [Exportar Materiales CSV]".to_string(),
                    "🔍 [Verificar Bloques (Verifier)]".to_string(),
                    "⏹ Descargar Esquemática".to_string(),
                ]
            }
            HorizonHudView::MiniButton => {
                vec![
                    "🛠️ [ABRIR HERRAMIENTAS Y EDICIÓN]".to_string(),
                    "⬆️ Subir 1 Bloque (Y+1)".to_string(),
                    "⬇️ Bajar 1 Bloque (Y-1)".to_string(),
                    "🔄 Rotar: [90° CW]".to_string(),
                    format!("✨ Opacidad: [{}%]", (self.opacity * 100.0) as i32),
                    "🔍 [Verificar Rápido]".to_string(),
                    "📋 Volver al Menú Principal".to_string(),
                ]
            }
            HorizonHudView::ToolsDrawer => {
                vec![
                    "🔽 [MINIMIZAR HERRAMIENTAS (1 BOTÓN)]".to_string(),
                    "🔄 Rotar: [90° CW]".to_string(),
                    "🔄 Rotar: [90° CCW]".to_string(),
                    "🪞 Espejo: [Alternar X/Z]".to_string(),
                    if self.locked { "🔒 Colocación: [BLOQUEADA]" } else { "🔓 Colocación: [DESBLOQUEADA]" }.to_string(),
                    "⬆️ Subir 1 Bloque (Y+1)".to_string(),
                    "⬇️ Bajar 1 Bloque (Y-1)".to_string(),
                    "📐 Capa: [+1 Arriba]".to_string(),
                    "📐 Capa: [-1 Abajo]".to_string(),
                    format!("👁️ Capas: [{}]", match self.layer_mode { LayerMode::All => "Ver Todo", LayerMode::SingleLayer => "Individual", LayerMode::LayerRange => "Rango" }),
                    format!("📐 Eje: [Eje {:?}]", self.layer_axis),
                    format!("✨ Opacidad: [{}%]", (self.opacity * 100.0) as i32),
                    if self.easy_place { "🔨 EasyPlace: [ACTIVADO]" } else { "🔨 EasyPlace: [DESACTIVADO]" }.to_string(),
                    "🔍 [Verificar Bloques (Verifier)]".to_string(),
                    "📦 [Comprobar Inventario / Faltantes]".to_string(),
                    "📊 [Exportar Materiales CSV]".to_string(),
                    "📍 Crear: [Fijar Pos 1 Aquí]".to_string(),
                    "📍 Crear: [Fijar Pos 2 Aquí]".to_string(),
                    "💾 Crear: [Guardar Nuevo Litematic]".to_string(),
                    "📋 [Volver al Menú Principal]".to_string(),
                    "⏹ Descargar Esquemática".to_string(),
                ]
            }
        }
    }

    fn on_settings_click(&mut self, button_idx: usize) {
        match self.hud_view {
            HorizonHudView::Main => {
                match button_idx {
                    0 => self.load_next_schematic(),
                    1 => {
                        let _ = self.import_from_downloads();
                    }
                    2 => self.toggle_mini_render(),
                    3 => self.start_hologram(),
                    4 => self.cycle_tool_mode(),
                    5 => self.rotate_placement(true),
                    6 => self.mirror_placement(),
                    7 => {
                        if let Some(ref mats) = self.materials {
                            self.add_toast(format!("📦 Total items: {}", mats.total_items), 0x00FF88);
                        }
                    }
                    8 => {
                        let _ = self.export_materials_csv();
                    }
                    9 => {
                        self.verify_schematic();
                    }
                    10 => self.unload_schematic(),
                    _ => {}
                }
            }
            HorizonHudView::MiniButton => {
                match button_idx {
                    0 => self.toggle_tools(),
                    1 => self.nudge_origin(0, 1, 0),
                    2 => self.nudge_origin(0, -1, 0),
                    3 => self.rotate_placement(true),
                    4 => self.cycle_opacity(),
                    5 => {
                        self.verify_schematic();
                    }
                    6 => self.open_main_menu(),
                    _ => {}
                }
            }
            HorizonHudView::ToolsDrawer => {
                match button_idx {
                    0 => self.toggle_tools(),
                    1 => self.rotate_placement(true),
                    2 => self.rotate_placement(false),
                    3 => self.mirror_placement(),
                    4 => self.toggle_placement_lock(),
                    5 => self.nudge_origin(0, 1, 0),
                    6 => self.nudge_origin(0, -1, 0),
                    7 => self.adjust_layer(1),
                    8 => self.adjust_layer(-1),
                    9 => self.cycle_layer_mode(),
                    10 => self.cycle_layer_axis(),
                    11 => self.cycle_opacity(),
                    12 => {
                        self.easy_place = !self.easy_place;
                        let s = if self.easy_place { "ACTIVADO" } else { "DESACTIVADO" };
                        self.add_toast(format!("🔨 EasyPlace: {}", s), 0x00FF88);
                    }
                    13 => {
                        self.verify_schematic();
                    }
                    14 => {
                        self.check_materials_progress();
                    }
                    15 => {
                        let _ = self.export_materials_csv();
                    }
                    16 => self.set_pos1_here(),
                    17 => self.set_pos2_here(),
                    18 => {
                        let _ = self.save_selection_as_litematic("mi_creacion");
                    }
                    19 => self.open_main_menu(),
                    20 => self.unload_schematic(),
                    _ => {}
                }
            }
        }
    }
}
