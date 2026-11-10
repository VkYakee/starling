//! Configuration system for VlaKeeMatica

pub mod colors;
pub mod hotkeys;
pub mod info_overlays;
pub mod visuals;
pub mod generic;

pub use colors::*;
pub use hotkeys::*;
pub use info_overlays::*;
pub use visuals::*;
pub use generic::*;

use serde::{Deserialize, Serialize};
use crate::util::pos::Axis;
use crate::util::layer::LayerMode;
use crate::util::easy_place::EasyPlaceProtocol;

/// Main configuration for VlaKeeMatica
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VlaKeeMaticaConfig {
    pub hotkeys: Hotkeys,
    pub visuals: Visuals,
    pub colors: Colors,
    pub info_overlays: InfoOverlays,
    pub generic: GenericConfig,
}

impl Default for VlaKeeMaticaConfig {
    fn default() -> Self {
        Self {
            hotkeys: Hotkeys::default(),
            visuals: Visuals::default(),
            colors: Colors::default(),
            info_overlays: InfoOverlays::default(),
            generic: GenericConfig::default(),
        }
    }
}

/// Generic settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericConfig {
    pub enabled: bool,
    pub easy_place_protocol: EasyPlaceProtocol,
    pub layer_mode: LayerMode,
    pub layer_axis: Axis,
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
            easy_place_protocol: EasyPlaceProtocol::V2,
            layer_mode: LayerMode::All,
            layer_axis: Axis::Y,
            default_opacity: 0.75,
            auto_save_projects: true,
            show_hologram_by_default: false,
            compact_hud_mode: true,
            language: "es".to_string(),
        }
    }
}
