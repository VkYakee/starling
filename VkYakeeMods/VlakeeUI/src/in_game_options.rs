//! In-game options and shortcuts ported from Déesse UI (Bedrock) to Starling (Rust).
//!
//! Important Architecture Constraint:
//! The main start/title screen of Minecraft is preserved 100% in Java Edition style.
//! All VlakeeUI enhancements, quick settings, pause menu overlays, container shortcuts,
//! coordinate bookmarks and temporary notes operate strictly INSIDE the game world.

use serde::{Deserialize, Serialize};

/// Camera perspective options (F5 equivalent in Java / Camera toggle in Déesse UI)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CameraPerspective {
    #[default]
    FirstPerson,
    ThirdPersonBack,
    ThirdPersonFront,
}

impl CameraPerspective {
    pub fn name(&self) -> &'static str {
        match self {
            CameraPerspective::FirstPerson => "1ª Persona",
            CameraPerspective::ThirdPersonBack => "3ª Persona (Espalda)",
            CameraPerspective::ThirdPersonFront => "3ª Persona (Frontal)",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            CameraPerspective::FirstPerson => CameraPerspective::ThirdPersonBack,
            CameraPerspective::ThirdPersonBack => CameraPerspective::ThirdPersonFront,
            CameraPerspective::ThirdPersonFront => CameraPerspective::FirstPerson,
        }
    }
}

/// Quick gamemode selector (Déesse UI quick gamemode panel)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QuickGamemode {
    #[default]
    Survival,
    Creative,
    Spectator,
    Adventure,
}

impl QuickGamemode {
    pub fn name(&self) -> &'static str {
        match self {
            QuickGamemode::Survival => "Supervivencia",
            QuickGamemode::Creative => "Creativo",
            QuickGamemode::Spectator => "Espectador",
            QuickGamemode::Adventure => "Aventura",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            QuickGamemode::Survival => QuickGamemode::Creative,
            QuickGamemode::Creative => QuickGamemode::Spectator,
            QuickGamemode::Spectator => QuickGamemode::Adventure,
            QuickGamemode::Adventure => QuickGamemode::Survival,
        }
    }

    pub fn command_str(&self) -> &'static str {
        match self {
            QuickGamemode::Survival => "/gamemode survival",
            QuickGamemode::Creative => "/gamemode creative",
            QuickGamemode::Spectator => "/gamemode spectator",
            QuickGamemode::Adventure => "/gamemode adventure",
        }
    }
}

/// Field of View quick presets (Déesse UI 30°, 75.6°, 110°)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FovPreset {
    Zoom30,
    Normal75,
    QuakePro110,
    Custom(f32),
}

impl Default for FovPreset {
    fn default() -> Self {
        FovPreset::Normal75
    }
}

impl FovPreset {
    pub fn degrees(&self) -> f32 {
        match self {
            FovPreset::Zoom30 => 30.0,
            FovPreset::Normal75 => 75.6,
            FovPreset::QuakePro110 => 110.0,
            FovPreset::Custom(v) => *v,
        }
    }

    pub fn label(&self) -> String {
        match self {
            FovPreset::Zoom30 => "Zoom (30.0°)".to_string(),
            FovPreset::Normal75 => "Normal (75.6°)".to_string(),
            FovPreset::QuakePro110 => "Quake Pro (110.0°)".to_string(),
            FovPreset::Custom(v) => format!("Personalizado ({:.1}°)", v),
        }
    }

    pub fn cycle(&self) -> Self {
        match self {
            FovPreset::Normal75 => FovPreset::QuakePro110,
            FovPreset::QuakePro110 => FovPreset::Zoom30,
            FovPreset::Zoom30 => FovPreset::Normal75,
            FovPreset::Custom(_) => FovPreset::Normal75,
        }
    }
}

/// Quick Time presets (Déesse UI host shortcuts)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuickTime {
    Day,
    Noon,
    Night,
    Midnight,
}

impl QuickTime {
    pub fn name(&self) -> &'static str {
        match self {
            QuickTime::Day => "Día (1000)",
            QuickTime::Noon => "Mediodía (6000)",
            QuickTime::Night => "Noche (13000)",
            QuickTime::Midnight => "Medianoche (18000)",
        }
    }

    pub fn command_str(&self) -> &'static str {
        match self {
            QuickTime::Day => "/time set day",
            QuickTime::Noon => "/time set noon",
            QuickTime::Night => "/time set night",
            QuickTime::Midnight => "/time set midnight",
        }
    }
}

/// Quick Weather presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuickWeather {
    Clear,
    Rain,
    Thunder,
}

impl QuickWeather {
    pub fn name(&self) -> &'static str {
        match self {
            QuickWeather::Clear => "Despejado",
            QuickWeather::Rain => "Lluvia",
            QuickWeather::Thunder => "Tormenta eléctrica",
        }
    }

    pub fn command_str(&self) -> &'static str {
        match self {
            QuickWeather::Clear => "/weather clear",
            QuickWeather::Rain => "/weather rain",
            QuickWeather::Thunder => "/weather thunder",
        }
    }
}

/// In-game coordinate note / waypoint (Déesse UI coord_notes)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CoordWaypoint {
    pub id: usize,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub dimension: String,
}

impl CoordWaypoint {
    pub fn format_entry(&self) -> String {
        format!("#{} [{}] @ [{}, {}, {}] ({})", self.id, self.name, self.x, self.y, self.z, self.dimension)
    }
}

/// Temporary notes manager (Déesse UI temporary_notes)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemporaryNotes {
    pub entries: Vec<String>,
}

impl TemporaryNotes {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn add(&mut self, text: impl Into<String>) {
        let t = text.into();
        if !t.trim().is_empty() {
            self.entries.push(t);
        }
    }

    pub fn remove(&mut self, idx: usize) -> bool {
        if idx < self.entries.len() {
            self.entries.remove(idx);
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Quick Container action mode (Déesse UI quick_container mode_switcher: move, drop, restock)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QuickContainerMode {
    #[default]
    MoveAll,
    DropAll,
    Restock,
}

impl QuickContainerMode {
    pub fn name(&self) -> &'static str {
        match self {
            QuickContainerMode::MoveAll => "Mover Todo a Contenedor",
            QuickContainerMode::DropAll => "Tirar Todo al Suelo",
            QuickContainerMode::Restock => "Reabastecer Hotbar",
        }
    }

    pub fn cycle(&self) -> Self {
        match self {
            QuickContainerMode::MoveAll => QuickContainerMode::DropAll,
            QuickContainerMode::DropAll => QuickContainerMode::Restock,
            QuickContainerMode::Restock => QuickContainerMode::MoveAll,
        }
    }
}

/// Complete in-game options manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InGameOptions {
    pub perspective: CameraPerspective,
    pub gamemode: QuickGamemode,
    pub fov: FovPreset,
    pub render_distance: u32,
    pub quick_quit_enabled: bool,
    pub waypoints: Vec<CoordWaypoint>,
    pub next_waypoint_id: usize,
    pub notes: TemporaryNotes,
    pub container_mode: QuickContainerMode,
    pub is_pause_menu_open: bool,
}

impl Default for InGameOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl InGameOptions {
    pub fn new() -> Self {
        Self {
            perspective: CameraPerspective::FirstPerson,
            gamemode: QuickGamemode::Survival,
            fov: FovPreset::Normal75,
            render_distance: 12,
            quick_quit_enabled: true,
            waypoints: Vec::new(),
            next_waypoint_id: 1,
            notes: TemporaryNotes::new(),
            container_mode: QuickContainerMode::MoveAll,
            is_pause_menu_open: false,
        }
    }

    /// Cycles camera perspective (F5)
    pub fn cycle_perspective(&mut self) -> CameraPerspective {
        self.perspective = self.perspective.next();
        self.perspective
    }

    /// Cycles quick gamemode
    pub fn cycle_gamemode(&mut self) -> QuickGamemode {
        self.gamemode = self.gamemode.next();
        self.gamemode
    }

    /// Cycles FOV preset (30° -> 75.6° -> 110°)
    pub fn cycle_fov(&mut self) -> FovPreset {
        self.fov = self.fov.cycle();
        self.fov
    }

    /// Cycles render distance (8 -> 12 -> 16 -> 24 -> 8)
    pub fn cycle_render_distance(&mut self) -> u32 {
        self.render_distance = match self.render_distance {
            8 => 12,
            12 => 16,
            16 => 24,
            _ => 8,
        };
        self.render_distance
    }

    /// Adds a coordinate waypoint
    pub fn add_waypoint(&mut self, name: impl Into<String>, x: i32, y: i32, z: i32, dimension: impl Into<String>) -> usize {
        let id = self.next_waypoint_id;
        self.next_waypoint_id += 1;
        self.waypoints.push(CoordWaypoint {
            id,
            name: name.into(),
            x,
            y,
            z,
            dimension: dimension.into(),
        });
        id
    }

    /// Deletes a waypoint by ID
    pub fn remove_waypoint(&mut self, id: usize) -> bool {
        if let Some(pos) = self.waypoints.iter().position(|w| w.id == id) {
            self.waypoints.remove(pos);
            true
        } else {
            false
        }
    }

    /// Formats chat text to share current coordinates (Déesse UI chat insertcoords)
    pub fn format_coords_for_chat(x: i32, y: i32, z: i32, dimension: &str) -> String {
        format!("📍 Coordenadas: [X: {}, Y: {}, Z: {}] ({})", x, y, z, dimension)
    }

    /// Formats summary lines for in-game pause menu options
    pub fn generate_in_game_menu_lines(&self) -> Vec<String> {
        vec![
            format!("📷 Cámara (F5): [{}]", self.perspective.name()),
            format!("🎮 Modo de Juego: [{}]", self.gamemode.name()),
            format!("👁️ Campo de Visión (FOV): [{}]", self.fov.label()),
            format!("🔭 Distancia Render: [{} Chunks]", self.render_distance),
            format!("📦 Atajos Contenedor: [{}]", self.container_mode.name()),
            format!("📝 Notas Activas: [{}]", self.notes.entries.len()),
            format!("📍 Waypoints Guardados: [{}]", self.waypoints.len()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_and_gamemode_cycling() {
        let mut opts = InGameOptions::new();
        assert_eq!(opts.perspective, CameraPerspective::FirstPerson);
        opts.cycle_perspective();
        assert_eq!(opts.perspective, CameraPerspective::ThirdPersonBack);
        opts.cycle_perspective();
        assert_eq!(opts.perspective, CameraPerspective::ThirdPersonFront);
        opts.cycle_perspective();
        assert_eq!(opts.perspective, CameraPerspective::FirstPerson);

        assert_eq!(opts.gamemode, QuickGamemode::Survival);
        opts.cycle_gamemode();
        assert_eq!(opts.gamemode, QuickGamemode::Creative);
        assert_eq!(opts.gamemode.command_str(), "/gamemode creative");
    }

    #[test]
    fn test_fov_and_render_distance() {
        let mut opts = InGameOptions::new();
        assert_eq!(opts.fov.degrees(), 75.6);
        opts.cycle_fov();
        assert_eq!(opts.fov.degrees(), 110.0);
        opts.cycle_fov();
        assert_eq!(opts.fov.degrees(), 30.0);

        assert_eq!(opts.render_distance, 12);
        opts.cycle_render_distance();
        assert_eq!(opts.render_distance, 16);
    }

    #[test]
    fn test_waypoints_and_notes() {
        let mut opts = InGameOptions::new();
        let w_id = opts.add_waypoint("Portal Principal", 120, 64, -300, "minecraft:overworld");
        assert_eq!(w_id, 1);
        assert_eq!(opts.waypoints.len(), 1);
        assert!(opts.waypoints[0].format_entry().contains("Portal Principal"));

        opts.notes.add("Granja de hierro en X: 500, Z: -200");
        assert_eq!(opts.notes.entries.len(), 1);
        assert!(!opts.notes.is_empty());

        assert!(opts.remove_waypoint(1));
        assert_eq!(opts.waypoints.len(), 0);

        let chat_text = InGameOptions::format_coords_for_chat(10, 64, 20, "Overworld");
        assert!(chat_text.contains("10, Y: 64, Z: 20"));
    }
}
