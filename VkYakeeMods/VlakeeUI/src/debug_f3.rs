//! SmartF3 and Debug Screen engine ported from Déesse UI for Starling.
//! Provides comprehensive in-game telemetry, coordinates, nether portal conversion,
//! facing angles, moon phases, day counter, and biome diagnostics.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum F3Mode {
    #[default]
    Disabled,
    /// Clean, compact HUD telemetry (Coordinates, Nether conversion, Facing, Biome, FPS)
    Compact,
    /// Complete technical F3 screen with detailed chunk, light, moon, and system stats
    Full,
}

impl F3Mode {
    pub fn next(&self) -> Self {
        match self {
            F3Mode::Disabled => F3Mode::Compact,
            F3Mode::Compact => F3Mode::Full,
            F3Mode::Full => F3Mode::Disabled,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            F3Mode::Disabled => "Desactivado",
            F3Mode::Compact => "Compacto",
            F3Mode::Full => "Completo (SmartF3)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    North, // -Z
    South, // +Z
    West,  // -X
    East,  // +X
}

impl Direction {
    pub fn from_yaw(yaw: f32) -> Self {
        // Normalize yaw to [0, 360)
        let normalized = ((yaw % 360.0) + 360.0) % 360.0;
        if (45.0..135.0).contains(&normalized) {
            Direction::West
        } else if (135.0..225.0).contains(&normalized) {
            Direction::North
        } else if (225.0..315.0).contains(&normalized) {
            Direction::East
        } else {
            Direction::South
        }
    }

    pub fn axis_info(&self) -> &'static str {
        match self {
            Direction::North => "Hacia -Z",
            Direction::South => "Hacia +Z",
            Direction::West => "Hacia -X",
            Direction::East => "Hacia +X",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Direction::North => "Norte (North)",
            Direction::South => "Sur (South)",
            Direction::West => "Oeste (West)",
            Direction::East => "Este (East)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartF3 {
    pub mode: F3Mode,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub dimension: String,
    pub biome_name: String,
    pub block_light: u8,
    pub sky_light: u8,
    pub day_count: u64,
    pub world_time_ticks: u64,
    pub fps: u32,
    pub entity_count: usize,
    pub total_entities: usize,
    pub xp_level: u32,
    pub xp_points: u32,
    pub hide_coordinates: bool,
    pub show_nether_converter: bool,
}

impl Default for SmartF3 {
    fn default() -> Self {
        Self {
            mode: F3Mode::Disabled,
            x: 0.0,
            y: 64.0,
            z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            dimension: "minecraft:overworld".to_string(),
            biome_name: "Plains".to_string(),
            block_light: 0,
            sky_light: 15,
            day_count: 1,
            world_time_ticks: 1000,
            fps: 60,
            entity_count: 12,
            total_entities: 48,
            xp_level: 0,
            xp_points: 0,
            hide_coordinates: false,
            show_nether_converter: true,
        }
    }
}

impl SmartF3 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_pos(&mut self, x: f64, y: f64, z: f64, yaw: f32, pitch: f32) {
        self.x = x;
        self.y = y;
        self.z = z;
        self.yaw = yaw;
        self.pitch = pitch;
    }

    pub fn set_biome(&mut self, name: impl Into<String>) {
        self.biome_name = name.into();
    }

    pub fn set_dimension(&mut self, dim: impl Into<String>) {
        self.dimension = dim.into();
    }

    pub fn set_time(&mut self, world_time: u64) {
        self.world_time_ticks = world_time % 24000;
        self.day_count = (world_time / 24000) + 1;
    }

    pub fn set_xp(&mut self, level: u32, points: u32) {
        self.xp_level = level;
        self.xp_points = points;
    }

    pub fn block_pos(&self) -> (i32, i32, i32) {
        (
            self.x.floor() as i32,
            self.y.floor() as i32,
            self.z.floor() as i32,
        )
    }

    pub fn chunk_pos(&self) -> (i32, i32) {
        let (bx, _, bz) = self.block_pos();
        (bx >> 4, bz >> 4)
    }

    pub fn rel_chunk_pos(&self) -> (i32, i32, i32) {
        let (bx, by, bz) = self.block_pos();
        (bx & 15, by & 15, bz & 15)
    }

    pub fn direction(&self) -> Direction {
        Direction::from_yaw(self.yaw)
    }

    /// Computes matching Nether or Overworld portal coordinates.
    /// In Overworld: (X / 8, Y, Z / 8)
    /// In Nether: (X * 8, Y, Z * 8)
    pub fn portal_target_pos(&self) -> (i32, i32, i32, &'static str) {
        let (bx, by, bz) = self.block_pos();
        if self.dimension.contains("nether") {
            (bx * 8, by, bz * 8, "Overworld Portal Target")
        } else {
            (bx / 8, by, bz / 8, "Nether Portal Target")
        }
    }

    /// Moon phase string based on day count (8-phase lunar cycle).
    pub fn moon_phase(&self) -> (&'static str, usize) {
        let phase = ((self.day_count - 1) % 8) as usize;
        let name = match phase {
            0 => "Luna Llena (Full Moon) §a[!]",
            1 => "Menguante Gibosa (Waning Gibbous)",
            2 => "Cuarto Menguante (Third Quarter)",
            3 => "Luna Menguante (Waning Crescent)",
            4 => "Luna Nueva (New Moon)",
            5 => "Luna Creciente (Waxing Crescent)",
            6 => "Cuarto Creciente (First Quarter)",
            7 => "Creciente Gibosa (Waxing Gibbous)",
            _ => "Luna Llena",
        };
        (name, phase)
    }

    /// Formats in-game clock time (e.g. 06:00, 12:30).
    pub fn in_game_clock(&self) -> String {
        // Minecraft 0 ticks is 06:00 AM
        let total_minutes = (self.world_time_ticks as f64 / 24000.0 * 1440.0 + 360.0) as u64 % 1440;
        let hours = total_minutes / 60;
        let minutes = total_minutes % 60;
        format!("{:02}:{:02}", hours, minutes)
    }

    /// Points needed to reach next XP level.
    pub fn xp_needed_for_next_level(&self) -> u32 {
        if self.xp_level < 16 {
            2 * self.xp_level + 7
        } else if self.xp_level < 31 {
            5 * self.xp_level - 38
        } else {
            9 * self.xp_level - 158
        }
    }

    /// Generates lines to render on HUD based on current F3Mode.
    pub fn generate_hud_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();

        match self.mode {
            F3Mode::Disabled => {}

            F3Mode::Compact => {
                let (bx, by, bz) = self.block_pos();
                let dir = self.direction();
                let clock = self.in_game_clock();

                if self.hide_coordinates {
                    lines.push("┌─ [ 📊 SmartF3: Compacto ] ──────────────────────┐".to_string());
                    lines.push("│ Coordenadas: [OCULTAS]                          │".to_string());
                } else {
                    lines.push("┌─ [ 📊 SmartF3: Compacto ] ──────────────────────┐".to_string());
                    lines.push(format!(
                        "│ XYZ: {:>5} / {:>3} / {:>5} | {} ({})",
                        bx, by, bz, dir.name(), dir.axis_info()
                    ));
                }

                if self.show_nether_converter {
                    let (nx, ny, nz, label) = self.portal_target_pos();
                    lines.push(format!("│ 🔴 {}: [{}, {}, {}]", label, nx, ny, nz));
                }

                lines.push(format!(
                    "│ Bioma: {:<14} | FPS: {:>3} | Día {} ({})",
                    self.biome_name, self.fps, self.day_count, clock
                ));
                lines.push("└──────────────────────────────────────────────┘".to_string());
            }

            F3Mode::Full => {
                let (bx, by, bz) = self.block_pos();
                let (cx, cz) = self.chunk_pos();
                let (rx, ry, rz) = self.rel_chunk_pos();
                let dir = self.direction();
                let (moon_name, _) = self.moon_phase();
                let clock = self.in_game_clock();
                let needed_xp = self.xp_needed_for_next_level();
                let xp_pct = if needed_xp > 0 {
                    (self.xp_points as f32 / needed_xp as f32 * 100.0).clamp(0.0, 100.0) as u32
                } else {
                    0
                };

                lines.push("┌─ [ 📊 SmartF3: Diagnóstico Completo (Starling Rust) ] ─┐".to_string());
                lines.push("│ Minecraft Starling 1.21.x (x86_64/AArch64 Android)     │".to_string());
                lines.push(format!(
                    "│ FPS: {} | Entidades: {}/{} | Dim: {}",
                    self.fps, self.entity_count, self.total_entities, self.dimension
                ));

                if self.hide_coordinates {
                    lines.push("│ Posición: §o[Coordenadas Ocultas]                        │".to_string());
                } else {
                    lines.push(format!("│ XYZ: {:.3} / {:.3} / {:.3}", self.x, self.y, self.z));
                    lines.push(format!("│ Bloque: {} {} {}", bx, by, bz));
                    lines.push(format!("│ Chunk: [{}, {}] en subchunk [{}, {}, {}]", cx, cz, rx, ry, rz));
                }

                lines.push(format!(
                    "│ Mirando: {} (Yaw: {:.1}°, Pitch: {:.1}°) {}",
                    dir.name(), self.yaw, self.pitch, dir.axis_info()
                ));

                if self.show_nether_converter {
                    let (nx, ny, nz, label) = self.portal_target_pos();
                    lines.push(format!("│ 🔴 {}: [{}, {}, {}]", label, nx, ny, nz));
                }

                lines.push(format!(
                    "│ Bioma: {} | Luz: {} (Cielo: {}, Bloque: {})",
                    self.biome_name,
                    self.sky_light.max(self.block_light),
                    self.sky_light,
                    self.block_light
                ));
                lines.push(format!("│ Tiempo: Día {} ({}) | Fase: {}", self.day_count, clock, moon_name));
                lines.push(format!(
                    "│ XP: Nivel {} ({}/{} pts - {}%)",
                    self.xp_level, self.xp_points, needed_xp, xp_pct
                ));
                lines.push("└────────────────────────────────────────────────────────┘".to_string());
            }
        }

        lines
    }
}
