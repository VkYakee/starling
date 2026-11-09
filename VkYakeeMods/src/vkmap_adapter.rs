//! VkMap adapter implementing VkMod for Starling / Horizon Android.
//! Delivers high-performance tactical mapping, entity radar, waypoint navigation,
//! subterranean structure analysis (Trial Chambers, Ancient Cities), multi-world isolation,
//! bidirectional SeedCracker synergy, and Android local storage exports.

use crate::api::{ModMetadata, VkMod};
use crate::storage::StorageManager;
use vkmap::{
    Dimension, FacingDirection, ImageBuffer, LightingMode, MapEngine,
    MinimapDisplaySize, MinimapShape, RadarEntity, RadarVisibilityMode, RenderMode,
    StructureSource, SubterraneanStructureAnalyzer, TouchWidgetAction, TravelSpeed, Waypoint,
};

use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct ToastLog {
    pub text: String,
    pub rgb: u32,
    pub created_at: Instant,
}

/// Settings subpage within VkMap interactive menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VkMapPage {
    #[default]
    Navigation,
    Structures,
    Radar,
    Perspectives,
    ExportStorage,
}

impl VkMapPage {
    pub fn name(&self) -> &'static str {
        match self {
            VkMapPage::Navigation => "1/5: Navegación & Waypoints",
            VkMapPage::Structures => "2/5: Estructuras & Cuevas",
            VkMapPage::Radar => "3/5: Radar & Entidades",
            VkMapPage::Perspectives => "4/5: Perspectiva & Render",
            VkMapPage::ExportStorage => "5/5: Exportar & Android",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            VkMapPage::Navigation => VkMapPage::Structures,
            VkMapPage::Structures => VkMapPage::Radar,
            VkMapPage::Radar => VkMapPage::Perspectives,
            VkMapPage::Perspectives => VkMapPage::ExportStorage,
            VkMapPage::ExportStorage => VkMapPage::Navigation,
        }
    }
}

pub struct VkMapMod {
    pub engine: MapEngine,
    pub storage: StorageManager,
    pub current_world_id: String,
    pub current_dimension: Dimension,
    pub player_pos: (f64, f64, f64),
    pub player_yaw: f32,
    pub current_biome: String,
    pub active: bool,
    pub show_hud: bool,
    pub show_minimap: bool,
    pub page: VkMapPage,
    pub cave_mode_active: bool,
    pub cave_slice_y: i32,
    pub render_mode: RenderMode,
    pub status_message: String,
    pub toasts: VecDeque<ToastLog>,
    pub auto_save_counter: u64,
    pub simulated_entities: Vec<RadarEntity>,
    /// Latest world-seed candidate published by SeedCracker (not a structure seed).
    pub seed_candidate: Option<u64>,
}

impl Default for VkMapMod {
    fn default() -> Self {
        Self::new()
    }
}

impl VkMapMod {
    pub fn new() -> Self {
        let storage = StorageManager::new();
        let default_world_id = "default_session".to_string();
        let world_dir = storage.vkmap_world_dir(&default_world_id);
        let mut engine = MapEngine::new(&world_dir);

        // Load world data if available
        let (wps, st_opt, ann_opt) = storage.load_vkmap_world_data(&default_world_id);
        if let Some(w) = wps {
            engine.waypoint_manager = w;
        }
        if let Some(st) = st_opt {
            engine.structure_bridge = st;
        }
        if let Some(ann) = ann_opt {
            engine.annotation_manager = ann;
        }

        Self {
            engine,
            storage,
            current_world_id: default_world_id,
            current_dimension: Dimension::Overworld,
            player_pos: (0.0, 64.0, 0.0),
            player_yaw: 0.0,
            current_biome: "minecraft:plains".to_string(),
            active: true,
            show_hud: true,
            show_minimap: true,
            page: VkMapPage::Navigation,
            cave_mode_active: false,
            cave_slice_y: -16, // Default negative Y layer for Trial Chambers / Ancient Cities
            render_mode: RenderMode::TopDownShaded,
            status_message: "VkMap: Listo. Motor de mapa y radar activo.".to_string(),
            toasts: VecDeque::new(),
            auto_save_counter: 0,
            simulated_entities: Vec::new(),
            seed_candidate: None,
        }
    }

    pub fn add_toast(&mut self, text: impl Into<String>, rgb: u32) {
        let msg = text.into();
        log::info!("VkMap Toast: {}", msg);
        if self.toasts.len() >= 6 {
            self.toasts.pop_front();
        }
        self.toasts.push_back(ToastLog {
            text: msg,
            rgb,
            created_at: Instant::now(),
        });
    }

    /// Changes the current world/server session and cleanly isolates data.
    pub fn switch_world(&mut self, new_world_id: &str) {
        // Save old world first
        let _ = self.save_current_world();

        self.current_world_id = new_world_id.to_string();
        let world_dir = self.storage.vkmap_world_dir(new_world_id);
        self.engine = MapEngine::new(&world_dir);

        // Load world-specific state
        let (wps, st_opt, ann_opt) = self.storage.load_vkmap_world_data(new_world_id);
        if let Some(w) = wps {
            self.engine.waypoint_manager = w;
        }
        if let Some(st) = st_opt {
            self.engine.structure_bridge = st;
        }
        if let Some(ann) = ann_opt {
            self.engine.annotation_manager = ann;
        }

        let msg = format!("🌍 Mundo cambiado a '{}' (Datos recargados)", new_world_id);
        self.status_message = msg.clone();
        self.add_toast(msg, 0x55FF55);
    }

    /// Saves all current world data to `/sdcard/VkYakeeMods/VkMap/worlds/{world_id}/`.
    pub fn save_current_world(&self) -> Result<(), String> {
        self.storage.save_vkmap_world_data(
            &self.current_world_id,
            &self.engine.waypoint_manager,
            &self.engine.structure_bridge,
            &self.engine.annotation_manager,
        )
    }

    /// Adds a waypoint at the player's exact current location.
    pub fn create_waypoint_at_player(&mut self, name: &str, color: [u8; 3]) {
        let (px, py, pz) = self.player_pos;
        let wp = Waypoint::new(
            name,
            px.round() as i32,
            py.round() as i32,
            pz.round() as i32,
            self.current_dimension.clone(),
            color,
        );
        let msg = format!("📍 Waypoint '{}' creado en [{}, {}, {}]", wp.name, wp.x, wp.y, wp.z);
        self.engine.waypoint_manager.add(wp);
        self.add_toast(msg, 0x00FF88);
        let _ = self.save_current_world();
    }

    /// Records a death waypoint when the player dies.
    pub fn record_death_point(&mut self, x: i32, y: i32, z: i32) {
        let wp = self.engine.waypoint_manager.create_death_point(x, y, z, self.current_dimension.clone());
        let msg = format!("💀 Punto de muerte registrado en [{}, {}, {}]", wp.x, wp.y, wp.z);
        self.add_toast(msg, 0xFF4444);
        let _ = self.save_current_world();
    }

    /// Receives SeedCracker's progressive world-seed candidate publication.
    /// Do not fabricate structure locations from a hash/modulo heuristic: VkMap
    /// only draws SeedPrediction markers after a real Minecraft placement check.
    pub fn sync_predicted_structures(&mut self, candidates: &[u64]) {
        let Some(&seed) = candidates.first() else { return; };
        self.seed_candidate = Some(seed);
        let msg = format!("SeedCracker: candidato de mundo 0x{:016X} recibido; mapa esperando predicción validada", seed);
        self.status_message = msg.clone();
        self.add_toast(msg, 0xFFD700);
    }

    /// Exports full high-resolution PNG image map to `/sdcard/VkYakeeMods/VkMap/exports/`.
    pub fn export_png(&mut self) -> Result<PathBuf, String> {
        let (px, _, pz) = self.player_pos;
        let size = self.engine.minimap.config.display_size.pixel_size();
        let terrain = ImageBuffer::new_filled(size, size, [50, 120, 50, 255]);

        let frame = self.engine.generate_minimap_frame(
            &terrain,
            px,
            self.player_pos.1,
            pz,
            self.player_yaw,
            &self.current_dimension,
            &self.simulated_entities,
        );

        let path = self.storage.export_vkmap_png(
            &self.current_world_id,
            &self.current_dimension,
            &frame,
        )?;

        let msg = format!("🖼️ Mapa PNG exportado en: {:?}", path);
        self.status_message = msg.clone();
        self.add_toast("🖼️ Mapa exportado a PNG con éxito", 0x00FF88);
        Ok(path)
    }

    /// Exports full JSON dataset to `/sdcard/VkYakeeMods/VkMap/exports/`.
    pub fn export_json(&mut self) -> Result<PathBuf, String> {
        let path = self.storage.export_vkmap_json(
            &self.current_world_id,
            &self.current_dimension,
            &self.engine.structure_bridge,
            &self.engine.waypoint_manager,
            &self.engine.annotation_manager,
        )?;

        let msg = format!("📊 Datos JSON exportados en: {:?}", path);
        self.status_message = msg.clone();
        self.add_toast("📊 Datos exportados a JSON con éxito", 0x00FF88);
        Ok(path)
    }
}

impl VkMod for VkMapMod {
    fn metadata(&self) -> ModMetadata {
        ModMetadata {
            id: "vkmap",
            name: "VkMap (Horizon Rust)",
            version: "1.0.0",
            description: "Motor de mapa in-game y radar táctil para Starling Android, con soporte multiversión, detección subterránea y sinergia SeedCracker",
            author: "VkYakee & Map_Rust Team",
        }
    }

    fn on_tick(&mut self) {
        if !self.active {
            return;
        }

        // Auto-save world data every 600 ticks (30 seconds)
        self.auto_save_counter += 1;
        if self.auto_save_counter >= 600 {
            self.auto_save_counter = 0;
            let _ = self.save_current_world();
        }

        // Clean expired toasts after 5 seconds
        let now = Instant::now();
        while let Some(t) = self.toasts.front() {
            if now.duration_since(t.created_at).as_secs() >= 5 {
                self.toasts.pop_front();
            } else {
                break;
            }
        }
    }

    fn on_player_pos(&mut self, x: i32, y: i32, z: i32) {
        self.player_pos = (x as f64, y as f64, z as f64);
    }

    fn on_biome_found(&mut self, x: i32, z: i32, _biome_id: i32, name: &str) {
        let _ = (x, z);
        self.current_biome = name.to_string();
    }

    fn on_chunk_loaded(
        &mut self,
        cx: i32,
        cz: i32,
        dimension: &str,
        _bedrock_mask: Option<&[u8; 256]>,
    ) {
        let dim = match dimension {
            "minecraft:the_nether" | "the_nether" | "nether" => Dimension::Nether,
            "minecraft:the_end" | "the_end" | "end" => Dimension::TheEnd,
            _ => Dimension::Overworld,
        };
        self.current_dimension = dim.clone();

        // Subterranean Structure Analysis: check trial chambers / ancient cities
        if self.cave_mode_active && dim == Dimension::Overworld {
            let detected = SubterraneanStructureAnalyzer::detect_subterranean_chunk(cx, cz);
            if let Some((st_type, y)) = detected {
                self.engine.structure_bridge.register_world_structure(
                    st_type,
                    cx,
                    cz,
                    y,
                    dim,
                );
                self.add_toast(
                    format!("🏛️ Subterráneo: {} detectado en Y={}", st_type, y),
                    0x00FFFF,
                );
            }
        }
    }

    fn on_structure_found(&mut self, st_type: &str, chunk_x: i32, chunk_z: i32) {
        // Register structure as verified from the real world
        let info = self.engine.structure_bridge.register_world_structure(
            st_type,
            chunk_x,
            chunk_z,
            self.cave_slice_y,
            self.current_dimension.clone(),
        );

        let msg = format!(
            "🏛️ VkMap [VERIF]: {} en Chunk ({}, {}) [{}, {}]",
            info.display_name, chunk_x, chunk_z, info.block_x, info.block_z
        );
        self.add_toast(msg, 0xFFD700);
        let _ = self.save_current_world();
    }

    fn on_command(&mut self, command: &str) -> bool {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return false;
        }

        let root = parts[0].to_ascii_lowercase();
        if !matches!(root.as_str(), "/vkmap" | "vkmap" | "/map" | "map") {
            return false;
        }

        if parts.len() == 1 {
            self.show_minimap = !self.show_minimap;
            let status = if self.show_minimap { "ACTIVADO" } else { "OCULTO" };
            self.add_toast(format!("🗺️ Minimapa HUD {}", status), 0x00FF88);
            return true;
        }

        match parts[1].to_ascii_lowercase().as_str() {
            "hud" => {
                self.show_hud = !self.show_hud;
                self.add_toast(format!("🗺️ VkMap HUD {}", if self.show_hud { "ON" } else { "OFF" }), 0x00FF88);
                true
            }
            "size" => {
                if parts.len() >= 3 {
                    match parts[2].to_ascii_lowercase().as_str() {
                        "small" | "pequeño" | "s" => self.engine.minimap.config.display_size = MinimapDisplaySize::Small,
                        "medium" | "mediano" | "m" => self.engine.minimap.config.display_size = MinimapDisplaySize::Medium,
                        "fullscreen" | "grande" | "f" => self.engine.minimap.config.display_size = MinimapDisplaySize::Fullscreen,
                        _ => self.engine.minimap.config.display_size = self.engine.minimap.config.display_size.next(),
                    }
                } else {
                    self.engine.minimap.config.display_size = self.engine.minimap.config.display_size.next();
                }
                self.engine.touch_widgets.display_size = self.engine.minimap.config.display_size;
                self.add_toast(format!("📐 Tamaño Minimapa: {}", self.engine.minimap.config.display_size.name()), 0x00FFFF);
                true
            }
            "zoom" => {
                if parts.len() >= 3 {
                    match parts[2].to_ascii_lowercase().as_str() {
                        "in" | "+" => self.engine.touch_widgets.handle_action(TouchWidgetAction::ZoomIn),
                        "out" | "-" => self.engine.touch_widgets.handle_action(TouchWidgetAction::ZoomOut),
                        v => {
                            if let Ok(z) = v.parse::<f32>() {
                                self.engine.minimap.config.zoom = z.clamp(0.25, 8.0);
                                self.engine.touch_widgets.zoom_level = self.engine.minimap.config.zoom;
                            }
                        }
                    }
                } else {
                    self.engine.touch_widgets.handle_action(TouchWidgetAction::ZoomIn);
                }
                self.engine.minimap.config.zoom = self.engine.touch_widgets.zoom_level;
                self.add_toast(format!("🔎 Zoom: {:.1}x", self.engine.minimap.config.zoom), 0xFFFF00);
                true
            }
            "radar" => {
                if parts.len() >= 3 {
                    match parts[2].to_ascii_lowercase().as_str() {
                        "los" | "vision" => self.engine.radar_tracker.visibility_mode = RadarVisibilityMode::LineOfSight,
                        "all" | "xray" | "todas" => self.engine.radar_tracker.visibility_mode = RadarVisibilityMode::AllLayers,
                        "surface" | "superficie" => self.engine.radar_tracker.visibility_mode = RadarVisibilityMode::SurfaceOnly,
                        _ => self.engine.radar_tracker.visibility_mode = self.engine.radar_tracker.visibility_mode.next(),
                    }
                } else {
                    self.engine.radar_tracker.visibility_mode = self.engine.radar_tracker.visibility_mode.next();
                }
                self.add_toast(format!("👁️ Modo Radar: {}", self.engine.radar_tracker.visibility_mode.name()), 0xFFAA00);
                true
            }
            "cave" | "cuevas" => {
                if parts.len() >= 3 {
                    if let Ok(y) = parts[2].parse::<i32>() {
                        self.cave_slice_y = y.clamp(-64, 320);
                        self.cave_mode_active = true;
                    } else if parts[2].eq_ignore_ascii_case("off") {
                        self.cave_mode_active = false;
                    } else {
                        self.cave_mode_active = !self.cave_mode_active;
                    }
                } else {
                    self.cave_mode_active = !self.cave_mode_active;
                }
                let st = if self.cave_mode_active {
                    format!("ACTIVADO (Capa Y={})", self.cave_slice_y)
                } else {
                    "DESACTIVADO".to_string()
                };
                self.add_toast(format!("🕳️ Analizador Subterráneo {}", st), 0x55FFFF);
                true
            }
            "wp" | "waypoint" => {
                if parts.len() >= 3 && parts[2].eq_ignore_ascii_case("add") {
                    let name = if parts.len() >= 4 { parts[3..].join(" ") } else { "Punto Marcado".to_string() };
                    self.create_waypoint_at_player(&name, [0, 255, 128]);
                } else if parts.len() >= 3 && parts[2].eq_ignore_ascii_case("death") {
                    if let Some(dwp) = self.engine.waypoint_manager.get_active_death_point() {
                        self.add_toast(format!("💀 Última Muerte en [{}, {}, {}]", dwp.x, dwp.y, dwp.z), 0xFF4444);
                    } else {
                        self.add_toast("No hay puntos de muerte registrados", 0xAAAAAA);
                    }
                } else {
                    self.add_toast(format!("📍 Waypoints totales: {}", self.engine.waypoint_manager.len()), 0x00FF88);
                }
                true
            }
            "world" => {
                if parts.len() >= 3 {
                    let wid = parts[2];
                    self.switch_world(wid);
                } else {
                    self.add_toast(format!("🌍 Mundo actual: {}", self.current_world_id), 0x55FF55);
                }
                true
            }
            "save" => {
                let _ = self.save_current_world();
                self.add_toast("💾 Mundo guardado exitosamente", 0x55FF55);
                true
            }
            "export" => {
                if parts.len() >= 3 && parts[2].eq_ignore_ascii_case("json") {
                    let _ = self.export_json();
                } else {
                    let _ = self.export_png();
                }
                true
            }
            "menu" => {
                self.engine.touch_widgets.handle_action(TouchWidgetAction::OpenMapMenu);
                true
            }
            "sync_seed" | "seed" => {
                if parts.len() >= 3 {
                    if let Ok(seed) = parts[2].parse::<u64>() {
                        self.sync_predicted_structures(&[seed]);
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn hud_lines(&self) -> Vec<String> {
        if !self.show_hud {
            return Vec::new();
        }

        let mut lines = Vec::new();
        let (px, py, pz) = self.player_pos;

        // Line 1: Header & World Status
        lines.push(format!(
            "§6[VkMap] §fMundo: §a{} §7| §e{} §7| §b{} §7| §d{:.1}x",
            self.current_world_id,
            self.current_dimension.as_str(),
            self.engine.minimap.config.display_size.name(),
            self.engine.minimap.config.zoom
        ));

        // Line 2: Telemetry
        let facing = FacingDirection::from_yaw(self.player_yaw).display_name();
        lines.push(format!(
            "§7Pos: §f{:.1}, {:.1}, {:.1} §7(§e{}§7) §7| Bioma: §a{}",
            px, py, pz, facing, self.current_biome
        ));

        // SeedCracker publication is surfaced without inventing map markers.
        if let Some(seed) = self.seed_candidate {
            lines.push(format!("§6SeedCracker: §e0x{:016X} §7(candidato)", seed));
        }

        // Line 3: Radar Contact Summary
        let radar_mode = self.engine.radar_tracker.visibility_mode.name();
        let contact_count = self.simulated_entities.len();
        lines.push(format!(
            "§7Radar: §c{} §7(§e{}§7) §7| Entidades: §f{}",
            if self.engine.radar_tracker.fair_play_mode { "FAIR-PLAY" } else { "ACTIVO" },
            radar_mode,
            contact_count
        ));

        // Line 4: Nearest Discovered Structure (Verified vs Predicted)
        let dim_structures = self.engine.structure_bridge.get_in_dimension(&self.current_dimension);
        if let Some(closest) = dim_structures.iter().min_by(|a, b| {
            let da = (a.block_x as f64 - px).hypot(a.block_z as f64 - pz);
            let db = (b.block_x as f64 - px).hypot(b.block_z as f64 - pz);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        }) {
            let dist = (closest.block_x as f64 - px).hypot(closest.block_z as f64 - pz);
            let badge_color = match closest.source {
                StructureSource::WorldScan => "§a[VERIF]",
                StructureSource::SeedPrediction => "§6[PRED]",
            };
            lines.push(format!(
                "§7Estructura: {} §f{} §7a §e{:.0}m §7en [{}, {}]",
                badge_color, closest.display_name, dist, closest.block_x, closest.block_z
            ));
        }

        // Line 5: Active Waypoint / ETA
        let wps = self.engine.waypoint_manager.get_active_in_dimension(&self.current_dimension);
        if let Some(first_wp) = wps.first() {
            let dist = first_wp.distance_2d(px, pz);
            let eta = first_wp.eta_formatted(px, pz, TravelSpeed::Sprinting);
            lines.push(format!(
                "§7Nav: §b{} §7a §e{:.0}m §7(ETA: §a{}§7)",
                first_wp.name, dist, eta
            ));
        }

        // Floating Touch Widgets line (rendered adjacent to map on Android)
        lines.push(format!(
            "§e[ 🔍+ Ampliar ] [ 🔍- Reducir ] [ 🔎 Zoom: {:.1}x ] [ 🗺️ Menú VkMap ]",
            self.engine.minimap.config.zoom
        ));

        // Recent event toasts
        for t in &self.toasts {
            lines.push(format!("§6» §f{}", t.text));
        }

        lines
    }

    fn settings_buttons(&self) -> Vec<String> {
        let mut buttons = Vec::new();
        buttons.push(format!("📑 Pestaña: {}", self.page.name()));

        match self.page {
            VkMapPage::Navigation => {
                buttons.push("⭐ Crear Waypoint en Posición Actual".to_string());
                buttons.push(format!(
                    "💀 Punto de Muerte: {}",
                    if self.engine.waypoint_manager.get_active_death_point().is_some() {
                        "REGISTRADO"
                    } else {
                        "NINGUNO"
                    }
                ));
                buttons.push(format!(
                    "🧭 Forma Minimapa: {}",
                    match self.engine.minimap.config.shape {
                        MinimapShape::Round => "Circular ⭕",
                        MinimapShape::Square => "Cuadrado ⬛",
                    }
                ));
                buttons.push(format!(
                    "🔄 Rotación con Jugador: {}",
                    if self.engine.minimap.config.rotating { "ON" } else { "OFF (Norte Fijo)" }
                ));
                buttons.push("🗑️ Limpiar Todos los Waypoints".to_string());
            }
            VkMapPage::Structures => {
                buttons.push(format!(
                    "🕳️ Analizador Subterráneo: {}",
                    if self.cave_mode_active { "ON" } else { "OFF" }
                ));
                buttons.push(format!("📐 Capa de Corte Y: {}", self.cave_slice_y));
                buttons.push(format!(
                    "🏛️ Estructuras Registradas: {}",
                    self.engine.structure_bridge.structures.len()
                ));
                buttons.push("📡 Sincronizar Predicciones SeedCracker".to_string());
                buttons.push("🔍 Analizar Trial Chambers (-16) / Ancient City (-48)".to_string());
            }
            VkMapPage::Radar => {
                buttons.push(format!(
                    "👁️ Modo Radar: {}",
                    self.engine.radar_tracker.visibility_mode.name()
                ));
                buttons.push(format!(
                    "🧟 Mostrar Hostiles: {}",
                    if self.engine.radar_tracker.show_hostiles { "SI" } else { "NO" }
                ));
                buttons.push(format!(
                    "🐑 Mostrar Pasivos: {}",
                    if self.engine.radar_tracker.show_passives { "SI" } else { "NO" }
                ));
                buttons.push(format!(
                    "👤 Mostrar Jugadores: {}",
                    if self.engine.radar_tracker.show_players { "SI" } else { "NO" }
                ));
                buttons.push(format!(
                    "⚖️ Modo Fair-Play (Anti-Cheat): {}",
                    if self.engine.radar_tracker.fair_play_mode { "ACTIVADO" } else { "DESACTIVADO" }
                ));
            }
            VkMapPage::Perspectives => {
                buttons.push(format!(
                    "🗺️ Perspectiva: {}",
                    match self.render_mode {
                        RenderMode::TopDownFlat => "Superficie 2D Plana",
                        RenderMode::TopDownShaded => "Superficie 2D Sombreada (Relieve)",
                        RenderMode::Isometric => "Isométrico 3D Axonométrico",
                        RenderMode::CaveMode { .. } => "Corte de Cuevas Horizontal",
                        RenderMode::NetherCeilingCut => "Nether Sin Techo",
                    }
                ));
                buttons.push(format!(
                    "📐 Tamaño HUD: {}",
                    self.engine.minimap.config.display_size.name()
                ));
                buttons.push(format!(
                    "🔎 Nivel de Zoom: {:.1}x",
                    self.engine.minimap.config.zoom
                ));
                buttons.push(format!(
                    "💡 Motor de Iluminación: {}",
                    if self.engine.lighting_engine.mode != LightingMode::None { "ON" } else { "OFF" }
                ));
            }
            VkMapPage::ExportStorage => {
                buttons.push("🖼️ Exportar Mapa Completo a PNG (/sdcard/.../exports/)".to_string());
                buttons.push("📊 Exportar Datos Estructuras y Waypoints a JSON".to_string());
                buttons.push("💾 Guardar Datos del Mundo Ahora".to_string());
                buttons.push(format!("🌍 Mundo Actual: {}", self.current_world_id));
                buttons.push("🔄 Recargar Datos de Almacenamiento Local".to_string());
            }
        }

        buttons.push("➡️ Cambiar Pestaña Menú".to_string());
        buttons
    }

    fn on_settings_click(&mut self, button_idx: usize) {
        let is_last = button_idx >= 6;
        if button_idx == 0 || is_last {
            self.page = self.page.next();
            self.add_toast(format!("📑 Pestaña: {}", self.page.name()), 0x55FF55);
            return;
        }

        match self.page {
            VkMapPage::Navigation => match button_idx {
                1 => self.create_waypoint_at_player("Marcador Táctico", [0, 200, 255]),
                2 => {
                    if let Some(dwp) = self.engine.waypoint_manager.get_active_death_point() {
                        self.add_toast(format!("💀 Punto Muerte en [{}, {}, {}]", dwp.x, dwp.y, dwp.z), 0xFF4444);
                    } else {
                        self.add_toast("No hay puntos de muerte activos", 0xAAAAAA);
                    }
                }
                3 => {
                    self.engine.minimap.config.shape = match self.engine.minimap.config.shape {
                        MinimapShape::Round => MinimapShape::Square,
                        MinimapShape::Square => MinimapShape::Round,
                    };
                    self.add_toast("🔘 Forma de minimapa alternada", 0x00FFFF);
                }
                4 => {
                    self.engine.minimap.config.rotating = !self.engine.minimap.config.rotating;
                    self.add_toast(
                        format!("🔄 Rotación {}", if self.engine.minimap.config.rotating { "ON" } else { "OFF" }),
                        0x00FFFF,
                    );
                }
                5 => {
                    self.engine.waypoint_manager.clear();
                    self.add_toast("🗑️ Waypoints eliminados", 0xFFAA00);
                    let _ = self.save_current_world();
                }
                _ => {}
            },
            VkMapPage::Structures => match button_idx {
                1 => {
                    self.cave_mode_active = !self.cave_mode_active;
                    self.add_toast(
                        format!("🕳️ Analizador Subterráneo {}", if self.cave_mode_active { "ON" } else { "OFF" }),
                        0x00FFFF,
                    );
                }
                2 => {
                    self.cave_slice_y = match self.cave_slice_y {
                        -16 => -48,
                        -48 => 0,
                        0 => 32,
                        32 => 64,
                        _ => -16,
                    };
                    self.add_toast(format!("📐 Capa de Corte Y = {}", self.cave_slice_y), 0x00FFFF);
                }
                3 => {
                    let total = self.engine.structure_bridge.structures.len();
                    self.add_toast(format!("🏛️ Total estructuras: {}", total), 0xFFD700);
                }
                4 => {
                    // Trigger sync with candidate seeds
                    self.sync_predicted_structures(&[123456789012345]);
                }
                5 => {
                    self.cave_slice_y = -16;
                    self.cave_mode_active = true;
                    self.add_toast("🔍 Enfocando Trial Chambers (-16) y Ancient Cities (-48)", 0x55FF55);
                }
                _ => {}
            },
            VkMapPage::Radar => match button_idx {
                1 => {
                    self.engine.radar_tracker.visibility_mode = self.engine.radar_tracker.visibility_mode.next();
                    self.add_toast(
                        format!("👁️ Modo Radar: {}", self.engine.radar_tracker.visibility_mode.name()),
                        0xFFAA00,
                    );
                }
                2 => {
                    self.engine.radar_tracker.show_hostiles = !self.engine.radar_tracker.show_hostiles;
                    self.add_toast(
                        format!("🧟 Hostiles: {}", if self.engine.radar_tracker.show_hostiles { "ON" } else { "OFF" }),
                        0xFFAA00,
                    );
                }
                3 => {
                    self.engine.radar_tracker.show_passives = !self.engine.radar_tracker.show_passives;
                    self.add_toast(
                        format!("🐑 Pasivos: {}", if self.engine.radar_tracker.show_passives { "ON" } else { "OFF" }),
                        0xFFAA00,
                    );
                }
                4 => {
                    self.engine.radar_tracker.show_players = !self.engine.radar_tracker.show_players;
                    self.add_toast(
                        format!("👤 Jugadores: {}", if self.engine.radar_tracker.show_players { "ON" } else { "OFF" }),
                        0xFFAA00,
                    );
                }
                5 => {
                    self.engine.radar_tracker.fair_play_mode = !self.engine.radar_tracker.fair_play_mode;
                    self.add_toast(
                        format!("⚖️ Fair-Play: {}", if self.engine.radar_tracker.fair_play_mode { "ON" } else { "OFF" }),
                        0xFF5555,
                    );
                }
                _ => {}
            },
            VkMapPage::Perspectives => match button_idx {
                1 => {
                    self.render_mode = match self.render_mode {
                        RenderMode::TopDownFlat => RenderMode::TopDownShaded,
                        RenderMode::TopDownShaded => RenderMode::Isometric,
                        RenderMode::Isometric => RenderMode::CaveMode {
                            slice_y: self.cave_slice_y,
                            depth_blocks: 32,
                        },
                        RenderMode::CaveMode { .. } => RenderMode::NetherCeilingCut,
                        RenderMode::NetherCeilingCut => RenderMode::TopDownFlat,
                    };
                    self.add_toast("🗺️ Perspectiva de Render cambiada", 0x55FF55);
                }
                2 => {
                    self.engine.minimap.config.display_size = self.engine.minimap.config.display_size.next();
                    self.engine.touch_widgets.display_size = self.engine.minimap.config.display_size;
                    self.add_toast(format!("📐 Tamaño HUD: {}", self.engine.minimap.config.display_size.name()), 0x00FFFF);
                }
                3 => {
                    self.engine.touch_widgets.handle_action(TouchWidgetAction::ZoomIn);
                    self.engine.minimap.config.zoom = self.engine.touch_widgets.zoom_level;
                    self.add_toast(format!("🔎 Zoom: {:.1}x", self.engine.minimap.config.zoom), 0xFFFF00);
                }
                4 => {
                    self.engine.lighting_engine.mode = match self.engine.lighting_engine.mode {
                        LightingMode::None => LightingMode::Daylight,
                        LightingMode::Daylight => LightingMode::Night,
                        LightingMode::Night => LightingMode::AmbientOcclusion,
                        LightingMode::AmbientOcclusion => LightingMode::None,
                    };
                    self.add_toast(
                        format!("💡 Modo Iluminación: {:?}", self.engine.lighting_engine.mode),
                        0xFFFF00,
                    );
                }
                _ => {}
            },
            VkMapPage::ExportStorage => match button_idx {
                1 => {
                    let _ = self.export_png();
                }
                2 => {
                    let _ = self.export_json();
                }
                3 => {
                    let _ = self.save_current_world();
                    self.add_toast("💾 Datos guardados en /sdcard/VkYakeeMods/VkMap/", 0x55FF55);
                }
                4 => {
                    self.add_toast(format!("🌍 Sesión actual: {}", self.current_world_id), 0x00FF88);
                }
                5 => {
                    let wid = self.current_world_id.clone();
                    self.switch_world(&wid);
                }
                _ => {}
            },
        }
    }
}
