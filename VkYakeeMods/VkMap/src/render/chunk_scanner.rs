use super::image_buffer::ImageBuffer;
use super::perspective::RenderMode;
use crate::color::biome::BiomePalette;
use crate::color::palette::BlockPalette;
use crate::color::shading::DepthShadingEngine;

/// Represents a single vertical column of blocks at a specific (x, z) in a chunk.
#[derive(Debug, Clone)]
pub struct BlockColumn {
    /// Pairs of `(y_level, block_name)` sorted descending by Y level.
    pub blocks: Vec<(i32, String)>,
    pub biome: String,
}

impl BlockColumn {
    pub fn new(biome: impl Into<String>) -> Self {
        Self {
            blocks: Vec::new(),
            biome: biome.into(),
        }
    }

    pub fn with_block(mut self, y: i32, block: impl Into<String>) -> Self {
        self.blocks.push((y, block.into()));
        self.blocks.sort_by(|a, b| b.0.cmp(&a.0));
        self
    }

    /// Finds the top visible surface block at or below `max_y`.
    pub fn get_surface_block(&self, max_y: i32) -> Option<(i32, &str)> {
        for (y, block) in &self.blocks {
            if *y <= max_y && !is_transparent(block) {
                return Some((*y, block.as_str()));
            }
        }
        None
    }

    /// Computes fluid depth if the surface block is water.
    pub fn get_fluid_depth_and_floor(&self, max_y: i32) -> (u8, &str) {
        let mut depth = 0;
        let mut floor = "sand";

        for (y, block) in &self.blocks {
            if *y > max_y {
                continue;
            }
            if block.contains("water") {
                depth += 1;
            } else if !is_transparent(block) {
                floor = block.as_str();
                break;
            }
        }
        (depth.min(16) as u8, floor)
    }
}

/// Standard 16x16 Minecraft Chunk data structure.
#[derive(Debug, Clone)]
pub struct ChunkData {
    pub chunk_x: i32,
    pub chunk_z: i32,
    /// 16x16 array of block columns indexed by `z * 16 + x`.
    pub columns: Vec<BlockColumn>,
}

impl ChunkData {
    pub fn new(chunk_x: i32, chunk_z: i32, default_biome: &str) -> Self {
        let mut columns = Vec::with_capacity(256);
        for _ in 0..256 {
            columns.push(BlockColumn::new(default_biome));
        }
        Self {
            chunk_x,
            chunk_z,
            columns,
        }
    }

    pub fn get_column(&self, x: u32, z: u32) -> &BlockColumn {
        let idx = ((z.min(15) * 16) + x.min(15)) as usize;
        &self.columns[idx]
    }

    pub fn get_column_mut(&mut self, x: u32, z: u32) -> &mut BlockColumn {
        let idx = ((z.min(15) * 16) + x.min(15)) as usize;
        &mut self.columns[idx]
    }

    pub fn set_column(&mut self, x: u32, z: u32, col: BlockColumn) {
        let idx = ((z.min(15) * 16) + x.min(15)) as usize;
        self.columns[idx] = col;
    }
}

/// High-speed Chunk Scanner & Rasterizer (Squaremap + Pl3xMap + Dynmap architecture).
#[derive(Debug, Clone)]
pub struct ChunkScanner {
    pub block_palette: BlockPalette,
    pub biome_palette: BiomePalette,
    pub shading_engine: DepthShadingEngine,
}

impl Default for ChunkScanner {
    fn default() -> Self {
        Self {
            block_palette: BlockPalette::new_with_defaults(),
            biome_palette: BiomePalette::new_with_defaults(),
            shading_engine: DepthShadingEngine::default(),
        }
    }
}

impl ChunkScanner {
    pub fn new(
        block_palette: BlockPalette,
        biome_palette: BiomePalette,
        shading_engine: DepthShadingEngine,
    ) -> Self {
        Self {
            block_palette,
            biome_palette,
            shading_engine,
        }
    }

    /// Renders a 16x16 chunk into an `ImageBuffer` and returns the heights of the bottom row (z=15)
    /// for seamless northern shading of the adjacent chunk.
    pub fn scan_chunk(
        &self,
        chunk: &ChunkData,
        north_neighbor_heights: Option<&[i32; 16]>,
        mode: RenderMode,
    ) -> (ImageBuffer, [i32; 16]) {
        let mut buffer = ImageBuffer::new(16, 16);
        let mut last_y_row = north_neighbor_heights.copied().unwrap_or([64; 16]);

        let max_render_y = match mode {
            RenderMode::NetherCeilingCut => 126,
            RenderMode::CaveMode { slice_y, .. } => slice_y,
            _ => 320,
        };

        for z in 0..16 {
            let world_z = chunk.chunk_z * 16 + z as i32;

            for x in 0..16 {
                let world_x = chunk.chunk_x * 16 + x as i32;
                let col = chunk.get_column(x, z);

                if let Some((cur_y, block_name)) = col.get_surface_block(max_render_y) {
                    let mut pixel_color = self.block_palette.get_top_rgba(block_name);

                    // Biome tinting
                    if is_grass(block_name) {
                        pixel_color = self.biome_palette.tint_grass(&col.biome, pixel_color);
                    } else if is_foliage(block_name) {
                        pixel_color = self.biome_palette.tint_foliage(&col.biome, pixel_color);
                    } else if is_water(block_name) {
                        pixel_color = self.biome_palette.tint_water(&col.biome, pixel_color);
                        let (depth, floor) = col.get_fluid_depth_and_floor(max_render_y);
                        let floor_color = self.block_palette.get_top_rgba(floor);
                        pixel_color = self.shading_engine.shade_fluid(pixel_color, floor_color, depth);
                    }

                    // Elevation relief shading
                    if mode.uses_elevation_shading() {
                        let prev_y = last_y_row[x as usize];
                        pixel_color = self.shading_engine.shade_elevation(
                            pixel_color,
                            cur_y,
                            prev_y,
                            world_x,
                            world_z,
                        );
                    }

                    last_y_row[x as usize] = cur_y;
                    buffer.set_pixel(x, z, pixel_color);
                } else {
                    // Void or ungenerated
                    buffer.set_pixel(x, z, [0, 0, 0, 0]);
                }
            }
        }

        (buffer, last_y_row)
    }
}

fn is_transparent(block: &str) -> bool {
    block.is_empty() || block == "air" || block == "cave_air" || block == "void_air"
}

fn is_grass(block: &str) -> bool {
    block.contains("grass_block") || block.contains("short_grass") || block.contains("tall_grass")
}

fn is_foliage(block: &str) -> bool {
    block.contains("leaves") || block.contains("vine")
}

fn is_water(block: &str) -> bool {
    block.contains("water")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_scanner_top_down() {
        let scanner = ChunkScanner::default();
        let mut chunk = ChunkData::new(0, 0, "plains");

        // Fill chunk columns with stone and grass
        for z in 0..16 {
            for x in 0..16 {
                let y = 64 + (x as i32 % 3);
                let col = BlockColumn::new("plains")
                    .with_block(y, "grass_block")
                    .with_block(y - 1, "dirt")
                    .with_block(y - 2, "stone");
                chunk.set_column(x, z, col);
            }
        }

        let (buf, bottom_y) = scanner.scan_chunk(&chunk, None, RenderMode::TopDownShaded);
        assert_eq!(buf.width, 16);
        assert_eq!(buf.height, 16);
        assert_ne!(buf.get_pixel(0, 0), [0, 0, 0, 0]);
        assert_eq!(bottom_y[0], 64);
    }
}
