use crate::annotations::manager::AnnotationManager;
use crate::render::image_buffer::ImageBuffer;
use crate::structure_bridge::StructureBridge;
use crate::waypoints::manager::WaypointManager;
use crate::waypoints::waypoint::Dimension;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Comprehensive world export dataset for external tools, seed finders, and web maps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapExportDataset {
    pub world_id: String,
    pub dimension: String,
    pub exported_at_utc: u64,
    pub total_structures: usize,
    pub structures: serde_json::Value,
    pub total_waypoints: usize,
    pub waypoints: serde_json::Value,
    pub total_annotations: usize,
    pub annotations: serde_json::Value,
}

/// Map Exporter generating high-res PNG stitchings and structured JSON datasets.
#[derive(Debug, Clone)]
pub struct MapExporter;

impl MapExporter {
    /// Stitches a collection of 16x16 chunk tiles into a composite high-resolution PNG map image.
    ///
    /// `chunks`: vector of `(chunk_x, chunk_z, &ImageBuffer)`
    pub fn stitch_chunks_to_image(
        chunks: &[(i32, i32, ImageBuffer)],
        min_cx: i32,
        max_cx: i32,
        min_cz: i32,
        max_cz: i32,
    ) -> Result<ImageBuffer, String> {
        let width_chunks = (max_cx - min_cx + 1).max(1) as u32;
        let height_chunks = (max_cz - min_cz + 1).max(1) as u32;

        let total_w = width_chunks * 16;
        let total_h = height_chunks * 16;

        let mut composite = ImageBuffer::new(total_w, total_h);

        for (cx, cz, buf) in chunks {
            let offset_x = ((cx - min_cx) * 16) as i32;
            let offset_y = ((cz - min_cz) * 16) as i32;
            composite.blit(buf, offset_x, offset_y);
        }

        Ok(composite)
    }

    /// Exports the composite map image to a PNG file.
    pub fn export_png<P: AsRef<Path>>(
        output_dir: P,
        world_id: &str,
        dimension: &Dimension,
        image: &ImageBuffer,
    ) -> std::io::Result<PathBuf> {
        let dir = output_dir.as_ref();
        fs::create_dir_all(dir)?;

        let filename = format!(
            "{}_{}_map_export.png",
            sanitize_name(world_id),
            dimension.as_str()
        );
        let out_path = dir.join(filename);
        image
            .save_png(&out_path)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        Ok(out_path)
    }

    /// Exports structured JSON data containing all discovered structures, waypoints, and claims.
    pub fn export_json<P: AsRef<Path>>(
        output_dir: P,
        world_id: &str,
        dimension: &Dimension,
        structures: &StructureBridge,
        waypoints: &WaypointManager,
        annotations: &AnnotationManager,
    ) -> std::io::Result<PathBuf> {
        let dir = output_dir.as_ref();
        fs::create_dir_all(dir)?;

        let dim_structures: Vec<_> = structures.get_in_dimension(dimension);
        let dim_waypoints: Vec<_> = waypoints.get_active_in_dimension(dimension);

        let dataset = MapExportDataset {
            world_id: world_id.to_string(),
            dimension: dimension.as_str().to_string(),
            exported_at_utc: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            total_structures: dim_structures.len(),
            structures: serde_json::to_value(&dim_structures).unwrap_or_default(),
            total_waypoints: dim_waypoints.len(),
            waypoints: serde_json::to_value(&dim_waypoints).unwrap_or_default(),
            total_annotations: annotations.len(),
            annotations: serde_json::to_value(annotations.list()).unwrap_or_default(),
        };

        let filename = format!(
            "{}_{}_map_data.json",
            sanitize_name(world_id),
            dimension.as_str()
        );
        let out_path = dir.join(filename);
        let json_str = serde_json::to_string_pretty(&dataset)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(&out_path, json_str)?;

        Ok(out_path)
    }
}

fn sanitize_name(raw: &str) -> String {
    raw.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_export_stitching() {
        let chunk1 = ImageBuffer::new_filled(16, 16, [255, 0, 0, 255]);
        let chunk2 = ImageBuffer::new_filled(16, 16, [0, 255, 0, 255]);

        let chunks = vec![(0, 0, chunk1), (1, 0, chunk2)];
        let stitched = MapExporter::stitch_chunks_to_image(&chunks, 0, 1, 0, 0).unwrap();

        assert_eq!(stitched.width, 32);
        assert_eq!(stitched.height, 16);
        assert_eq!(stitched.get_pixel(0, 0), [255, 0, 0, 255]);
        assert_eq!(stitched.get_pixel(16, 0), [0, 255, 0, 255]);
    }
}
