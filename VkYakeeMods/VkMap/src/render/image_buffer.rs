use std::fs::File;
use std::io::{BufWriter, Cursor};
use std::path::Path;

/// Fast, safe RGBA Image Buffer for rasterizing Minecraft chunks, minimaps, and overlays.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageBuffer {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl ImageBuffer {
    /// Creates a new transparent RGBA buffer.
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width as usize) * (height as usize) * 4;
        Self {
            width,
            height,
            pixels: vec![0; size],
        }
    }

    /// Creates a new RGBA buffer initialized with a solid color.
    pub fn new_filled(width: u32, height: u32, color: [u8; 4]) -> Self {
        let mut buf = Self::new(width, height);
        buf.fill(color);
        buf
    }

    /// Sets an RGBA pixel at (x, y). Out of bounds writes are silently ignored.
    pub fn set_pixel(&mut self, x: u32, y: u32, rgba: [u8; 4]) {
        if x < self.width && y < self.height {
            let idx = ((y * self.width + x) * 4) as usize;
            self.pixels[idx] = rgba[0];
            self.pixels[idx + 1] = rgba[1];
            self.pixels[idx + 2] = rgba[2];
            self.pixels[idx + 3] = rgba[3];
        }
    }

    /// Retrieves the RGBA pixel at (x, y). Out of bounds returns transparent black.
    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        if x < self.width && y < self.height {
            let idx = ((y * self.width + x) * 4) as usize;
            [
                self.pixels[idx],
                self.pixels[idx + 1],
                self.pixels[idx + 2],
                self.pixels[idx + 3],
            ]
        } else {
            [0, 0, 0, 0]
        }
    }

    /// Blends an incoming pixel over the existing pixel using alpha compositing.
    pub fn blend_pixel(&mut self, x: u32, y: u32, fg: [u8; 4]) {
        if fg[3] == 0 {
            return;
        }
        if fg[3] == 255 {
            self.set_pixel(x, y, fg);
            return;
        }

        let bg = self.get_pixel(x, y);
        let alpha = fg[3] as f32 / 255.0;
        let inv_alpha = 1.0 - alpha;

        let r = ((fg[0] as f32 * alpha) + (bg[0] as f32 * inv_alpha)) as u8;
        let g = ((fg[1] as f32 * alpha) + (bg[1] as f32 * inv_alpha)) as u8;
        let b = ((fg[2] as f32 * alpha) + (bg[2] as f32 * inv_alpha)) as u8;
        let a = (bg[3] as f32 + (255.0 - bg[3] as f32) * alpha).min(255.0) as u8;

        self.set_pixel(x, y, [r, g, b, a]);
    }

    /// Fills the entire buffer with a solid color.
    pub fn fill(&mut self, color: [u8; 4]) {
        for chunk in self.pixels.chunks_exact_mut(4) {
            chunk[0] = color[0];
            chunk[1] = color[1];
            chunk[2] = color[2];
            chunk[3] = color[3];
        }
    }

    /// Blits another image buffer onto this one at `(dst_x, dst_y)`.
    pub fn blit(&mut self, src: &ImageBuffer, dst_x: i32, dst_y: i32) {
        for sy in 0..src.height {
            let dy = dst_y + sy as i32;
            if dy < 0 || dy >= self.height as i32 {
                continue;
            }
            for sx in 0..src.width {
                let dx = dst_x + sx as i32;
                if dx < 0 || dx >= self.width as i32 {
                    continue;
                }
                let color = src.get_pixel(sx, sy);
                self.blend_pixel(dx as u32, dy as u32, color);
            }
        }
    }

    /// Draws a line between two points using Bresenham's line algorithm.
    pub fn draw_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: [u8; 4], thickness: u32) {
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        let half_t = (thickness as i32) / 2;

        loop {
            // Plot thickness kernel
            for oy in -half_t..=half_t {
                for ox in -half_t..=half_t {
                    let px = x0 + ox;
                    let py = y0 + oy;
                    if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                        self.blend_pixel(px as u32, py as u32, color);
                    }
                }
            }

            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Draws a rectangle outline or filled rectangle.
    pub fn draw_rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 4], filled: bool) {
        let min_x = x0.min(x1).max(0) as u32;
        let max_x = x0.max(x1).min(self.width as i32 - 1) as u32;
        let min_y = y0.min(y1).max(0) as u32;
        let max_y = y0.max(y1).min(self.height as i32 - 1) as u32;

        if filled {
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    self.blend_pixel(x, y, color);
                }
            }
        } else {
            for x in min_x..=max_x {
                self.blend_pixel(x, min_y, color);
                self.blend_pixel(x, max_y, color);
            }
            for y in min_y..=max_y {
                self.blend_pixel(min_x, y, color);
                self.blend_pixel(max_x, y, color);
            }
        }
    }

    /// Draws a circle with center `(cx, cy)` and radius `r`.
    pub fn draw_circle(&mut self, cx: i32, cy: i32, radius: i32, color: [u8; 4], filled: bool) {
        let r_sq = radius * radius;
        let r_inner_sq = (radius - 1) * (radius - 1);

        let min_x = (cx - radius).max(0) as u32;
        let max_x = (cx + radius).min(self.width as i32 - 1) as u32;
        let min_y = (cy - radius).max(0) as u32;
        let max_y = (cy + radius).min(self.height as i32 - 1) as u32;

        for y in min_y..=max_y {
            let dy = y as i32 - cy;
            for x in min_x..=max_x {
                let dx = x as i32 - cx;
                let dist_sq = dx * dx + dy * dy;
                if filled {
                    if dist_sq <= r_sq {
                        self.blend_pixel(x, y, color);
                    }
                } else if dist_sq <= r_sq && dist_sq >= r_inner_sq {
                    self.blend_pixel(x, y, color);
                }
            }
        }
    }

    /// Encodes the buffer into raw PNG bytes.
    pub fn to_png_bytes(&self) -> Result<Vec<u8>, String> {
        let mut buffer = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut buffer, self.width, self.height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
            writer.write_image_data(&self.pixels).map_err(|e| e.to_string())?;
        }
        Ok(buffer)
    }

    /// Saves the image buffer as a PNG file.
    pub fn save_png<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let file = File::create(path).map_err(|e| e.to_string())?;
        let ref mut w = BufWriter::new(file);
        let mut encoder = png::Encoder::new(w, self.width, self.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer.write_image_data(&self.pixels).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Decodes an image from PNG bytes into an RGBA `ImageBuffer`.
    pub fn from_png_bytes(bytes: &[u8]) -> Result<Self, String> {
        let cursor = Cursor::new(bytes);
        let decoder = png::Decoder::new(cursor);
        let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
        let buf_size = reader
            .output_buffer_size()
            .unwrap_or_else(|| (reader.info().width * reader.info().height * 4) as usize);
        let mut buf = vec![0; buf_size];
        let info = reader.next_frame(&mut buf).map_err(|e| e.to_string())?;
        
        let width = info.width;
        let height = info.height;
        let bytes = &buf[..info.buffer_size()];

        let rgba_pixels = match info.color_type {
            png::ColorType::Rgba => bytes.to_vec(),
            png::ColorType::Rgb => {
                let mut out = Vec::with_capacity((width * height * 4) as usize);
                for rgb in bytes.chunks_exact(3) {
                    out.push(rgb[0]);
                    out.push(rgb[1]);
                    out.push(rgb[2]);
                    out.push(255);
                }
                out
            }
            png::ColorType::Grayscale => {
                let mut out = Vec::with_capacity((width * height * 4) as usize);
                for &g in bytes {
                    out.push(g);
                    out.push(g);
                    out.push(g);
                    out.push(255);
                }
                out
            }
            png::ColorType::GrayscaleAlpha => {
                let mut out = Vec::with_capacity((width * height * 4) as usize);
                for ga in bytes.chunks_exact(2) {
                    out.push(ga[0]);
                    out.push(ga[0]);
                    out.push(ga[0]);
                    out.push(ga[1]);
                }
                out
            }
            _ => return Err(format!("Unsupported PNG color type: {:?}", info.color_type)),
        };

        Ok(Self {
            width,
            height,
            pixels: rgba_pixels,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_buffer_basic() {
        let mut buf = ImageBuffer::new(16, 16);
        assert_eq!(buf.get_pixel(0, 0), [0, 0, 0, 0]);

        buf.set_pixel(0, 0, [255, 0, 0, 255]);
        assert_eq!(buf.get_pixel(0, 0), [255, 0, 0, 255]);

        buf.draw_rect(2, 2, 6, 6, [0, 255, 0, 255], true);
        assert_eq!(buf.get_pixel(3, 3), [0, 255, 0, 255]);
    }

    #[test]
    fn test_png_roundtrip() {
        let mut buf = ImageBuffer::new(32, 32);
        buf.draw_circle(16, 16, 8, [0, 0, 255, 255], true);

        let png_bytes = buf.to_png_bytes().expect("PNG encoding failed");
        assert!(!png_bytes.is_empty());

        let decoded = ImageBuffer::from_png_bytes(&png_bytes).expect("PNG decoding failed");
        assert_eq!(decoded.width, 32);
        assert_eq!(decoded.height, 32);
        assert_eq!(decoded.get_pixel(16, 16), [0, 0, 255, 255]);
    }
}
