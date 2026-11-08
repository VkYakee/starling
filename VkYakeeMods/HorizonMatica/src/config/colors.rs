use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColorRgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorRgba {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn hex(hex: u32) -> Self {
        let a = ((hex >> 24) & 0xFF) as f32 / 255.0;
        let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let b = (hex & 0xFF) as f32 / 255.0;
        Self { r, g, b, a }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConfigs {
    pub missing_block: ColorRgba,
    pub extra_block: ColorRgba,
    pub wrong_block: ColorRgba,
    pub wrong_state: ColorRgba,
    pub selection_box: ColorRgba,
    pub placement_box: ColorRgba,
}

impl Default for ColorConfigs {
    fn default() -> Self {
        Self {
            missing_block: ColorRgba::new(1.0, 0.0, 0.0, 0.5),     // Red
            extra_block: ColorRgba::new(1.0, 0.0, 1.0, 0.5),       // Magenta
            wrong_block: ColorRgba::new(1.0, 0.5, 0.0, 0.5),       // Orange
            wrong_state: ColorRgba::new(1.0, 1.0, 0.0, 0.5),       // Yellow
            selection_box: ColorRgba::new(0.0, 0.8, 1.0, 1.0),     // Cyan
            placement_box: ColorRgba::new(0.2, 1.0, 0.2, 1.0),     // Green
        }
    }
}
