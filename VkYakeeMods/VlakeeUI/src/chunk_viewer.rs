//! Chunk Viewer & 3D Boundaries engine ported from Déesse UI for Starling.
//! Computes subchunks, chunk boundaries, edge/corner markers, and 3D bounding boxes.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ChunkViewerMode {
    #[default]
    None,
    /// Shows indicator when standing in the middle 2x2 of the chunk (X/Z = 7..8)
    ChunkMiddle,
    /// Shows indicator when standing on the outer edge (X/Z = 0 or 15)
    ChunkEdge,
    /// Shows indicator when standing on chunk corner blocks ((0,0), (0,15), (15,0), (15,15))
    ChunkCorner,
    /// Full 3D wireframe bounding box of current chunk
    FullGrid,
}

impl ChunkViewerMode {
    pub fn next(&self) -> Self {
        match self {
            ChunkViewerMode::None => ChunkViewerMode::ChunkMiddle,
            ChunkViewerMode::ChunkMiddle => ChunkViewerMode::ChunkEdge,
            ChunkViewerMode::ChunkEdge => ChunkViewerMode::ChunkCorner,
            ChunkViewerMode::ChunkCorner => ChunkViewerMode::FullGrid,
            ChunkViewerMode::FullGrid => ChunkViewerMode::None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ChunkViewerMode::None => "Desactivado",
            ChunkViewerMode::ChunkMiddle => "Centro de Chunk",
            ChunkViewerMode::ChunkEdge => "Borde de Chunk",
            ChunkViewerMode::ChunkCorner => "Esquina de Chunk",
            ChunkViewerMode::FullGrid => "Rejilla 3D Completa",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkViewer {
    pub mode: ChunkViewerMode,
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub rel_x: i32,
    pub rel_y: i32,
    pub rel_z: i32,
    pub player_y: i32,
}

impl Default for ChunkViewer {
    fn default() -> Self {
        Self {
            mode: ChunkViewerMode::None,
            chunk_x: 0,
            chunk_z: 0,
            rel_x: 0,
            rel_y: 64,
            rel_z: 0,
            player_y: 64,
        }
    }
}

impl ChunkViewer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_pos(&mut self, block_x: i32, block_y: i32, block_z: i32) {
        self.chunk_x = block_x >> 4;
        self.chunk_z = block_z >> 4;
        self.rel_x = block_x.rem_euclid(16);
        self.rel_y = block_y;
        self.rel_z = block_z.rem_euclid(16);
        self.player_y = block_y;
    }

    pub fn is_at_middle(&self) -> bool {
        (7..=8).contains(&self.rel_x) && (7..=8).contains(&self.rel_z)
    }

    pub fn is_at_edge(&self) -> bool {
        self.rel_x == 0 || self.rel_x == 15 || self.rel_z == 0 || self.rel_z == 15
    }

    pub fn is_at_corner(&self) -> bool {
        (self.rel_x == 0 || self.rel_x == 15) && (self.rel_z == 0 || self.rel_z == 15)
    }

    pub fn status_label(&self) -> Option<&'static str> {
        match self.mode {
            ChunkViewerMode::None => None,
            ChunkViewerMode::ChunkMiddle if self.is_at_middle() => Some("🎯 Centro del Chunk [7..8]"),
            ChunkViewerMode::ChunkEdge if self.is_at_edge() => Some("🚧 Borde del Chunk [0/15]"),
            ChunkViewerMode::ChunkCorner if self.is_at_corner() => Some("📐 Esquina del Chunk [0/15]"),
            ChunkViewerMode::FullGrid => Some("🌐 Rejilla de Chunks Activa"),
            _ => None,
        }
    }

    /// Generates 3D bounding boxes `[min_x, min_y, min_z, max_x, max_y, max_z, r, g, b, a]`
    /// for Starling Vulkan render pass.
    pub fn render_bounding_boxes(&self) -> Vec<[f32; 10]> {
        if self.mode == ChunkViewerMode::None {
            return Vec::new();
        }

        let mut boxes = Vec::new();
        let min_x = (self.chunk_x * 16) as f32;
        let min_z = (self.chunk_z * 16) as f32;
        let max_x = min_x + 16.0;
        let max_z = min_z + 16.0;

        let y_bottom = -64.0f32;
        let y_top = 320.0f32;
        let py = self.player_y as f32;

        match self.mode {
            ChunkViewerMode::None => {}

            ChunkViewerMode::FullGrid => {
                // Entire chunk column box: Light Blue (R: 0.2, G: 0.8, B: 1.0, A: 0.35)
                boxes.push([min_x, y_bottom, min_z, max_x, y_top, max_z, 0.2, 0.8, 1.0, 0.35]);

                // Current subchunk (16x16x16) highlight: Yellow (R: 1.0, G: 0.9, B: 0.1, A: 0.5)
                let sub_y = (self.player_y >> 4) * 16;
                boxes.push([
                    min_x, sub_y as f32, min_z, max_x, (sub_y + 16) as f32, max_z,
                    1.0, 0.9, 0.1, 0.5,
                ]);
            }

            ChunkViewerMode::ChunkMiddle => {
                // Center 2x2 highlight at player feet: Green (R: 0.1, G: 1.0, B: 0.2, A: 0.7)
                let mid_x = min_x + 7.0;
                let mid_z = min_z + 7.0;
                boxes.push([mid_x, py, mid_z, mid_x + 2.0, py + 2.0, mid_z + 2.0, 0.1, 1.0, 0.2, 0.7]);
            }

            ChunkViewerMode::ChunkEdge => {
                // Outer chunk border lines at player Y: Orange (R: 1.0, G: 0.5, B: 0.0, A: 0.6)
                boxes.push([min_x, py, min_z, max_x, py + 1.0, max_z, 1.0, 0.5, 0.0, 0.6]);
            }

            ChunkViewerMode::ChunkCorner => {
                // 4 corner markers at player Y: Magenta (R: 1.0, G: 0.1, B: 0.8, A: 0.8)
                let corners = [
                    (min_x, min_z),
                    (max_x - 1.0, min_z),
                    (min_x, max_z - 1.0),
                    (max_x - 1.0, max_z - 1.0),
                ];
                for (cx, cz) in corners {
                    boxes.push([cx, py, cz, cx + 1.0, py + 2.0, cz + 1.0, 1.0, 0.1, 0.8, 0.8]);
                }
            }
        }

        boxes
    }
}
