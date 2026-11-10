//! Info overlay configuration for VlaKeeMatica

use serde::{Deserialize, Serialize};

/// Info overlay settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoOverlays {
    // Schematic info
    pub show_schematic_name: bool,
    pub show_schematic_dimensions: bool,
    pub show_schematic_blocks: bool,
    pub show_schematic_author: bool,
    
    // Placement info
    pub show_placement_origin: bool,
    pub show_placement_rotation: bool,
    pub show_placement_mirror: bool,
    pub show_placement_locked: bool,
    
    // Layer info
    pub show_layer_info: bool,
    pub show_layer_axis: bool,
    pub show_layer_mode: bool,
    
    // Material info
    pub show_material_count: bool,
    pub show_missing_count: bool,
    pub show_shulker_count: bool,
    
    // Performance
    pub show_fps: bool,
    pub show_tps: bool,
    pub show_memory: bool,
    
    // Position
    pub overlay_position: OverlayPosition,
    pub overlay_scale: f32,
    pub overlay_opacity: f32,
}

/// Overlay position presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverlayPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    CenterTop,
    CenterBottom,
    Center,
}

impl Default for OverlayPosition {
    fn default() -> Self {
        OverlayPosition::TopRight
    }
}

impl Default for InfoOverlays {
    fn default() -> Self {
        Self {
            // Schematic
            show_schematic_name: true,
            show_schematic_dimensions: true,
            show_schematic_blocks: true,
            show_schematic_author: false,
            
            // Placement
            show_placement_origin: true,
            show_placement_rotation: true,
            show_placement_mirror: true,
            show_placement_locked: true,
            
            // Layer
            show_layer_info: true,
            show_layer_axis: true,
            show_layer_mode: true,
            
            // Materials
            show_material_count: true,
            show_missing_count: true,
            show_shulker_count: true,
            
            // Performance
            show_fps: false,
            show_tps: false,
            show_memory: false,
            
            // Position
            overlay_position: OverlayPosition::TopRight,
            overlay_scale: 1.0,
            overlay_opacity: 0.85,
        }
    }
}
