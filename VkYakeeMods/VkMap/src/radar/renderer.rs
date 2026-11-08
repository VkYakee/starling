use super::tracker::{ElevationIndicator, RadarContact};
use crate::render::image_buffer::ImageBuffer;

/// Radar blitter rendering entity contacts onto minimap image buffers.
#[derive(Debug, Clone, Default)]
pub struct RadarRenderer {
    pub dot_radius: i32,
    pub show_elevation_arrows: bool,
}

impl RadarRenderer {
    pub fn new() -> Self {
        Self {
            dot_radius: 2,
            show_elevation_arrows: true,
        }
    }

    /// Renders all visible radar contacts onto a minimap buffer centered at `(center_x, center_y)`.
    pub fn render_radar_overlay(
        &self,
        target_buf: &mut ImageBuffer,
        center_x: i32,
        center_y: i32,
        contacts: &[RadarContact],
    ) {
        for contact in contacts {
            let px = center_x + contact.screen_dx.round() as i32;
            let py = center_y + contact.screen_dy.round() as i32;

            // Check if within buffer bounds
            if px < 2 || px >= target_buf.width as i32 - 2 || py < 2 || py >= target_buf.height as i32 - 2 {
                continue;
            }

            // Draw center contact dot
            target_buf.draw_circle(px, py, self.dot_radius, contact.color, true);

            // Draw elevation indicators
            if self.show_elevation_arrows {
                match contact.elevation {
                    ElevationIndicator::Above => {
                        // Tiny '^' arrow above dot
                        target_buf.set_pixel(px as u32, (py - 3).max(0) as u32, [255, 255, 255, 255]);
                        target_buf.set_pixel((px - 1).max(0) as u32, (py - 2).max(0) as u32, [255, 255, 255, 255]);
                        target_buf.set_pixel((px + 1) as u32, (py - 2).max(0) as u32, [255, 255, 255, 255]);
                    }
                    ElevationIndicator::Below => {
                        // Tiny 'v' arrow below dot
                        target_buf.set_pixel(px as u32, (py + 3) as u32, [20, 20, 20, 255]);
                        target_buf.set_pixel((px - 1).max(0) as u32, (py + 2) as u32, [20, 20, 20, 255]);
                        target_buf.set_pixel((px + 1) as u32, (py + 2) as u32, [20, 20, 20, 255]);
                    }
                    ElevationIndicator::Level => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::radar::entity::{EntityCategory, RadarEntity};

    #[test]
    fn test_radar_rendering() {
        let renderer = RadarRenderer::new();
        let mut buf = ImageBuffer::new(64, 64);

        let contact = RadarContact {
            entity: RadarEntity::new(1, "Player", EntityCategory::Player, 10.0, 68.0, 0.0, 0.0),
            dx: 10.0,
            dz: 0.0,
            distance: 10.0,
            screen_dx: 10.0,
            screen_dy: 0.0,
            elevation: ElevationIndicator::Above,
            color: [80, 160, 255, 255],
        };

        renderer.render_radar_overlay(&mut buf, 32, 32, &[contact]);
        // Pixel at (42, 32) should be colored
        let p = buf.get_pixel(42, 32);
        assert_eq!(p[0], 80);
        assert_eq!(p[1], 160);
    }
}
