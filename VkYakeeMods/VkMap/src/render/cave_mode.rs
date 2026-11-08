use super::chunk_scanner::ChunkData;
use super::image_buffer::ImageBuffer;
use crate::color::palette::BlockPalette;
use crate::color::shading::apply_shade_ratio;

/// Underground Cave Slicing Renderer (Dynmap Cave Shader architecture).
#[derive(Debug, Clone)]
pub struct CaveSliceRenderer {
    pub palette: BlockPalette,
    pub slice_y: i32,
    pub depth_blocks: u8,
}

impl Default for CaveSliceRenderer {
    fn default() -> Self {
        Self {
            palette: BlockPalette::new_with_defaults(),
            slice_y: 60,
            depth_blocks: 48,
        }
    }
}

impl CaveSliceRenderer {
    pub fn new(palette: BlockPalette, slice_y: i32, depth_blocks: u8) -> Self {
        Self {
            palette,
            slice_y,
            depth_blocks,
        }
    }

    /// Renders a 16x16 underground slice of a chunk.
    pub fn render_cave_chunk(&self, chunk: &ChunkData) -> ImageBuffer {
        let mut buffer = ImageBuffer::new(16, 16);
        let min_y = (self.slice_y - self.depth_blocks as i32).max(-64);

        for z in 0..16 {
            for x in 0..16 {
                let col = chunk.get_column(x, z);
                let mut found_floor = false;

                // Look for cave openings starting from slice_y down to min_y
                for (y, block) in &col.blocks {
                    if *y > self.slice_y {
                        continue;
                    }
                    if *y < min_y {
                        break;
                    }

                    if is_solid_or_fluid(block) {
                        let base_color = self.palette.get_top_rgba(block);
                        // Dynmap cave attenuation: blocks deeper below slice_y become darker
                        let depth_diff = (self.slice_y - *y) as f32;
                        let attenuation = (1.0 - (depth_diff / (self.depth_blocks as f32 * 1.2))).clamp(0.2, 1.0);
                        
                        let cave_color = apply_shade_ratio(base_color, attenuation);
                        buffer.set_pixel(x, z, cave_color);
                        found_floor = true;
                        break;
                    }
                }

                if !found_floor {
                    // Solid rock or unexplored void
                    buffer.set_pixel(x, z, [20, 20, 25, 255]);
                }
            }
        }

        buffer
    }
}

fn is_solid_or_fluid(block: &str) -> bool {
    !block.is_empty() && block != "air" && block != "cave_air" && block != "void_air"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::chunk_scanner::BlockColumn;

    #[test]
    fn test_cave_rendering() {
        let renderer = CaveSliceRenderer::default();
        let mut chunk = ChunkData::new(0, 0, "dripstone_caves");

        // Simulate a cave ceiling at Y=64, air at Y=55..63, cave floor at Y=54
        for z in 0..16 {
            for x in 0..16 {
                let col = BlockColumn::new("dripstone_caves")
                    .with_block(64, "stone")
                    .with_block(54, "deepslate")
                    .with_block(50, "bedrock");
                chunk.set_column(x, z, col);
            }
        }

        let buf = renderer.render_cave_chunk(&chunk);
        assert_eq!(buf.width, 16);
        assert_ne!(buf.get_pixel(0, 0), [0, 0, 0, 0]);
    }
}
