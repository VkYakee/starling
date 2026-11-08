use super::chunk_scanner::ChunkData;
use super::image_buffer::ImageBuffer;
use crate::color::palette::BlockPalette;

/// 45-degree Axonometric Isometric Projection Renderer (Dynmap & Minecraft-Overviewer standard).
#[derive(Debug, Clone)]
pub struct IsometricRenderer {
    pub palette: BlockPalette,
    pub block_width: u32,  // Default 8 pixels
    pub block_height: u32, // Default 4 pixels
    pub y_step: u32,       // Default 4 pixels vertical displacement per Y
}

impl Default for IsometricRenderer {
    fn default() -> Self {
        Self {
            palette: BlockPalette::new_with_defaults(),
            block_width: 8,
            block_height: 4,
            y_step: 4,
        }
    }
}

impl IsometricRenderer {
    pub fn new(palette: BlockPalette) -> Self {
        Self {
            palette,
            block_width: 8,
            block_height: 4,
            y_step: 4,
        }
    }

    /// Calculates screen position (px, py) for world coordinate (x, y, z) relative to an origin.
    pub fn world_to_screen(&self, x: i32, y: i32, z: i32, origin_x: i32, origin_y: i32) -> (i32, i32) {
        let half_w = (self.block_width / 2) as i32;
        let quart_h = (self.block_height / 2) as i32;

        let sx = origin_x + (x - z) * half_w;
        let sy = origin_y + (x + z) * quart_h - (y * self.y_step as i32);
        (sx, sy)
    }

    /// Renders a chunk into an isometric image buffer.
    pub fn render_isometric_chunk(&self, chunk: &ChunkData) -> ImageBuffer {
        // A 16x16 chunk in isometric takes approximately (16 + 16) * 4 = 128 px wide
        // and (16 + 16) * 2 + 128 px high
        let out_w = 256;
        let out_h = 256;
        let mut buffer = ImageBuffer::new(out_w, out_h);

        let origin_x = (out_w / 2) as i32;
        let origin_y = (out_h / 2) as i32 + 30;

        // Iterate sorted by painter's order (from back to front: ascending x + z)
        for sum in 0..=30 {
            for x in 0..16 {
                let z = sum as i32 - x as i32;
                if z < 0 || z >= 16 {
                    continue;
                }

                let col = chunk.get_column(x, z as u32);
                if let Some((y, block_name)) = col.get_surface_block(320) {
                    let face_color = self.palette.get_color(block_name);
                    let (sx, sy) = self.world_to_screen(x as i32, y, z, origin_x, origin_y);

                    // Draw isometric diamond top face
                    self.draw_isometric_diamond(&mut buffer, sx, sy, face_color.top);

                    // Draw left face (shadowed)
                    self.draw_vertical_face(&mut buffer, sx - 2, sy + 2, 2, 3, face_color.north);
                    // Draw right face (medium shaded)
                    self.draw_vertical_face(&mut buffer, sx, sy + 2, 2, 3, face_color.east);
                }
            }
        }

        buffer
    }

    fn draw_isometric_diamond(&self, buf: &mut ImageBuffer, cx: i32, cy: i32, color: [u8; 4]) {
        // Simple 4x2 isometric diamond
        let offsets = [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1), (-2, 0), (2, 0)];
        for (ox, oy) in offsets {
            let px = cx + ox;
            let py = cy + oy;
            if px >= 0 && px < buf.width as i32 && py >= 0 && py < buf.height as i32 {
                buf.blend_pixel(px as u32, py as u32, color);
            }
        }
    }

    fn draw_vertical_face(&self, buf: &mut ImageBuffer, x: i32, y: i32, w: i32, h: i32, color: [u8; 4]) {
        for oy in 0..h {
            for ox in 0..w {
                let px = x + ox;
                let py = y + oy;
                if px >= 0 && px < buf.width as i32 && py >= 0 && py < buf.height as i32 {
                    buf.blend_pixel(px as u32, py as u32, color);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::chunk_scanner::BlockColumn;

    #[test]
    fn test_isometric_projection_math() {
        let renderer = IsometricRenderer::default();
        let (sx1, sy1) = renderer.world_to_screen(0, 64, 0, 100, 100);
        let (sx2, sy2) = renderer.world_to_screen(1, 64, 0, 100, 100);

        // x increases screen_x and screen_y
        assert!(sx2 > sx1);
        assert!(sy2 > sy1);

        // Higher y decreases screen_y (moves up on screen)
        let (_sx3, sy3) = renderer.world_to_screen(0, 70, 0, 100, 100);
        assert!(sy3 < sy1);
    }

    #[test]
    fn test_isometric_chunk_render() {
        let renderer = IsometricRenderer::default();
        let mut chunk = ChunkData::new(0, 0, "plains");

        for z in 0..16 {
            for x in 0..16 {
                let col = BlockColumn::new("plains")
                    .with_block(64, "stone")
                    .with_block(65, "grass_block");
                chunk.set_column(x, z, col);
            }
        }

        let buf = renderer.render_isometric_chunk(&chunk);
        assert_eq!(buf.width, 256);
        assert_eq!(buf.height, 256);
    }
}
