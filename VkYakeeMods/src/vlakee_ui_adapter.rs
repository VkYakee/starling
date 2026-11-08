//! VlakeeUI adapter implementing VkMod for Starling / Horizon Android.
//! Brings Bedrock's Déesse UI in-game shortcuts, SmartF3, InvHUD, and quick tools to Rust:
//!   - SmartF3: Compact & Full telemetry, Nether portal converter, Moon phases, Biome
//!   - SmartF3 More (Speedrunner & Technical F-Keys):
//!       * F1: Cinema / Screenshot mode (Hide all HUD and GUI elements)
//!       * F3 + G: Chunk boundaries wireframe overlay
//!       * F3 + B: Entity hitboxes (vital for mob tracking, bastions, blazes)
//!       * F3 + H: Advanced tooltips (numerical durability, item IDs, NBT tags count)
//!       * F3 + N: Quick Spectator swap (swap between Spectator and previous gamemode)
//!       * F3 + A: Reload all world chunks (graphics/render reload)
//!       * F3 + C: Copy exact teleport coordinates (/execute in <dim> run tp @s ...)
//!       * F3 + D: Clear chat buffer
//!       * F3 + T: Reload textures and resource packs
//!       * Shift + F3: Pie Chart / Profiler overlay for tick & entity analysis
//!   - Master F3 Killswitch:
//!       * 1-touch button to turn OFF both SmartF3 and Starling's native default F3
//!   - Fast Crafting & Mass Crafting (Déesse mass_craft):
//!       * Mass Craft mode (craft all possible from inventory in 1 touch)
//!       * 1-Stack mode (craft up to 64 items max)
//!       * Quick recipes: Wood->Planks, Planks->Sticks, Planks->Table, Torches, Blocks<->Ingots, Rockets
//!   - In-Game Options & Shortcuts (Déesse UI):
//!       * Camera perspective toggle (1st person, 3rd person back, 3rd person front / F5)
//!       * Quick Gamemode switcher (Survival, Creative, Spectator, Adventure)
//!       * Quick FOV presets (Zoom 30°, Normal 75.6°, Quake Pro 110°)
//!       * Render distance quick selector (8, 12, 16, 24 chunks)
//!       * Time and Weather shortcuts (/time set, /weather)
//!       * Coordinate Waypoints / Bookmarks (/vui mark <nombre>, /vui waypoints)
//!       * Chat coordinates sharing (/vui sharepos, /vui pastecoords)
//!       * Temporary in-game notepad (/vui note <texto>, /vui notes)
//!       * Quick Container actions (Move All, Drop All, Restock)
//!
//! Important Architectural Note:
//! The main start/title screen of Minecraft is preserved 100% in Java Edition style.
//! All VlakeeUI enhancements operate strictly INSIDE the game world (in-game pause screen,
//! in-game HUD, container screens, chat, and quick settings drawers).

use crate::api::{ModMetadata, VkMod};
use vlakee_ui::{
    CameraPerspective, ChunkViewer, ChunkViewerMode, F3Mode, FovPreset, InGameCalculator,
    InGameOptions, InvHud, QuickContainerMode, QuickCraftManager, QuickGamemode, QuickRecipe,
    QuickTime, QuickToggles, QuickWeather, SmartF3, SmartF3More, VlakeeHudMenu, VlakeeMenuPage,
};

use std::collections::VecDeque;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct ToastLog {
    pub text: String,
    pub rgb: u32,
    pub created_at: Instant,
}

pub struct VlakeeUIMod {
    pub f3: SmartF3,
    pub f3_more: SmartF3More,
    pub inv: InvHud,
    pub chunk_viewer: ChunkViewer,
    pub toggles: QuickToggles,
    pub options: InGameOptions,
    pub quick_craft: QuickCraftManager,
    pub master_f3_active: bool,
    pub menu: VlakeeHudMenu,
    pub active: bool,
    pub show_hud: bool,
    pub toasts: VecDeque<ToastLog>,
    pub status_message: String,
}

impl Default for VlakeeUIMod {
    fn default() -> Self {
        Self::new()
    }
}

impl VlakeeUIMod {
    pub fn new() -> Self {
        let mut f3 = SmartF3::new();
        f3.mode = F3Mode::Compact; // Default to sleek compact F3 telemetry

        Self {
            f3,
            f3_more: SmartF3More::new(),
            inv: InvHud::new(),
            chunk_viewer: ChunkViewer::new(),
            toggles: QuickToggles::new(),
            options: InGameOptions::new(),
            quick_craft: QuickCraftManager::new(),
            master_f3_active: true,
            menu: VlakeeHudMenu::new(),
            active: true,
            show_hud: true,
            toasts: VecDeque::new(),
            status_message: "VlakeeUI: Listo. SmartF3, Atajos Speedrun y Crafteo Rápido activos.".to_string(),
        }
    }

    pub fn add_toast(&mut self, text: impl Into<String>, rgb: u32) {
        let msg = text.into();
        log::info!("VlakeeUI Toast: {}", msg);
        if self.toasts.len() >= 6 {
            self.toasts.pop_front();
        }
        self.toasts.push_back(ToastLog {
            text: msg,
            rgb,
            created_at: Instant::now(),
        });
    }

    // ==========================================
    // Master F3 Killswitch
    // ==========================================

    /// Toggles both SmartF3 and Starling's native default F3 screen
    pub fn toggle_master_f3(&mut self) -> bool {
        self.master_f3_active = !self.master_f3_active;
        crate::set_default_f3_suppressed(!self.master_f3_active);
        if self.master_f3_active {
            self.f3.mode = F3Mode::Compact;
            self.add_toast("✅ F3 Maestro: ACTIVADO (SmartF3 y F3 nativo de Starling restaurados)", 0x55FF55);
        } else {
            self.f3.mode = F3Mode::Disabled;
            self.add_toast("⛔ F3 Maestro: APAGADO (SmartF3 y F3 nativo de Starling desactivados)", 0xFF5555);
        }
        self.master_f3_active
    }

    // ==========================================
    // SmartF3 More: Speedrun & Technical F-Keys
    // ==========================================

    /// Toggles F1 Cinema mode (hides HUD completely)
    pub fn toggle_f1(&mut self) {
        let hidden = self.f3_more.toggle_f1();
        self.toggles.hide_hud = hidden;
        let s = if hidden { "ACTIVADO (HUD Oculto)" } else { "DESACTIVADO (HUD Visible)" };
        self.add_toast(format!("🎬 Modo F1: {}", s), 0x55FFFF);
    }

    /// Toggles F3+G Chunk boundaries
    pub fn toggle_chunk_boundaries(&mut self) {
        let on = self.f3_more.toggle_chunk_boundaries();
        self.chunk_viewer.mode = if on { ChunkViewerMode::FullGrid } else { ChunkViewerMode::None };
        let s = if on { "ACTIVADOS" } else { "DESACTIVADOS" };
        self.add_toast(format!("🗺️ Bordes Chunks (F3+G): {}", s), 0x00FF88);
    }

    /// Toggles F3+B Entity Hitboxes (Speedrunner tool)
    pub fn toggle_entity_hitboxes(&mut self) {
        let on = self.f3_more.toggle_entity_hitboxes();
        let s = if on { "ACTIVADAS" } else { "DESACTIVADAS" };
        self.add_toast(format!("📦 Hitboxes de Entidades (F3+B): {}", s), 0xFFAA00);
    }

    /// Toggles F3+H Advanced Tooltips
    pub fn toggle_advanced_tooltips(&mut self) {
        let on = self.f3_more.toggle_advanced_tooltips();
        let s = if on { "ACTIVADOS (Durabilidad Numérica / IDs)" } else { "DESACTIVADOS" };
        self.add_toast(format!("ℹ️ Tooltips Avanzados (F3+H): {}", s), 0x55FFFF);
    }

    /// Quick spectator swap (F3+N)
    pub fn quick_spectator_swap(&mut self) {
        let is_spec = self.options.gamemode == QuickGamemode::Spectator;
        let (cmd, target_is_spec) = self.f3_more.swap_spectator_gamemode(is_spec);
        self.options.gamemode = if target_is_spec {
            QuickGamemode::Spectator
        } else {
            QuickGamemode::Survival
        };
        self.add_toast(format!("👻 F3+N: Modo cambiado a {} ({})", self.options.gamemode.name(), cmd), 0xFFAA00);
    }

    /// Reload chunks (F3+A)
    pub fn reload_chunks(&mut self) {
        self.add_toast("🔄 F3+A: Recargando todos los chunks de renderizado...", 0x55FF55);
    }

    /// Reload textures and resourcepacks (F3+T)
    pub fn reload_textures(&mut self) {
        self.add_toast("🎨 F3+T: Recargando texturas y ResourcePacks...", 0x55FFFF);
    }

    /// Clear chat history (F3+D)
    pub fn clear_chat_history(&mut self) {
        self.add_toast("🧹 F3+D: Historial de chat limpiado.", 0xAAAAAA);
    }

    /// Copies TP coordinates (F3+C)
    pub fn copy_tp_coords(&mut self) -> String {
        let (bx, by, bz) = self.f3.block_pos();
        let cmd = self.f3_more.generate_f3_c_tp(bx as f64, by as f64, bz as f64, self.f3.yaw, self.f3.pitch, &self.f3.dimension);
        self.add_toast(format!("📋 F3+C: Coords TP copiadas: {}", cmd), 0x55FFFF);
        cmd
    }

    /// Toggles Shift+F3 Pie Chart Profiler
    pub fn toggle_pie_chart(&mut self) {
        let on = self.f3_more.toggle_pie_chart();
        let s = if on { "ACTIVADO (Speedrun Entidades)" } else { "DESACTIVADO" };
        self.add_toast(format!("⏱️ Profiler / Pie Chart (Shift+F3): {}", s), 0xFFFF55);
    }

    // ==========================================
    // Fast Crafting & Mass Crafting
    // ==========================================

    /// Cycles Quick Craft mode (Normal -> MassCraft -> OneStack)
    pub fn cycle_craft_mode(&mut self) {
        let m = self.quick_craft.cycle_mode();
        self.add_toast(format!("⚡ Modo Crafteo: {}", m.name()), 0xFFAA00);
    }

    /// Performs rapid recipe craft
    pub fn execute_quick_craft(&mut self, recipe: QuickRecipe, available_inputs: usize) {
        let res = self.quick_craft.craft(recipe, available_inputs);
        self.add_toast(res.format_summary(), 0x00FF88);
    }

    // ==========================================
    // SmartF3 Core
    // ==========================================

    /// Cycles through SmartF3 modes: Disabled -> Compact -> Full -> Disabled
    pub fn cycle_f3_mode(&mut self) {
        self.f3.mode = self.f3.mode.next();
        self.add_toast(format!("📊 SmartF3: {}", self.f3.mode.name()), 0x55FFFF);
    }

    /// Cycles chunk viewer modes: None -> Middle -> Edge -> Corner -> Full Grid -> None
    pub fn cycle_chunk_mode(&mut self) {
        self.chunk_viewer.mode = self.chunk_viewer.mode.next();
        self.add_toast(format!("🗺️ Chunk Viewer: {}", self.chunk_viewer.mode.name()), 0x00FF88);
    }

    /// Toggles InvHUD (inventory slots and durability)
    pub fn toggle_inv_hud(&mut self) {
        self.inv.enabled = !self.inv.enabled;
        let s = if self.inv.enabled { "ACTIVADO" } else { "DESACTIVADO" };
        self.add_toast(format!("🎒 InvHUD: {}", s), 0xFFAA00);
    }

    /// Toggles durability tracking
    pub fn toggle_durability(&mut self) {
        self.inv.show_durability = !self.inv.show_durability;
        let s = if self.inv.show_durability { "VISIBLE" } else { "OCULTO" };
        self.add_toast(format!("⚔️ Visor de Durabilidad: {}", s), 0x55FF55);
    }

    /// Toggles night vision brightness overlay
    pub fn toggle_night_vision(&mut self) {
        let on = self.toggles.toggle_night_vision();
        if on {
            self.add_toast(format!("💡 Visión Nocturna: ACTIVADA ({:.0}%)", self.toggles.brightness_level * 100.0), 0xFFFF55);
        } else {
            self.add_toast("💡 Visión Nocturna: DESACTIVADA", 0xAAAAAA);
        }
    }

    /// Cycles night vision brightness: 25% -> 50% -> 75% -> 100% -> 25%
    pub fn cycle_night_vision_brightness(&mut self) {
        self.toggles.cycle_brightness();
        self.toggles.night_vision = true;
        self.add_toast(format!("💡 Brillo Nocturno: {:.0}%", self.toggles.brightness_level * 100.0), 0xFFFF55);
    }

    /// Toggles Nether Portal coordinate target calculation
    pub fn toggle_nether_converter(&mut self) {
        self.f3.show_nether_converter = !self.f3.show_nether_converter;
        let s = if self.f3.show_nether_converter { "ACTIVADO" } else { "OCULTO" };
        self.add_toast(format!("🔴 Convertidor Portal Nether: {}", s), 0xFF5555);
    }

    /// Toggles hiding coordinates
    pub fn toggle_hide_coordinates(&mut self) {
        self.f3.hide_coordinates = !self.f3.hide_coordinates;
        let s = if self.f3.hide_coordinates { "OCULTAS" } else { "VISIBLES" };
        self.add_toast(format!("🔢 Coordenadas: {}", s), 0x55FFFF);
    }

    /// Toggles recovery compass
    pub fn toggle_recovery_compass(&mut self) {
        self.toggles.recovery_compass = !self.toggles.recovery_compass;
        let s = if self.toggles.recovery_compass { "ACTIVADA" } else { "DESACTIVADA" };
        self.add_toast(format!("🧭 Brújula de Recuperación: {}", s), 0x00FF88);
    }

    /// Evaluates in-game calculator expression
    pub fn evaluate_calc(&mut self, expr: &str) {
        match InGameCalculator::evaluate(expr) {
            Ok(val) => {
                let stack_str = if (1.0..=1_000_000.0).contains(&val) {
                    InGameCalculator::calculate_stacks(val as usize)
                } else {
                    format!("Resultado: {}", val)
                };
                self.add_toast(format!("🧮 {} = {}", expr, val), 0x00FF88);
                self.add_toast(format!("📦 {}", stack_str), 0x55FFFF);
            }
            Err(e) => {
                self.add_toast(format!("❌ Error en cálculo: {}", e), 0xFF5555);
            }
        }
    }

    // ==========================================
    // In-Game Options & Shortcuts (Déesse UI)
    // ==========================================

    /// Cycles camera perspective (1st -> 3rd Back -> 3rd Front -> 1st)
    pub fn cycle_perspective(&mut self) {
        let p = self.options.cycle_perspective();
        self.add_toast(format!("📷 Cámara: {}", p.name()), 0x55FFFF);
    }

    /// Sets camera perspective directly
    pub fn set_perspective(&mut self, p: CameraPerspective) {
        self.options.perspective = p;
        self.add_toast(format!("📷 Cámara: {}", p.name()), 0x55FFFF);
    }

    /// Cycles gamemode (Survival -> Creative -> Spectator -> Adventure -> Survival)
    pub fn cycle_gamemode(&mut self) {
        let gm = self.options.cycle_gamemode();
        self.add_toast(format!("🎮 Modo de Juego: {} ({})", gm.name(), gm.command_str()), 0xFFAA00);
    }

    /// Sets gamemode directly
    pub fn set_gamemode(&mut self, gm: QuickGamemode) {
        self.options.gamemode = gm;
        self.add_toast(format!("🎮 Modo de Juego fijado: {} ({})", gm.name(), gm.command_str()), 0xFFAA00);
    }

    /// Cycles FOV presets (Normal 75.6° -> Quake Pro 110° -> Zoom 30° -> Normal 75.6°)
    pub fn cycle_fov(&mut self) {
        let f = self.options.cycle_fov();
        self.add_toast(format!("👁️ Campo de Visión: {}", f.label()), 0x55FF55);
    }

    /// Cycles render distance (8 -> 12 -> 16 -> 24 -> 8)
    pub fn cycle_render_distance(&mut self) {
        let rd = self.options.cycle_render_distance();
        self.add_toast(format!("🔭 Distancia Render: {} Chunks", rd), 0x55FFFF);
    }

    /// Cycles container mode (Move All -> Drop All -> Restock -> Move All)
    pub fn cycle_container_mode(&mut self) {
        self.options.container_mode = self.options.container_mode.cycle();
        self.add_toast(format!("📦 Modo Contenedor: {}", self.options.container_mode.name()), 0xFFAA00);
    }

    /// Adds an in-game temporary note
    pub fn add_note(&mut self, text: impl Into<String>) {
        let t = text.into();
        self.options.notes.add(t.clone());
        self.add_toast(format!("📝 Nota agregada: \"{}\"", t), 0xFFFF55);
    }

    /// Clears all temporary notes
    pub fn clear_notes(&mut self) {
        self.options.notes.clear();
        self.add_toast("📝 Todas las notas temporales han sido borradas.", 0xAAAAAA);
    }

    /// Bookmarks current player coordinates as a Waypoint
    pub fn add_current_waypoint(&mut self, name: &str) -> usize {
        let (x, y, z) = self.f3.block_pos();
        let dim = self.f3.dimension.clone();
        let id = self.options.add_waypoint(name, x, y, z, dim.clone());
        self.add_toast(format!("📍 Waypoint #{}: \"{}\" guardado en [{}, {}, {}] ({})", id, name, x, y, z, dim), 0x00FF88);
        id
    }

    /// Removes a waypoint by ID
    pub fn remove_waypoint(&mut self, id: usize) -> bool {
        if self.options.remove_waypoint(id) {
            self.add_toast(format!("📍 Waypoint #{} eliminado.", id), 0xFF5555);
            true
        } else {
            self.add_toast(format!("❌ No se encontró el Waypoint #{}.", id), 0xFF5555);
            false
        }
    }

    /// Formats and copies/toasts current coordinates to share in chat
    pub fn share_current_coords(&mut self) -> String {
        let (x, y, z) = self.f3.block_pos();
        let msg = InGameOptions::format_coords_for_chat(x, y, z, &self.f3.dimension);
        self.add_toast(format!("💬 Coords listas: {}", msg), 0x55FFFF);
        msg
    }

    /// Sets quick time preset
    pub fn set_time_preset(&mut self, time: QuickTime) {
        self.add_toast(format!("⏰ Tiempo fijado: {} ({})", time.name(), time.command_str()), 0xFFFF55);
    }

    /// Sets quick weather preset
    pub fn set_weather_preset(&mut self, weather: QuickWeather) {
        self.add_toast(format!("🌧️ Clima fijado: {} ({})", weather.name(), weather.command_str()), 0x55FFFF);
    }

    /// Toggles the in-game options pause menu drawer
    pub fn toggle_pause_menu(&mut self) {
        self.options.is_pause_menu_open = !self.options.is_pause_menu_open;
        let s = if self.options.is_pause_menu_open { "ABIERTO" } else { "CERRADO" };
        self.add_toast(format!("🎮 Menú de Opciones en Juego: {}", s), 0x00FF88);
    }
}

impl VkMod for VlakeeUIMod {
    fn metadata(&self) -> ModMetadata {
        ModMetadata {
            id: "vlakee_ui",
            name: "VlakeeUI",
            version: "0.3.0",
            description: "Atajos de Bedrock, SmartF3 More (Speedrun), InvHUD, Crafteo Rápido y Opciones Dentro del Juego",
            author: "VkYakee (Inspirado en Déesse UI por Itzriyo157)",
        }
    }

    fn on_tick(&mut self) {}

    fn on_player_pos(&mut self, x: i32, y: i32, z: i32) {
        self.f3.update_pos(x as f64, y as f64, z as f64, self.f3.yaw, self.f3.pitch);
        self.chunk_viewer.update_pos(x, y, z);
    }

    fn on_biome_found(&mut self, _x: i32, _z: i32, _biome_id: i32, name: &str) {
        self.f3.set_biome(name);
    }

    fn on_chunk_loaded(&mut self, cx: i32, cz: i32, dimension: &str, _bedrock_mask: Option<&[u8; 256]>) {
        self.f3.set_dimension(dimension);
        let _ = (cx, cz);
    }

    fn on_command(&mut self, command: &str) -> bool {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return false;
        }

        let prefix = parts[0].to_ascii_lowercase();

        // 1. Direct F-Key shortcuts
        if prefix == "f1" {
            self.toggle_f1();
            return true;
        }

        if prefix == "f5" {
            self.cycle_perspective();
            return true;
        }

        if prefix == "f3" && parts.len() == 1 {
            self.cycle_f3_mode();
            return true;
        }

        if !matches!(prefix.as_str(), "vui" | "vlakee" | "vlakeeui" | "f3") {
            return false;
        }

        let action = parts.get(1).map(|s| s.to_ascii_lowercase()).unwrap_or_else(|| "help".to_string());

        match action.as_str() {
            // Master F3 Killswitch
            "f3mute" | "silence" | "f3off" => {
                self.toggle_master_f3();
            }
            "master_f3_off" => {
                self.master_f3_active = false;
                self.f3.mode = F3Mode::Disabled;
            }
            "master_f3_on" => {
                self.master_f3_active = true;
                self.f3.mode = F3Mode::Compact;
            }
            // SmartF3 More (Speedrunner & Technical F-Keys)
            "f1" | "cinema" => {
                self.toggle_f1();
            }
            "f3b" | "hitbox" | "hitboxes" => {
                self.toggle_entity_hitboxes();
            }
            "f3g" | "borders" | "chunkborders" => {
                self.toggle_chunk_boundaries();
            }
            "f3h" | "tooltips" => {
                self.toggle_advanced_tooltips();
            }
            "f3n" | "spectate" => {
                self.quick_spectator_swap();
            }
            "f3a" | "reloadchunks" => {
                self.reload_chunks();
            }
            "f3c" | "copytp" => {
                self.copy_tp_coords();
            }
            "f3d" | "clearchat" => {
                self.clear_chat_history();
            }
            "f3t" | "reloadtextures" => {
                self.reload_textures();
            }
            "pie" | "profiler" => {
                self.toggle_pie_chart();
            }
            "more" | "f3more" => {
                self.menu.set_page(VlakeeMenuPage::Page4More);
                self.add_toast("➕ Menú: Página 4 (SmartF3 More & Speedrun)", 0x55FFFF);
            }
            // Fast Crafting & Mass Crafting
            "craft" => {
                match parts.get(2).copied().unwrap_or("menu") {
                    "mode" | "cycle" => self.cycle_craft_mode(),
                    "planks" | "wood" => self.execute_quick_craft(QuickRecipe::WoodToPlanks, 64),
                    "sticks" => self.execute_quick_craft(QuickRecipe::PlanksToSticks, 64),
                    "table" => self.execute_quick_craft(QuickRecipe::PlanksToCraftingTable, 64),
                    "torches" => self.execute_quick_craft(QuickRecipe::SticksCoalToTorches, 64),
                    "blocks" => self.execute_quick_craft(QuickRecipe::IngotsToBlock, 64),
                    "ingots" => self.execute_quick_craft(QuickRecipe::BlockToIngots, 64),
                    "rockets" => self.execute_quick_craft(QuickRecipe::PaperGunpowderToRockets, 64),
                    "bread" => self.execute_quick_craft(QuickRecipe::WheatToBread, 64),
                    "shield" => self.execute_quick_craft(QuickRecipe::IronToShield, 64),
                    "bucket" => self.execute_quick_craft(QuickRecipe::IronToBucket, 64),
                    _ => {
                        self.menu.set_page(VlakeeMenuPage::Page5Craft);
                        self.add_toast("⚡ Menú: Página 5 (Atajos de Crafteo Rápido)", 0xFFAA00);
                    }
                }
            }
            // SmartF3 core
            "f3" | "debug" => {
                match parts.get(2).copied().unwrap_or("next") {
                    "b" => self.toggle_entity_hitboxes(),
                    "g" => self.toggle_chunk_boundaries(),
                    "h" => self.toggle_advanced_tooltips(),
                    "n" => self.quick_spectator_swap(),
                    "a" => self.reload_chunks(),
                    "c" => { self.copy_tp_coords(); },
                    "d" => self.clear_chat_history(),
                    "t" => self.reload_textures(),
                    "compact" | "c_mode" => {
                        self.f3.mode = F3Mode::Compact;
                        self.add_toast("📊 SmartF3: Compacto", 0x55FFFF);
                    }
                    "full" | "f" => {
                        self.f3.mode = F3Mode::Full;
                        self.add_toast("📊 SmartF3: Completo", 0x55FFFF);
                    }
                    "off" | "disable" => {
                        self.f3.mode = F3Mode::Disabled;
                        self.add_toast("📊 SmartF3: Desactivado", 0xAAAAAA);
                    }
                    _ => self.cycle_f3_mode(),
                }
            }
            // Camera perspective (F5 equivalent)
            "cam" | "camera" | "perspective" | "f5" => {
                match parts.get(2).copied().unwrap_or("next") {
                    "first" | "1" => self.set_perspective(CameraPerspective::FirstPerson),
                    "back" | "3" => self.set_perspective(CameraPerspective::ThirdPersonBack),
                    "front" => self.set_perspective(CameraPerspective::ThirdPersonFront),
                    _ => self.cycle_perspective(),
                }
            }
            // Quick gamemode switcher
            "gm" | "gamemode" => {
                match parts.get(2).copied().unwrap_or("next") {
                    "s" | "survival" => self.set_gamemode(QuickGamemode::Survival),
                    "c" | "creative" => self.set_gamemode(QuickGamemode::Creative),
                    "sp" | "spectator" => self.set_gamemode(QuickGamemode::Spectator),
                    "a" | "adventure" => self.set_gamemode(QuickGamemode::Adventure),
                    _ => self.cycle_gamemode(),
                }
            }
            // Quick FOV presets
            "fov" => {
                if let Some(arg) = parts.get(2) {
                    match *arg {
                        "30" | "zoom" => {
                            self.options.fov = FovPreset::Zoom30;
                            self.add_toast("👁️ FOV: Zoom (30.0°)", 0x55FF55);
                        }
                        "75" | "normal" => {
                            self.options.fov = FovPreset::Normal75;
                            self.add_toast("👁️ FOV: Normal (75.6°)", 0x55FF55);
                        }
                        "110" | "quake" => {
                            self.options.fov = FovPreset::QuakePro110;
                            self.add_toast("👁️ FOV: Quake Pro (110.0°)", 0x55FF55);
                        }
                        _ => {
                            if let Ok(val) = arg.parse::<f32>() {
                                self.options.fov = FovPreset::Custom(val.clamp(30.0, 110.0));
                                self.add_toast(format!("👁️ FOV Personalizado: {:.1}°", val), 0x55FF55);
                            } else {
                                self.cycle_fov();
                            }
                        }
                    }
                } else {
                    self.cycle_fov();
                }
            }
            // Render distance presets
            "render" | "distance" => {
                if let Some(arg) = parts.get(2) {
                    if let Ok(val) = arg.parse::<u32>() {
                        self.options.render_distance = val.clamp(2, 32);
                        self.add_toast(format!("🔭 Distancia Render: {} Chunks", self.options.render_distance), 0x55FFFF);
                    } else {
                        self.cycle_render_distance();
                    }
                } else {
                    self.cycle_render_distance();
                }
            }
            // Quick Time
            "time" => {
                match parts.get(2).copied().unwrap_or("day") {
                    "day" => self.set_time_preset(QuickTime::Day),
                    "noon" => self.set_time_preset(QuickTime::Noon),
                    "night" => self.set_time_preset(QuickTime::Night),
                    "midnight" => self.set_time_preset(QuickTime::Midnight),
                    _ => self.add_toast("Uso: /vui time [day|noon|night|midnight]", 0xFFFF55),
                }
            }
            // Quick Weather
            "weather" => {
                match parts.get(2).copied().unwrap_or("clear") {
                    "clear" | "sun" => self.set_weather_preset(QuickWeather::Clear),
                    "rain" => self.set_weather_preset(QuickWeather::Rain),
                    "thunder" => self.set_weather_preset(QuickWeather::Thunder),
                    _ => self.add_toast("Uso: /vui weather [clear|rain|thunder]", 0xFFFF55),
                }
            }
            // Temporary notes
            "note" => {
                if parts.len() > 2 {
                    let note_text = parts[2..].join(" ");
                    self.add_note(note_text);
                } else {
                    self.add_toast("Uso: /vui note <texto de la nota>", 0xFFFF55);
                }
            }
            "notes" => {
                if self.options.notes.is_empty() {
                    self.add_toast("📝 No hay notas temporales guardadas.", 0xAAAAAA);
                } else {
                    let notes: Vec<String> = self.options.notes.entries.clone();
                    self.add_toast(format!("📝 Notas ({}) :", notes.len()), 0xFFFF55);
                    for (i, n) in notes.into_iter().enumerate() {
                        self.add_toast(format!("  {}. {}", i + 1, n), 0xFFFFFF);
                    }
                }
            }
            "clearnotes" => {
                self.clear_notes();
            }
            // Coordinate Waypoints
            "mark" | "waypoint" => {
                if parts.len() > 2 {
                    let wp_name = parts[2..].join(" ");
                    self.add_current_waypoint(&wp_name);
                } else {
                    self.add_current_waypoint("Punto de Interés");
                }
            }
            "waypoints" => {
                if self.options.waypoints.is_empty() {
                    self.add_toast("📍 No hay waypoints guardados.", 0xAAAAAA);
                } else {
                    let waypoints: Vec<String> = self.options.waypoints.iter().map(|w| w.format_entry()).collect();
                    self.add_toast(format!("📍 Waypoints ({}) :", waypoints.len()), 0x00FF88);
                    for entry in waypoints {
                        self.add_toast(entry, 0x55FFFF);
                    }
                }
            }
            "delmark" => {
                if let Some(arg) = parts.get(2) {
                    if let Ok(id) = arg.parse::<usize>() {
                        self.remove_waypoint(id);
                    } else {
                        self.add_toast("Uso: /vui delmark <id_waypoint>", 0xFF5555);
                    }
                } else {
                    self.add_toast("Uso: /vui delmark <id_waypoint>", 0xFF5555);
                }
            }
            // Share coordinates to chat
            "sharepos" | "pastecoords" => {
                self.share_current_coords();
            }
            // Quick Container actions
            "container" | "qc" => {
                match parts.get(2).copied().unwrap_or("cycle") {
                    "move" => {
                        self.options.container_mode = QuickContainerMode::MoveAll;
                        self.add_toast("📦 Contenedor: Mover Todo activado", 0xFFAA00);
                    }
                    "drop" => {
                        self.options.container_mode = QuickContainerMode::DropAll;
                        self.add_toast("📦 Contenedor: Tirar Todo al Suelo activado", 0xFF5555);
                    }
                    "restock" => {
                        self.options.container_mode = QuickContainerMode::Restock;
                        self.add_toast("📦 Contenedor: Reabastecer Hotbar activado", 0x55FF55);
                    }
                    _ => self.cycle_container_mode(),
                }
            }
            // In-game pause menu options drawer
            "pause" => {
                self.toggle_pause_menu();
            }
            // Nether portal target
            "nether" | "portal" => {
                self.toggle_nether_converter();
            }
            // Chunk Viewer
            "chunk" | "grid" => {
                match parts.get(2).copied().unwrap_or("next") {
                    "mid" | "middle" => {
                        self.chunk_viewer.mode = ChunkViewerMode::ChunkMiddle;
                        self.add_toast("🗺️ Chunk Viewer: Centro", 0x00FF88);
                    }
                    "edge" => {
                        self.chunk_viewer.mode = ChunkViewerMode::ChunkEdge;
                        self.add_toast("🗺️ Chunk Viewer: Borde", 0x00FF88);
                    }
                    "corner" => {
                        self.chunk_viewer.mode = ChunkViewerMode::ChunkCorner;
                        self.add_toast("🗺️ Chunk Viewer: Esquina", 0x00FF88);
                    }
                    "grid" | "all" => {
                        self.chunk_viewer.mode = ChunkViewerMode::FullGrid;
                        self.add_toast("🗺️ Chunk Viewer: Rejilla 3D", 0x00FF88);
                    }
                    "none" | "off" => {
                        self.chunk_viewer.mode = ChunkViewerMode::None;
                        self.add_toast("🗺️ Chunk Viewer: Desactivado", 0xAAAAAA);
                    }
                    _ => self.cycle_chunk_mode(),
                }
            }
            // InvHUD & Durability
            "inv" | "invhud" => {
                self.toggle_inv_hud();
            }
            "dura" | "durability" => {
                self.toggle_durability();
            }
            // Night Vision
            "nv" | "nightvision" | "gamma" => {
                if let Some(arg) = parts.get(2) {
                    if let Ok(val) = arg.parse::<f32>() {
                        self.toggles.brightness_level = (val / 100.0).clamp(0.1, 1.0);
                        self.toggles.night_vision = true;
                        self.add_toast(format!("💡 Brillo Nocturno fijado en {:.0}%", self.toggles.brightness_level * 100.0), 0xFFFF55);
                    } else if *arg == "cycle" {
                        self.cycle_night_vision_brightness();
                    } else {
                        self.toggle_night_vision();
                    }
                } else {
                    self.toggle_night_vision();
                }
            }
            // Clock and Compass
            "clock" | "compass" => {
                self.toggles.clock_compass = !self.toggles.clock_compass;
                let s = if self.toggles.clock_compass { "ACTIVADO" } else { "DESACTIVADO" };
                self.add_toast(format!("🧭 Reloj/Brújula: {}", s), 0x55FFFF);
            }
            // Hide coordinates / HUD toggles
            "hidecoords" | "pos" => {
                self.toggle_hide_coordinates();
            }
            "hidechat" | "chat" => {
                self.toggles.hide_chat = !self.toggles.hide_chat;
                let s = if self.toggles.hide_chat { "OCULTO" } else { "VISIBLE" };
                self.add_toast(format!("💬 Chat: {}", s), 0x00FF88);
            }
            "hidesb" | "scoreboard" => {
                self.toggles.hide_scoreboard = !self.toggles.hide_scoreboard;
                let s = if self.toggles.hide_scoreboard { "OCULTO" } else { "VISIBLE" };
                self.add_toast(format!("📋 Scoreboard: {}", s), 0x00FF88);
            }
            "hidehud" | "hud" => {
                self.show_hud = !self.show_hud;
                let s = if self.show_hud { "VISIBLE" } else { "OCULTO" };
                self.add_toast(format!("🖥️ HUD VlakeeUI: {}", s), 0x00FF88);
            }
            // Death coords
            "death" | "recovery" => {
                if parts.len() >= 5 {
                    if let (Ok(x), Ok(y), Ok(z)) = (parts[2].parse::<i32>(), parts[3].parse::<i32>(), parts[4].parse::<i32>()) {
                        self.toggles.record_death(x, y, z);
                        self.add_toast(format!("💀 Muerte registrada en [{}, {}, {}]", x, y, z), 0xFF5555);
                    }
                } else {
                    let (bx, by, bz) = self.f3.block_pos();
                    self.toggles.record_death(bx, by, bz);
                    self.add_toast(format!("💀 Muerte fijada aquí: [{}, {}, {}]", bx, by, bz), 0xFF5555);
                }
            }
            // In-game calculator
            "calc" => {
                if parts.len() > 2 {
                    let expr = parts[2..].join(" ");
                    self.evaluate_calc(&expr);
                } else {
                    self.add_toast("Uso: /vui calc <expresión> (ej: /vui calc 128*4+64)", 0xFFFF55);
                }
            }
            // Page navigation
            "page" => {
                self.menu.toggle_page();
                self.add_toast(format!("📑 Menú: {}", self.menu.active_page.name()), 0x55FFFF);
            }
            _ => {
                self.add_toast("Comandos: /vui [more|craft|cam|gm|fov|f3|f3mute|f1|f3b|f3g|f3h|f3n|f3c|f3a|time|weather|calc|pause]", 0xFFFF55);
            }
        }

        true
    }

    fn hud_lines(&self) -> Vec<String> {
        // F1 hides entire HUD (cinema / screenshot mode)
        if self.f3_more.f1_hud_hidden || !self.active || !self.show_hud || self.toggles.hide_hud {
            return Vec::new();
        }

        let mut lines = Vec::new();

        // If Master F3 is silenced, show notification badge if desired or skip F3
        if !self.master_f3_active {
            lines.push("⛔ [ F3 Maestro: APAGADO (SmartF3 + Starling F3 Silenciados) ]".to_string());
        } else {
            // 1. SmartF3 Section (Compact or Full)
            let f3_lines = self.f3.generate_hud_lines();
            lines.extend(f3_lines);
        }

        // 2. SmartF3 More Speedrun Indicators (Hitboxes, Chunk Boundaries, Pie Chart)
        let more_lines = self.f3_more.generate_status_lines();
        lines.extend(more_lines);

        // 3. InvHUD Section (Capacity & Durability)
        let inv_lines = self.inv.generate_hud_lines();
        lines.extend(inv_lines);

        // 4. Chunk Viewer Alert / Location Status
        if let Some(lbl) = self.chunk_viewer.status_label() {
            lines.push(format!("🗺️ {}", lbl));
        }

        // 5. Quick Toggles (Recovery Compass & Night Vision)
        let (bx, by, bz) = self.f3.block_pos();
        let toggle_lines = self.toggles.generate_hud_lines(bx, by, bz);
        lines.extend(toggle_lines);

        // 6. In-Game Pause Drawer (when open)
        if self.options.is_pause_menu_open {
            lines.push("┌─ [ 🎮 Opciones Dentro del Juego (Déesse Pause) ]".to_string());
            for opt_line in self.options.generate_in_game_menu_lines() {
                lines.push(format!("│  {}", opt_line));
            }
            lines.push("└──────────────────────────────────────────────".to_string());
        }

        // 7. Recent Event Toasts
        if !self.toasts.is_empty() {
            lines.push("├─ [ 🔔 Notificaciones VlakeeUI ]".to_string());
            for t in self.toasts.iter().rev().take(2) {
                lines.push(format!("│  {}", t.text));
            }
        }

        lines
    }

    fn render_bounding_boxes(&self) -> Vec<[f32; 10]> {
        if self.f3_more.f1_hud_hidden || !self.active || self.toggles.hide_hud {
            return Vec::new();
        }

        self.chunk_viewer.render_bounding_boxes()
    }

    fn settings_buttons(&self) -> Vec<String> {
        match self.menu.active_page {
            VlakeeMenuPage::Page1 => {
                vec![
                    format!("📊 SmartF3: [{}]", self.f3.mode.name()),
                    format!("🎒 InvHUD: [{}]", if self.inv.enabled { "ACTIVADO" } else { "DESACTIVADO" }),
                    format!("⚔️ Durabilidad: [{}]", if self.inv.show_durability { "VISIBLE" } else { "OCULTO" }),
                    format!(
                        "💡 Visión Nocturna: [{}]",
                        if self.toggles.night_vision {
                            format!("{:.0}%", self.toggles.brightness_level * 100.0)
                        } else {
                            "OFF".to_string()
                        }
                    ),
                    format!("⛔ F3 Maestro: [{}]", if self.master_f3_active { "ACTIVO" } else { "APAGADO" }),
                    "➕ [ SMARTF3 MORE (Speedrun & F-Keys) ]".to_string(),
                    "➡️ Ir a Página 2 (Chunks, Nether, Muerte)".to_string(),
                ]
            }
            VlakeeMenuPage::Page2 => {
                vec![
                    format!("🗺️ Chunk Viewer: [{}]", self.chunk_viewer.mode.name()),
                    format!("🔴 Convertidor Nether: [{}]", if self.f3.show_nether_converter { "ACTIVADO" } else { "DESACTIVADO" }),
                    format!("🧭 Brújula Muerte: [{}]", if self.toggles.recovery_compass { "ACTIVADA" } else { "DESACTIVADA" }),
                    format!("🔢 Coordenadas: [{}]", if self.f3.hide_coordinates { "OCULTAS" } else { "VISIBLES" }),
                    format!("💬 Chat en Pantalla: [{}]", if self.toggles.hide_chat { "OCULTO" } else { "VISIBLE" }),
                    "🧮 [Calcular Stacks (64 x 4)]".to_string(),
                    "➡️ Ir a Página 3 (🎮 Opciones Dentro del Juego)".to_string(),
                ]
            }
            VlakeeMenuPage::Page3 => {
                vec![
                    format!("📷 Cámara (F5): [{}]", self.options.perspective.name()),
                    format!("🎮 Modo de Juego: [{}]", self.options.gamemode.name()),
                    format!("👁️ Campo Visión (FOV): [{}]", self.options.fov.label()),
                    format!("🔭 Render: [{} Chunks]", self.options.render_distance),
                    format!("📦 Atajos Contenedor: [{}]", self.options.container_mode.name()),
                    "⚡ [ ATAJOS DE CRAFTEO RÁPIDO ]".to_string(),
                    "📍 [Guardar Coords Aquí (Waypoint)]".to_string(),
                    "💬 [Compartir Coordenadas en Chat]".to_string(),
                    "➡️ Ir a Página 4 (➕ SmartF3 More)".to_string(),
                ]
            }
            VlakeeMenuPage::Page4More => {
                vec![
                    format!("🎬 F1 (Ocultar HUD): [{}]", if self.f3_more.f1_hud_hidden { "OCULTO" } else { "VISIBLE" }),
                    format!("🗺️ F3+G (Bordes Chunks): [{}]", if self.f3_more.chunk_boundaries { "ON" } else { "OFF" }),
                    format!("📦 F3+B (Hitboxes): [{}]", if self.f3_more.entity_hitboxes { "ON" } else { "OFF" }),
                    format!("ℹ️ F3+H (Tooltips): [{}]", if self.f3_more.advanced_tooltips { "ON" } else { "OFF" }),
                    "👻 [ F3+N: Alternar Modo Espectador ]".to_string(),
                    "🔄 [ F3+A: Recargar Todos los Chunks ]".to_string(),
                    "📋 [ F3+C: Copiar Coords TP ]".to_string(),
                    format!("⏱️ Shift+F3 (Pie Chart): [{}]", if self.f3_more.pie_chart_profiler { "ON" } else { "OFF" }),
                    "➡️ Ir a Página 5 (⚡ Crafteo Rápido)".to_string(),
                ]
            }
            VlakeeMenuPage::Page5Craft => {
                vec![
                    format!("⚡ Modo Crafteo: [{}]", self.quick_craft.mode.name()),
                    "🪵 [ Craftear: Madera ➔ Tablones ]".to_string(),
                    "🥢 [ Craftear: Tablones ➔ Palos ]".to_string(),
                    "🪑 [ Craftear: Tablones ➔ Mesa Crafteo ]".to_string(),
                    "🕯️ [ Craftear: Palos+Carbón ➔ Antorchas ]".to_string(),
                    "🧱 [ Craftear: Lingotes ➔ Bloque Compacto ]".to_string(),
                    "🪙 [ Craftear: Bloques ➔ Lingotes ]".to_string(),
                    "🚀 [ Craftear: Pólvora+Papel ➔ Cohetes ]".to_string(),
                    "⬅️ Volver a Página 1 (F3 & InvHUD)".to_string(),
                ]
            }
        }
    }

    fn on_settings_click(&mut self, button_idx: usize) {
        match self.menu.active_page {
            VlakeeMenuPage::Page1 => {
                match button_idx {
                    0 => self.cycle_f3_mode(),
                    1 => self.toggle_inv_hud(),
                    2 => self.toggle_durability(),
                    3 => self.cycle_night_vision_brightness(),
                    4 => { self.toggle_master_f3(); }
                    5 => self.menu.set_page(VlakeeMenuPage::Page4More),
                    6 => self.menu.set_page(VlakeeMenuPage::Page2),
                    _ => {}
                }
            }
            VlakeeMenuPage::Page2 => {
                match button_idx {
                    0 => self.cycle_chunk_mode(),
                    1 => self.toggle_nether_converter(),
                    2 => self.toggle_recovery_compass(),
                    3 => self.toggle_hide_coordinates(),
                    4 => {
                        self.toggles.hide_chat = !self.toggles.hide_chat;
                        let s = if self.toggles.hide_chat { "OCULTO" } else { "VISIBLE" };
                        self.add_toast(format!("💬 Chat: {}", s), 0x00FF88);
                    }
                    5 => self.evaluate_calc("64 * 4"),
                    6 => self.menu.set_page(VlakeeMenuPage::Page3),
                    _ => {}
                }
            }
            VlakeeMenuPage::Page3 => {
                match button_idx {
                    0 => self.cycle_perspective(),
                    1 => self.cycle_gamemode(),
                    2 => self.cycle_fov(),
                    3 => self.cycle_render_distance(),
                    4 => self.cycle_container_mode(),
                    5 => self.menu.set_page(VlakeeMenuPage::Page5Craft),
                    6 => {
                        let (x, y, z) = self.f3.block_pos();
                        let name = format!("Punto ({}, {}, {})", x, y, z);
                        self.add_current_waypoint(&name);
                    }
                    7 => {
                        self.share_current_coords();
                    }
                    8 => self.menu.set_page(VlakeeMenuPage::Page4More),
                    _ => {}
                }
            }
            VlakeeMenuPage::Page4More => {
                match button_idx {
                    0 => self.toggle_f1(),
                    1 => self.toggle_chunk_boundaries(),
                    2 => self.toggle_entity_hitboxes(),
                    3 => self.toggle_advanced_tooltips(),
                    4 => self.quick_spectator_swap(),
                    5 => self.reload_chunks(),
                    6 => { self.copy_tp_coords(); }
                    7 => self.toggle_pie_chart(),
                    8 => self.menu.set_page(VlakeeMenuPage::Page5Craft),
                    _ => {}
                }
            }
            VlakeeMenuPage::Page5Craft => {
                match button_idx {
                    0 => self.cycle_craft_mode(),
                    1 => self.execute_quick_craft(QuickRecipe::WoodToPlanks, 64),
                    2 => self.execute_quick_craft(QuickRecipe::PlanksToSticks, 64),
                    3 => self.execute_quick_craft(QuickRecipe::PlanksToCraftingTable, 64),
                    4 => self.execute_quick_craft(QuickRecipe::SticksCoalToTorches, 64),
                    5 => self.execute_quick_craft(QuickRecipe::IngotsToBlock, 64),
                    6 => self.execute_quick_craft(QuickRecipe::BlockToIngots, 64),
                    7 => self.execute_quick_craft(QuickRecipe::PaperGunpowderToRockets, 64),
                    8 => self.menu.set_page(VlakeeMenuPage::Page1),
                    _ => {}
                }
            }
        }
    }
}
