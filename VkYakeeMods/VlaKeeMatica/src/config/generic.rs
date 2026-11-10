//! Generic configuration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericConfig {
    pub enabled: bool,
    pub easy_place_protocol: crate::util::easy_place::EasyPlaceProtocol,
    pub layer_mode: crate::util::layer::LayerMode,
    pub layer_axis: crate::util::pos::Axis,
    pub default_opacity: f32,
    pub auto_save_projects: bool,
    pub show_hologram_by_default: bool,
    pub compact_hud_mode: bool,
    pub language: String,
}

impl Default for GenericConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            easy_place_protocol: crate::util::easy_place::EasyPlaceProtocol::V2,
            layer_mode: crate::util::layer::LayerMode::All,
            layer_axis: crate::util::pos::Axis::Y,
            default_opacity: 0.75,
            auto_save_projects: true,
            show_hologram_by_default: false,
            compact_hud_mode: true,
            language: "es".to_string(),
        }
    }
}
