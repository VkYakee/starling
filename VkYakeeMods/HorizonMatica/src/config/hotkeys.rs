use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfigs {
    pub open_gui_main_menu: String,
    pub open_gui_schematic_placements: String,
    pub open_gui_loaded_schematics: String,
    pub open_gui_material_list: String,
    pub open_gui_schematic_verifier: String,
    pub easy_place_toggle: String,
    pub pick_block: String,
    pub layer_mode_next: String,
    pub layer_mode_prev: String,
    pub layer_next: String,
    pub layer_prev: String,
    pub nudge_modifier: String,
}

impl Default for HotkeyConfigs {
    fn default() -> Self {
        Self {
            open_gui_main_menu: "M".to_string(),
            open_gui_schematic_placements: "M,P".to_string(),
            open_gui_loaded_schematics: "M,L".to_string(),
            open_gui_material_list: "M,L".to_string(),
            open_gui_schematic_verifier: "M,V".to_string(),
            easy_place_toggle: "V".to_string(),
            pick_block: "MOUSE_MIDDLE".to_string(),
            layer_mode_next: "M,PAGE_UP".to_string(),
            layer_mode_prev: "M,PAGE_DOWN".to_string(),
            layer_next: "PAGE_UP".to_string(),
            layer_prev: "PAGE_DOWN".to_string(),
            nudge_modifier: "LEFT_ALT".to_string(),
        }
    }
}
