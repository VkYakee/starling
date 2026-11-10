//! Color configuration for VlaKeeMatica

use serde::{Deserialize, Serialize};

/// Color in RGBA format (0xRRGGBBAA)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color(pub u32);

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32))
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::new(r, g, b, a)
    }

    pub fn to_array(&self) -> [f32; 4] {
        [
            ((self.0 >> 24) & 0xFF) as f32 / 255.0,
            ((self.0 >> 16) & 0xFF) as f32 / 255.0,
            ((self.0 >> 8) & 0xFF) as f32 / 255.0,
            (self.0 & 0xFF) as f32 / 255.0,
        ]
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::rgb(255, 255, 255)
    }
}

/// Color scheme for VlaKeeMatica
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Colors {
    // Hologram colors
    pub hologram_correct: Color,
    pub hologram_missing: Color,
    pub hologram_extra: Color,
    pub hologram_wrong_block: Color,
    pub hologram_wrong_state: Color,
    pub hologram_outline: Color,
    pub hologram_ghost: Color,

    // HUD colors
    pub hud_background: Color,
    pub hud_text: Color,
    pub hud_accent: Color,
    pub hud_warning: Color,
    pub hud_error: Color,
    pub hud_success: Color,

    // Mini-render colors
    pub render_solid: Color,
    pub render_glass: Color,
    pub render_wood: Color,
    pub render_stairs: Color,
    pub render_special: Color,
    pub render_organic: Color,
    pub render_top: Color,

    // Selection colors
    pub selection_pos1: Color,
    pub selection_pos2: Color,
    pub selection_box: Color,

    // Verifier colors
    pub verify_correct: Color,
    pub verify_missing: Color,
    pub verify_extra: Color,
    pub verify_wrong_block: Color,
    pub verify_wrong_state: Color,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            // Hologram - Litematica style
            hologram_correct: Color::rgba(0, 255, 0, 180),       // Green - correct
            hologram_missing: Color::rgba(255, 0, 0, 200),        // Red - missing
            hologram_extra: Color::rgba(255, 165, 0, 200),        // Orange - extra
            hologram_wrong_block: Color::rgba(255, 0, 255, 200),  // Magenta - wrong block
            hologram_wrong_state: Color::rgba(255, 255, 0, 200),  // Yellow - wrong state
            hologram_outline: Color::rgba(255, 255, 255, 100),    // White outline
            hologram_ghost: Color::rgba(100, 100, 255, 150),      // Blue ghost

            // HUD
            hud_background: Color::rgba(0, 0, 0, 180),
            hud_text: Color::rgb(255, 255, 255),
            hud_accent: Color::rgb(0, 255, 136),
            hud_warning: Color::rgb(255, 165, 0),
            hud_error: Color::rgb(255, 0, 0),
            hud_success: Color::rgb(0, 255, 0),

            // Mini-render
            render_solid: Color::rgb(200, 200, 200),
            render_glass: Color::rgba(100, 200, 255, 100),
            render_wood: Color::rgb(139, 90, 43),
            render_stairs: Color::rgb(180, 180, 180),
            render_special: Color::rgb(255, 215, 0),
            render_organic: Color::rgb(34, 139, 34),
            render_top: Color::rgb(220, 220, 220),

            // Selection
            selection_pos1: Color::rgba(0, 255, 0, 200),
            selection_pos2: Color::rgba(255, 0, 0, 200),
            selection_box: Color::rgba(0, 200, 255, 150),

            // Verifier
            verify_correct: Color::rgb(0, 255, 0),
            verify_missing: Color::rgb(255, 0, 0),
            verify_extra: Color::rgb(255, 165, 0),
            verify_wrong_block: Color::rgb(255, 0, 255),
            verify_wrong_state: Color::rgb(255, 255, 0),
        }
    }
}
