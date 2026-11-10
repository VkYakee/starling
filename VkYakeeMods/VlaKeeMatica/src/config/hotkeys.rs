//! Hotkey configuration for VlaKeeMatica

use serde::{Deserialize, Serialize};

/// A key binding
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyBind {
    pub key: String,
    pub modifiers: Vec<String>, // "ctrl", "shift", "alt"
}

impl KeyBind {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            modifiers: Vec::new(),
        }
    }

    pub fn with_modifiers(mut self, modifiers: Vec<String>) -> Self {
        self.modifiers = modifiers;
        self
    }
}

/// Hotkey categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkeys {
    // Main mod toggle
    pub toggle_mod: KeyBind,
    
    // Hologram controls
    pub toggle_hologram: KeyBind,
    pub toggle_hologram_render: KeyBind,
    pub toggle_locked: KeyBind,
    
    // Placement manipulation
    pub nudge_up: KeyBind,
    pub nudge_down: KeyBind,
    pub nudge_north: KeyBind,
    pub nudge_south: KeyBind,
    pub nudge_east: KeyBind,
    pub nudge_west: KeyBind,
    pub rotate_cw: KeyBind,
    pub rotate_ccw: KeyBind,
    pub mirror_toggle: KeyBind,
    
    // Layer controls
    pub layer_up: KeyBind,
    pub layer_down: KeyBind,
    pub cycle_layer_mode: KeyBind,
    pub cycle_layer_axis: KeyBind,
    
    // Tool modes
    pub cycle_tool_mode: KeyBind,
    pub toggle_easy_place: KeyBind,
    pub cycle_easy_place_protocol: KeyBind,
    
    // Opacity
    pub opacity_up: KeyBind,
    pub opacity_down: KeyBind,
    
    // Verifier
    pub verify_schematic: KeyBind,
    pub check_materials: KeyBind,
    pub export_materials: KeyBind,
    
    // Selection
    pub set_pos1: KeyBind,
    pub set_pos2: KeyBind,
    pub save_selection: KeyBind,
    
    // UI
    pub open_main_menu: KeyBind,
    pub open_tools_drawer: KeyBind,
    pub toggle_mini_render: KeyBind,
}

impl Default for Hotkeys {
    fn default() -> Self {
        Self {
            // Main
            toggle_mod: KeyBind::new("M"),
            
            // Hologram
            toggle_hologram: KeyBind::new("H"),
            toggle_hologram_render: KeyBind::new("R").with_modifiers(vec!["shift".into()]),
            toggle_locked: KeyBind::new("L"),
            
            // Nudge
            nudge_up: KeyBind::new("UP").with_modifiers(vec!["shift".into()]),
            nudge_down: KeyBind::new("DOWN").with_modifiers(vec!["shift".into()]),
            nudge_north: KeyBind::new("UP"),
            nudge_south: KeyBind::new("DOWN"),
            nudge_east: KeyBind::new("RIGHT"),
            nudge_west: KeyBind::new("LEFT"),
            
            // Rotate/Mirror
            rotate_cw: KeyBind::new("R"),
            rotate_ccw: KeyBind::new("R").with_modifiers(vec!["ctrl".into()]),
            mirror_toggle: KeyBind::new("M").with_modifiers(vec!["shift".into()]),
            
            // Layer
            layer_up: KeyBind::new("PAGE_UP"),
            layer_down: KeyBind::new("PAGE_DOWN"),
            cycle_layer_mode: KeyBind::new("L").with_modifiers(vec!["ctrl".into()]),
            cycle_layer_axis: KeyBind::new("X").with_modifiers(vec!["ctrl".into()]),
            
            // Tools
            cycle_tool_mode: KeyBind::new("T"),
            toggle_easy_place: KeyBind::new("E"),
            cycle_easy_place_protocol: KeyBind::new("E").with_modifiers(vec!["shift".into()]),
            
            // Opacity
            opacity_up: KeyBind::new("O"),
            opacity_down: KeyBind::new("O").with_modifiers(vec!["shift".into()]),
            
            // Verifier
            verify_schematic: KeyBind::new("V"),
            check_materials: KeyBind::new("I"),
            export_materials: KeyBind::new("X"),
            
            // Selection
            set_pos1: KeyBind::new("F3").with_modifiers(vec!["ctrl".into()]),
            set_pos2: KeyBind::new("F3").with_modifiers(vec!["shift".into()]),
            save_selection: KeyBind::new("F3").with_modifiers(vec!["alt".into()]),
            
            // UI
            open_main_menu: KeyBind::new("ESCAPE"),
            open_tools_drawer: KeyBind::new("TAB"),
            toggle_mini_render: KeyBind::new("P"),
        }
    }
}
