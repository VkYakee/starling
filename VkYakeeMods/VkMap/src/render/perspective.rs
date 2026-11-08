use serde::{Deserialize, Serialize};

/// Map Render Perspective and Slicing Modes (combining Dynmap, Overviewer, Squaremap, and VoxelMap).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderMode {
    /// Classic 2D top-down orthogonal map without elevation shading.
    TopDownFlat,
    /// High-performance 2D top-down map with dynamic 3D elevation slope shading (Squaremap / Pl3xMap style).
    TopDownShaded,
    /// 45-degree axonometric isometric projection showing 3D block cubes (Dynmap / Minecraft-Overviewer style).
    Isometric,
    /// Underground horizontal slice rendering for cave, mineshaft, and dungeon exploration (Dynmap Cave mode).
    CaveMode {
        /// Maximum vertical Y layer to render (blocks above this are stripped).
        slice_y: i32,
        /// Number of blocks depth to look down.
        depth_blocks: u8,
    },
    /// Nether mode with top bedrock ceiling stripped (Y >= 127) to expose the underworld.
    NetherCeilingCut,
}

impl Default for RenderMode {
    fn default() -> Self {
        RenderMode::TopDownShaded
    }
}

impl RenderMode {
    /// Returns a human-readable title of the render mode.
    pub fn name(&self) -> &'static str {
        match self {
            RenderMode::TopDownFlat => "Superficie 2D Plana",
            RenderMode::TopDownShaded => "Superficie 2D Sombreada (Relieve)",
            RenderMode::Isometric => "Isométrico 3D (Ángulo 45°)",
            RenderMode::CaveMode { .. } => "Modo Cuevas (Corte Subterráneo)",
            RenderMode::NetherCeilingCut => "Nether Sin Techo",
        }
    }

    /// Determines if elevation shading should be applied for this mode.
    pub fn uses_elevation_shading(&self) -> bool {
        matches!(self, RenderMode::TopDownShaded)
    }

    /// Determines if isometric projection math is needed.
    pub fn is_isometric(&self) -> bool {
        matches!(self, RenderMode::Isometric)
    }

    /// Determines if underground slicing is active.
    pub fn is_cave(&self) -> bool {
        matches!(self, RenderMode::CaveMode { .. })
    }
}
