//! Mini-Renderer for HorizonMatica schematics.
//! Generates 2.5D Isometric 3D projections and 2D Layer-by-Layer floorplan previews
//! for real-time visualization directly on the mobile in-game HUD.

use super::litematic::LitematicaSchematic;
use super::palette::BlockState;
use crate::util::pos::Rotation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewMode {
    Isometric3D,
    LayerSlice2D,
}

pub struct SchematicMiniRenderer;

impl SchematicMiniRenderer {
    /// Renders a dynamic text-based mini-render of the schematic to embed in the HUD.
    /// If `layer_filter` is Some(y), renders a detailed 2D cross-section of that layer.
    /// If None, renders an isometric 3D voxel projection of the whole structure.
    pub fn render_preview(
        schematic: &LitematicaSchematic,
        layer_filter: Option<i32>,
        rotation: Rotation,
    ) -> Vec<String> {
        if schematic.regions.is_empty() {
            return vec!["  [ Esquemática vacía sin bloques ]".to_string()];
        }

        match layer_filter {
            Some(layer_y) => Self::render_layer_slice(schematic, layer_y),
            None => Self::render_isometric_3d(schematic, rotation),
        }
    }

    /// Generates an Isometric 3D projection of the schematic.
    fn render_isometric_3d(schematic: &LitematicaSchematic, rotation: Rotation) -> Vec<String> {
        let mut lines = Vec::new();

        // Get the first region or combine bounding dimensions
        let region = match schematic.regions.values().next() {
            Some(r) => r,
            None => return vec!["  [ Sin datos de región ]".to_string()],
        };

        let sx = region.size.x.unsigned_abs() as usize;
        let sy = region.size.y.unsigned_abs() as usize;
        let sz = region.size.z.unsigned_abs() as usize;

        if sx == 0 || sy == 0 || sz == 0 {
            return vec!["  [ Dimensiones de región nulas ]".to_string()];
        }

        // Target canvas size for mobile HUD
        const CANVAS_W: usize = 14;
        const CANVAS_H: usize = 6;
        let mut canvas = vec![vec![' '; CANVAS_W]; CANVAS_H];
        let mut depth_buf = vec![vec![i32::MIN; CANVAS_W]; CANVAS_H];

        // Isometric projection constants
        // u = (x - z), v = (x + z) / 2 - y
        let min_u = -(sz as i32);
        let max_u = sx as i32;
        let span_u = (max_u - min_u).max(1);

        let min_v = -((sy * 2) as i32);
        let max_v = ((sx + sz) as i32) / 2;
        let span_v = (max_v - min_v).max(1);

        // Project non-air blocks into canvas
        for y in 0..sy {
            for z in 0..sz {
                for x in 0..sx {
                    let state = region.container.get(x, y, z);
                    if state.is_air() {
                        continue;
                    }

                    // Apply rotation to local coordinate
                    let (rx, rz) = match rotation {
                        Rotation::None => (x as i32, z as i32),
                        Rotation::Clockwise90 => (z as i32, (sx - 1 - x) as i32),
                        Rotation::Clockwise180 => ((sx - 1 - x) as i32, (sz - 1 - z) as i32),
                        Rotation::CounterClockwise90 => ((sz - 1 - z) as i32, x as i32),
                    };

                    let u = rx - rz;
                    let v = (rx + rz) / 2 - (y as i32);
                    let depth = rx + rz + (y as i32) * 2;

                    // Normalize to canvas coordinates
                    let cu = (((u - min_u) * (CANVAS_W as i32 - 1)) / span_u) as usize;
                    let cv = (((v - min_v) * (CANVAS_H as i32 - 1)) / span_v) as usize;

                    let cu = cu.min(CANVAS_W - 1);
                    let cv = cv.min(CANVAS_H - 1);

                    if depth > depth_buf[cv][cu] {
                        depth_buf[cv][cu] = depth;
                        canvas[cv][cu] = Self::block_to_glyph(&state, y == sy - 1);
                    }
                }
            }
        }

        // Add 3D frame and side metadata
        lines.push("  ┌─── [ 📐 Mini-Render 3D Isometric ] ───┐".to_string());
        for (i, row) in canvas.iter().enumerate() {
            let row_str: String = row.iter().collect();
            let label = match i {
                0 => format!("  ▲ Y (Techo) [{}x{}x{}]", sx, sy, sz),
                1 => format!("  Rotación: {:?}", rotation),
                2 => format!("  Bloques: {}", schematic.metadata.total_blocks),
                3 => "  Capas: [Todas visibles]".to_string(),
                _ => String::new(),
            };
            lines.push(format!("  │{:14}│ {}", row_str, label));
        }
        lines.push("  └───────────────────────────────────────┘".to_string());

        lines
    }

    /// Generates a 2D top-down slice of a specific Y layer.
    fn render_layer_slice(schematic: &LitematicaSchematic, layer_y: i32) -> Vec<String> {
        let mut lines = Vec::new();
        let region = match schematic.regions.values().next() {
            Some(r) => r,
            None => return vec!["  [ Sin datos de región ]".to_string()],
        };

        let sx = region.size.x.unsigned_abs() as usize;
        let sy = region.size.y.unsigned_abs() as usize;
        let sz = region.size.z.unsigned_abs() as usize;

        let y_idx = (layer_y - region.position.y).clamp(0, (sy as i32 - 1).max(0)) as usize;

        const GRID_W: usize = 14;
        const GRID_H: usize = 6;
        let mut canvas = vec![vec!['·'; GRID_W]; GRID_H];

        let mut layer_block_count = 0;

        for z in 0..sz {
            for x in 0..sx {
                let state = region.container.get(x, y_idx, z);
                if !state.is_air() {
                    layer_block_count += 1;
                    let gx = (x * (GRID_W - 1)) / (sx.max(1));
                    let gz = (z * (GRID_H - 1)) / (sz.max(1));
                    canvas[gz.min(GRID_H - 1)][gx.min(GRID_W - 1)] = Self::block_to_glyph(&state, false);
                }
            }
        }

        lines.push(format!("  ┌─── [ 📐 Capa 2D: Y={} (Plano) ] ───┐", layer_y));
        for (i, row) in canvas.iter().enumerate() {
            let row_str: String = row.iter().collect();
            let label = match i {
                0 => format!("  Bloques en capa: {}", layer_block_count),
                1 => format!("  Nivel Y relativo: {}/{}", y_idx + 1, sy),
                2 => "  Guía: [█ Sólido, ░ Vidrio, · Aire]".to_string(),
                _ => String::new(),
            };
            lines.push(format!("  │{:14}│ {}", row_str, label));
        }
        lines.push("  └──────────────────────────────────────┘".to_string());

        lines
    }

    /// Maps a block state to a distinctive Unicode glyph for the mini-render.
    fn block_to_glyph(state: &BlockState, is_top: bool) -> char {
        let name = state.name.as_str();
        if name.contains("glass") || name.contains("pane") {
            '░'
        } else if name.contains("log") || name.contains("stem") || name.contains("wood") {
            '▓'
        } else if name.contains("stair") || name.contains("slab") {
            '▄'
        } else if name.contains("chest") || name.contains("door") || name.contains("lantern") {
            '■'
        } else if name.contains("leaf") || name.contains("leaves") || name.contains("plant") {
            '▒'
        } else if is_top {
            '▀'
        } else {
            '█'
        }
    }
}
