use super::touch_widgets::MinimapDisplaySize;
use crate::radar::tracker::RadarContact;
use crate::render::image_buffer::ImageBuffer;
use crate::structure_bridge::{StructureInfo, StructureSource};
use crate::waypoints::waypoint::Waypoint;
use serde::{Deserialize, Serialize};

/// Minimap frame geometry (VoxelMap & Diorama styles).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MinimapShape {
    Round,
    Square,
}

impl Default for MinimapShape {
    fn default() -> Self {
        MinimapShape::Round
    }
}

/// Minimap HUD configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimapConfig {
    pub shape: MinimapShape,
    pub display_size: MinimapDisplaySize,
    pub zoom: f32,             // Scale factor (pixels per block, e.g. 1.0)
    pub rotating: bool,        // True = rotates with yaw, False = North locked
    pub show_radar: bool,
    pub show_waypoints: bool,
    pub show_structures: bool,
    pub show_cardinals: bool,
}

impl Default for MinimapConfig {
    fn default() -> Self {
        Self {
            shape: MinimapShape::Round,
            display_size: MinimapDisplaySize::Medium,
            zoom: 1.0,
            rotating: false,
            show_radar: true,
            show_waypoints: true,
            show_structures: true,
            show_cardinals: true,
        }
    }
}

impl MinimapConfig {
    pub fn current_pixel_size(&self) -> u32 {
        self.display_size.pixel_size()
    }
}

/// Minimap HUD Compositor assembling terrain, radar, waypoints, structures, and frame.
#[derive(Debug, Clone)]
pub struct MinimapCompositor {
    pub config: MinimapConfig,
}

impl Default for MinimapCompositor {
    fn default() -> Self {
        Self::new(MinimapConfig::default())
    }
}

impl MinimapCompositor {
    pub fn new(config: MinimapConfig) -> Self {
        Self { config }
    }

    /// Composites all minimap layers into a ready-to-render RGBA image buffer.
    pub fn composite_minimap(
        &self,
        terrain: &ImageBuffer,
        player_x: f64,
        player_z: f64,
        player_yaw: f32,
        radar_contacts: &[RadarContact],
        waypoints: &[&Waypoint],
        structures: &[&StructureInfo],
    ) -> ImageBuffer {
        let size = self.config.current_pixel_size();
        let mut final_buf = ImageBuffer::new(size, size);
        let center = (size / 2) as i32;
        let radius = center - 4;

        // 1. Copy terrain layer onto target
        final_buf.blit(terrain, 0, 0);

        // 2. Discovered & Predicted Structures layer
        if self.config.show_structures {
            for st in structures {
                let st_dx = (st.block_x as f64 - player_x) as f32 * self.config.zoom;
                let st_dz = (st.block_z as f64 - player_z) as f32 * self.config.zoom;
                let sx = center + st_dx.round() as i32;
                let sy = center + st_dz.round() as i32;

                if self.is_inside_frame(sx - center, sy - center, radius) {
                    final_buf.draw_rect(sx - 3, sy - 3, sx + 3, sy + 3, st.color, true);
                    let border_col = match st.source {
                        StructureSource::WorldScan => [255, 255, 255, 255],     // White border
                        StructureSource::SeedPrediction => [255, 215, 0, 255], // Gold border
                    };
                    final_buf.draw_rect(sx - 3, sy - 3, sx + 3, sy + 3, border_col, false);
                }
            }
        }

        // 3. Waypoints layer
        if self.config.show_waypoints {
            for wp in waypoints {
                let wp_dx = (wp.x as f64 - player_x) as f32 * self.config.zoom;
                let wp_dz = (wp.z as f64 - player_z) as f32 * self.config.zoom;
                let sx = center + wp_dx.round() as i32;
                let sy = center + wp_dz.round() as i32;

                if self.is_inside_frame(sx - center, sy - center, radius) {
                    let color = [wp.color[0], wp.color[1], wp.color[2], 255];
                    final_buf.draw_rect(sx - 2, sy - 2, sx + 2, sy + 2, color, true);
                    final_buf.draw_rect(sx - 2, sy - 2, sx + 2, sy + 2, [255, 255, 255, 255], false);
                }
            }
        }

        // 4. Radar layer
        if self.config.show_radar {
            for contact in radar_contacts {
                let sx = center + contact.screen_dx.round() as i32;
                let sy = center + contact.screen_dy.round() as i32;

                if self.is_inside_frame(sx - center, sy - center, radius) {
                    final_buf.draw_circle(sx, sy, 2, contact.color, true);
                }
            }
        }

        // 5. Center player indicator (classic navigation arrow or red dot)
        self.draw_player_arrow(&mut final_buf, center, center, player_yaw);

        // 6. Apply stencil mask (masking out outside corners)
        self.apply_mask(&mut final_buf, center, radius);

        // 7. Draw outer frame border
        self.draw_frame_border(&mut final_buf, center, radius);

        // 8. Cardinal compass points (N, E, S, W)
        if self.config.show_cardinals {
            self.draw_cardinals(&mut final_buf, center, radius, player_yaw);
        }

        final_buf
    }

    fn is_inside_frame(&self, dx: i32, dy: i32, radius: i32) -> bool {
        match self.config.shape {
            MinimapShape::Round => (dx * dx + dy * dy) <= (radius * radius),
            MinimapShape::Square => dx.abs() <= radius && dy.abs() <= radius,
        }
    }

    fn apply_mask(&self, buf: &mut ImageBuffer, center: i32, radius: i32) {
        if self.config.shape == MinimapShape::Round {
            let r_sq = radius * radius;
            for y in 0..buf.height as i32 {
                let dy = y - center;
                for x in 0..buf.width as i32 {
                    let dx = x - center;
                    if (dx * dx + dy * dy) > r_sq {
                        buf.set_pixel(x as u32, y as u32, [0, 0, 0, 0]);
                    }
                }
            }
        }
    }

    fn draw_frame_border(&self, buf: &mut ImageBuffer, center: i32, radius: i32) {
        let border_color = [40, 40, 45, 255];
        let rim_highlight = [180, 180, 190, 255];

        match self.config.shape {
            MinimapShape::Round => {
                buf.draw_circle(center, center, radius, rim_highlight, false);
                buf.draw_circle(center, center, radius + 1, border_color, false);
                buf.draw_circle(center, center, radius + 2, border_color, false);
            }
            MinimapShape::Square => {
                buf.draw_rect(center - radius, center - radius, center + radius, center + radius, rim_highlight, false);
                buf.draw_rect(center - radius - 1, center - radius - 1, center + radius + 1, center + radius + 1, border_color, false);
            }
        }
    }

    fn draw_player_arrow(&self, buf: &mut ImageBuffer, cx: i32, cy: i32, yaw: f32) {
        // Player center dot with direction notch
        buf.draw_circle(cx, cy, 3, [255, 255, 255, 255], true);
        buf.draw_circle(cx, cy, 2, [255, 40, 40, 255], true);

        // Direction pointer
        let rad = if self.config.rotating {
            0.0
        } else {
            (yaw + 180.0).to_radians()
        };
        let tip_x = cx + ((rad.sin() * 7.0).round() as i32);
        let tip_y = cy - ((rad.cos() * 7.0).round() as i32);
        buf.draw_line(cx, cy, tip_x, tip_y, [255, 255, 255, 255], 1);
    }

    fn draw_cardinals(&self, buf: &mut ImageBuffer, cx: i32, cy: i32, yaw: f32) {
        let offset = cy - 2;
        let north_color = [255, 80, 80, 255]; // Red 'N'

        if !self.config.rotating {
            // Static North at the top
            buf.set_pixel(cx as u32, (cy - offset).max(0) as u32, north_color);
            buf.set_pixel((cx - 1).max(0) as u32, (cy - offset + 1).max(0) as u32, north_color);
            buf.set_pixel((cx + 1) as u32, (cy - offset + 1).max(0) as u32, north_color);
        } else {
            // Rotating North indicator
            let n_rad = (-yaw).to_radians();
            let nx = cx + ((n_rad.sin() * offset as f32).round() as i32);
            let ny = cy - ((n_rad.cos() * offset as f32).round() as i32);
            if nx >= 0 && nx < buf.width as i32 && ny >= 0 && ny < buf.height as i32 {
                buf.draw_circle(nx, ny, 2, north_color, true);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimap_display_size_compositing() {
        let mut config = MinimapConfig::default();
        config.display_size = MinimapDisplaySize::Small;
        let compositor = MinimapCompositor::new(config);
        let terrain = ImageBuffer::new_filled(96, 96, [50, 150, 50, 255]);

        let output = compositor.composite_minimap(&terrain, 0.0, 0.0, 0.0, &[], &[], &[]);
        assert_eq!(output.width, 96);
        assert_eq!(output.height, 96);
    }
}
