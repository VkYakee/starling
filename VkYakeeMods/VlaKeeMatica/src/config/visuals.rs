//! Visual settings for VlaKeeMatica

use serde::{Deserialize, Serialize};

/// Visual configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Visuals {
    // Hologram rendering
    pub show_hologram: bool,
    pub show_hologram_outline: bool,
    pub show_hologram_ghost_blocks: bool,
    pub hologram_opacity: f32,
    pub hologram_line_width: f32,
    
    // Mini-render
    pub show_mini_render: bool,
    pub mini_render_mode: MiniRenderMode,
    pub mini_render_size: u32,
    
    // Verifier overlay
    pub show_verifier_overlay: bool,
    pub verifier_highlight_missing: bool,
    pub verifier_highlight_extra: bool,
    pub verifier_highlight_wrong: bool,
    
    // Selection
    pub show_selection_box: bool,
    pub selection_box_thickness: f32,
    
    // Material list
    pub show_material_list: bool,
    pub material_list_max_entries: usize,
    
    // Info display
    pub show_coordinates: bool,
    pub show_block_info: bool,
    pub show_performance: bool,
}

/// Mini-render mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MiniRenderMode {
    Isometric3D,
    LayerSlice2D,
}

impl Default for MiniRenderMode {
    fn default() -> Self {
        MiniRenderMode::Isometric3D
    }
}

impl Default for Visuals {
    fn default() -> Self {
        Self {
            // Hologram
            show_hologram: true,
            show_hologram_outline: true,
            show_hologram_ghost_blocks: true,
            hologram_opacity: 0.75,
            hologram_line_width: 2.0,
            
            // Mini-render
            show_mini_render: true,
            mini_render_mode: MiniRenderMode::Isometric3D,
            mini_render_size: 140,
            
            // Verifier
            show_verifier_overlay: true,
            verifier_highlight_missing: true,
            verifier_highlight_extra: true,
            verifier_highlight_wrong: true,
            
            // Selection
            show_selection_box: true,
            selection_box_thickness: 2.0,
            
            // Materials
            show_material_list: true,
            material_list_max_entries: 50,
            
            // Info
            show_coordinates: true,
            show_block_info: true,
            show_performance: false,
        }
    }
}
