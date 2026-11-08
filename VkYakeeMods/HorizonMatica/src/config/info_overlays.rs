use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoOverlayConfigs {
    pub show_info_hud: bool,
    pub show_tool_mode: bool,
    pub show_selection_info: bool,
    pub show_targeted_block: bool,
    pub show_layer_info: bool,
    pub verifier_overlay_enabled: bool,
}

impl Default for InfoOverlayConfigs {
    fn default() -> Self {
        Self {
            show_info_hud: true,
            show_tool_mode: true,
            show_selection_info: true,
            show_targeted_block: true,
            show_layer_info: true,
            verifier_overlay_enabled: true,
        }
    }
}
