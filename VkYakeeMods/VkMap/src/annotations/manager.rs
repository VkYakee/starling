use super::shape::{Annotation, AnnotationShape};
use crate::render::image_buffer::ImageBuffer;
use serde::{Deserialize, Serialize};

/// High-level Annotation and Claim Overlay Manager (Conflux-Map vector engine).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnnotationManager {
    annotations: Vec<Annotation>,
}

impl AnnotationManager {
    pub fn new() -> Self {
        Self {
            annotations: Vec::new(),
        }
    }

    pub fn add(&mut self, ann: Annotation) {
        if let Some(pos) = self.annotations.iter().position(|a| a.id == ann.id) {
            self.annotations[pos] = ann;
        } else {
            self.annotations.push(ann);
        }
    }

    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(pos) = self.annotations.iter().position(|a| a.id == id) {
            self.annotations.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn toggle(&mut self, id: &str) -> bool {
        if let Some(ann) = self.annotations.iter_mut().find(|a| a.id == id) {
            ann.visible = !ann.visible;
            true
        } else {
            false
        }
    }

    pub fn list(&self) -> &[Annotation] {
        &self.annotations
    }

    /// Rasterizes all visible annotations onto an `ImageBuffer`.
    ///
    /// `center_world_x`, `center_world_z`: camera position
    /// `zoom`: scale factor (pixels per block)
    pub fn rasterize_to_buffer(
        &self,
        buf: &mut ImageBuffer,
        center_world_x: f64,
        center_world_z: f64,
        zoom: f32,
    ) {
        let half_w = buf.width as f32 / 2.0;
        let half_h = buf.height as f32 / 2.0;

        let to_screen = |wx: i32, wz: i32| -> (i32, i32) {
            let sx = half_w + ((wx as f64 - center_world_x) as f32 * zoom);
            let sy = half_h + ((wz as f64 - center_world_z) as f32 * zoom);
            (sx.round() as i32, sy.round() as i32)
        };

        for ann in &self.annotations {
            if !ann.visible {
                continue;
            }

            match &ann.shape {
                AnnotationShape::Line { start, end, thickness } => {
                    let (sx0, sy0) = to_screen(start.0, start.1);
                    let (sx1, sy1) = to_screen(end.0, end.1);
                    let scaled_t = ((*thickness as f32 * zoom).round() as u32).max(1);
                    buf.draw_line(sx0, sy0, sx1, sy1, ann.color, scaled_t);
                }
                AnnotationShape::Circle { center, radius, filled } => {
                    let (cx, cy) = to_screen(center.0, center.1);
                    let r = (*radius as f32 * zoom).round() as i32;
                    if r > 0 {
                        buf.draw_circle(cx, cy, r, ann.color, *filled);
                    }
                }
                AnnotationShape::Rectangle { min_x, min_z, max_x, max_z, filled } => {
                    let (x0, y0) = to_screen(*min_x, *min_z);
                    let (x1, y1) = to_screen(*max_x, *max_z);
                    buf.draw_rect(x0, y0, x1, y1, ann.color, *filled);
                }
                AnnotationShape::Freehand { points, thickness } => {
                    let scaled_t = ((*thickness as f32 * zoom).round() as u32).max(1);
                    for i in 0..points.len().saturating_sub(1) {
                        let (sx0, sy0) = to_screen(points[i].0, points[i].1);
                        let (sx1, sy1) = to_screen(points[i + 1].0, points[i + 1].1);
                        buf.draw_line(sx0, sy0, sx1, sy1, ann.color, scaled_t);
                    }
                }
                AnnotationShape::Label { pos, .. } => {
                    let (lx, ly) = to_screen(pos.0, pos.1);
                    buf.draw_circle(lx, ly, 3, ann.color, true);
                }
            }
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.annotations)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let annotations = serde_json::from_str(json)?;
        Ok(Self { annotations })
    }

    pub fn len(&self) -> usize {
        self.annotations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.annotations.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotation_lifecycle_and_rasterization() {
        let mut mgr = AnnotationManager::new();
        let claim = Annotation::chunk_claim("claim_1", "VkYakee", 0, 0, [0, 255, 255, 200]);
        let highway = Annotation::line("nether_hw", (0, 0), (100, 100), [255, 128, 0, 255], 2);

        mgr.add(claim);
        mgr.add(highway);
        assert_eq!(mgr.len(), 2);

        let mut buf = ImageBuffer::new(128, 128);
        mgr.rasterize_to_buffer(&mut buf, 50.0, 50.0, 1.0);

        // Center of the buffer (64, 64) is world (50, 50), which lies directly on the highway!
        let p = buf.get_pixel(64, 64);
        assert_eq!(p[0], 255);
        assert_eq!(p[1], 128);
    }
}
