use crate::render::image_buffer::ImageBuffer;
use serde::{Deserialize, Serialize};

/// Hierarchical Multi-resolution Tile Coordinate (BlueMap & Squaremap LOD pyramid standard).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileCoord {
    pub zoom: u8, // 0 = native 1:1, 1 = 1:2, 2 = 1:4, etc.
    pub x: i32,
    pub z: i32,
}

impl TileCoord {
    pub fn new(zoom: u8, x: i32, z: i32) -> Self {
        Self { zoom, x, z }
    }

    /// Converts block coordinates to chunk coordinates (16x16 blocks).
    pub fn block_to_chunk(bx: i32, bz: i32) -> (i32, i32) {
        (bx >> 4, bz >> 4)
    }

    /// Converts chunk coordinates to MCA region coordinates (32x32 chunks / 512x512 blocks).
    pub fn chunk_to_region(cx: i32, cz: i32) -> (i32, i32) {
        (cx >> 5, cz >> 5)
    }

    /// Returns the 4 child tiles at the next higher zoom level (`zoom - 1`).
    pub fn children(&self) -> Option<[TileCoord; 4]> {
        if self.zoom == 0 {
            return None;
        }
        let next_z = self.zoom - 1;
        let x2 = self.x * 2;
        let z2 = self.z * 2;
        Some([
            TileCoord::new(next_z, x2, z2),
            TileCoord::new(next_z, x2 + 1, z2),
            TileCoord::new(next_z, x2, z2 + 1),
            TileCoord::new(next_z, x2 + 1, z2 + 1),
        ])
    }

    /// Downsamples a 2x2 grid of 16x16 child buffers into a single 16x16 lower-resolution buffer.
    pub fn downsample_2x2(
        top_left: &ImageBuffer,
        top_right: &ImageBuffer,
        bottom_left: &ImageBuffer,
        bottom_right: &ImageBuffer,
    ) -> ImageBuffer {
        let mut out = ImageBuffer::new(16, 16);

        let downsample_quad = |src: &ImageBuffer, target: &mut ImageBuffer, start_x: u32, start_y: u32| {
            for y in 0..8 {
                for x in 0..8 {
                    // Average 2x2 block of pixels in src
                    let p00 = src.get_pixel(x * 2, y * 2);
                    let p10 = src.get_pixel(x * 2 + 1, y * 2);
                    let p01 = src.get_pixel(x * 2, y * 2 + 1);
                    let p11 = src.get_pixel(x * 2 + 1, y * 2 + 1);

                    let avg_r = ((p00[0] as u32 + p10[0] as u32 + p01[0] as u32 + p11[0] as u32) / 4) as u8;
                    let avg_g = ((p00[1] as u32 + p10[1] as u32 + p01[1] as u32 + p11[1] as u32) / 4) as u8;
                    let avg_b = ((p00[2] as u32 + p10[2] as u32 + p01[2] as u32 + p11[2] as u32) / 4) as u8;
                    let avg_a = ((p00[3] as u32 + p10[3] as u32 + p01[3] as u32 + p11[3] as u32) / 4) as u8;

                    target.set_pixel(start_x + x, start_y + y, [avg_r, avg_g, avg_b, avg_a]);
                }
            }
        };

        downsample_quad(top_left, &mut out, 0, 0);
        downsample_quad(top_right, &mut out, 8, 0);
        downsample_quad(bottom_left, &mut out, 0, 8);
        downsample_quad(bottom_right, &mut out, 8, 8);

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_coordinates_and_downsampling() {
        assert_eq!(TileCoord::block_to_chunk(15, 15), (0, 0));
        assert_eq!(TileCoord::block_to_chunk(16, 32), (1, 2));
        assert_eq!(TileCoord::chunk_to_region(31, 31), (0, 0));
        assert_eq!(TileCoord::chunk_to_region(32, 64), (1, 2));

        let red = ImageBuffer::new_filled(16, 16, [255, 0, 0, 255]);
        let blue = ImageBuffer::new_filled(16, 16, [0, 0, 255, 255]);
        let green = ImageBuffer::new_filled(16, 16, [0, 255, 0, 255]);
        let yellow = ImageBuffer::new_filled(16, 16, [255, 255, 0, 255]);

        let lod = TileCoord::downsample_2x2(&red, &blue, &green, &yellow);
        assert_eq!(lod.get_pixel(0, 0), [255, 0, 0, 255]);
        assert_eq!(lod.get_pixel(8, 0), [0, 0, 255, 255]);
        assert_eq!(lod.get_pixel(0, 8), [0, 255, 0, 255]);
        assert_eq!(lod.get_pixel(8, 8), [255, 255, 0, 255]);
    }
}
