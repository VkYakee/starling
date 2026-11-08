use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualConfigs {
    pub enable_rendering: bool,
    pub render_schematic_blocks: bool,
    pub render_ghost_blocks: bool,
    pub render_overlay: bool,
    pub render_verifier_overlay: bool,
    pub render_area_selection_boxes: bool,
    pub render_placement_boxes: bool,
    pub render_colliding_schematic_blocks: bool,
    pub ghost_block_alpha: f32,
    pub overlay_alpha: f32,
}

impl Default for VisualConfigs {
    fn default() -> Self {
        Self {
            enable_rendering: true,
            render_schematic_blocks: true,
            render_ghost_blocks: true,
            render_overlay: true,
            render_verifier_overlay: true,
            render_area_selection_boxes: true,
            render_placement_boxes: true,
            render_colliding_schematic_blocks: true,
            ghost_block_alpha: 0.5,
            overlay_alpha: 0.4,
        }
    }
}
