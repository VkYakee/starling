//! Quick Toggles and Convenience Features ported from Déesse UI for Starling.
//! Manages Night Vision brightness, Recovery Compass (death coords), Clock/Compass,
//! and UI panel visibility options.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickToggles {
    pub night_vision: bool,
    pub brightness_level: f32, // 0.0 to 1.0 (default 0.75 when on)
    pub recovery_compass: bool,
    pub last_death_pos: Option<(i32, i32, i32)>,
    pub clock_compass: bool,
    pub hide_chat: bool,
    pub hide_scoreboard: bool,
    pub hide_hud: bool,
    pub empty_slot_counter: bool,
    pub show_day_advancement: bool,
}

impl Default for QuickToggles {
    fn default() -> Self {
        Self {
            night_vision: false,
            brightness_level: 0.75,
            recovery_compass: true,
            last_death_pos: None,
            clock_compass: true,
            hide_chat: false,
            hide_scoreboard: false,
            hide_hud: false,
            empty_slot_counter: true,
            show_day_advancement: true,
        }
    }
}

impl QuickToggles {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle_night_vision(&mut self) -> bool {
        self.night_vision = !self.night_vision;
        self.night_vision
    }

    pub fn adjust_brightness(&mut self, delta: f32) {
        self.brightness_level = (self.brightness_level + delta).clamp(0.1, 1.0);
    }

    pub fn cycle_brightness(&mut self) {
        self.brightness_level = if self.brightness_level < 0.3 {
            0.50
        } else if self.brightness_level < 0.6 {
            0.75
        } else if self.brightness_level < 0.85 {
            1.00
        } else {
            0.25
        };
    }

    pub fn record_death(&mut self, x: i32, y: i32, z: i32) {
        self.last_death_pos = Some((x, y, z));
    }

    pub fn distance_to_death(&self, current_x: i32, current_y: i32, current_z: i32) -> Option<f32> {
        self.last_death_pos.map(|(dx, dy, dz)| {
            let rx = (current_x - dx) as f32;
            let ry = (current_y - dy) as f32;
            let rz = (current_z - dz) as f32;
            (rx * rx + ry * ry + rz * rz).sqrt()
        })
    }

    pub fn generate_hud_lines(&self, current_x: i32, current_y: i32, current_z: i32) -> Vec<String> {
        let mut lines = Vec::new();

        // 1. Recovery Compass (Last Death)
        if self.recovery_compass {
            if let Some((dx, dy, dz)) = self.last_death_pos {
                if let Some(dist) = self.distance_to_death(current_x, current_y, current_z) {
                    lines.push(format!("🧭 Brújula de Recuperación: [{}, {}, {}] ({:.0}m)", dx, dy, dz, dist));
                }
            }
        }

        // 2. Night Vision / Brightness status
        if self.night_vision {
            lines.push(format!("💡 Visión Nocturna: ACTIVADA (Brillo: {:.0}%)", self.brightness_level * 100.0));
        }

        // 3. UI states
        if self.hide_chat {
            lines.push("💬 Chat: [OCULTO]".to_string());
        }
        if self.hide_scoreboard {
            lines.push("📋 Scoreboard: [OCULTO]".to_string());
        }

        lines
    }
}
