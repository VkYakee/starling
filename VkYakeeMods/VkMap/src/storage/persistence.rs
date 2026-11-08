use super::tile::TileCoord;
use crate::annotations::manager::AnnotationManager;
use crate::render::image_buffer::ImageBuffer;
use crate::waypoints::manager::WaypointManager;
use std::fs;
use std::path::{Path, PathBuf};

/// Multi-World and Multi-Server disk persistence system (XaeroPlus & BlueMap layout).
#[derive(Debug, Clone)]
pub struct MapStorage {
    pub base_dir: PathBuf,
}

impl MapStorage {
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    /// Generates file path for a PNG tile: `{base_dir}/tiles/{world}/{dimension}/z{zoom}/{x}_{z}.png`
    pub fn tile_path(&self, world: &str, dimension: &str, coord: TileCoord) -> PathBuf {
        self.base_dir
            .join("tiles")
            .join(sanitize_path(world))
            .join(sanitize_path(dimension))
            .join(format!("z{}", coord.zoom))
            .join(format!("{}_{}.png", coord.x, coord.z))
    }

    /// Saves a tile `ImageBuffer` to disk as a PNG file.
    pub fn save_tile(
        &self,
        world: &str,
        dimension: &str,
        coord: TileCoord,
        buf: &ImageBuffer,
    ) -> std::io::Result<()> {
        let path = self.tile_path(world, dimension, coord);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        buf.save_png(&path)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    /// Loads a tile `ImageBuffer` from disk if present.
    pub fn load_tile(&self, world: &str, dimension: &str, coord: TileCoord) -> Option<ImageBuffer> {
        let path = self.tile_path(world, dimension, coord);
        if path.exists() {
            let bytes = fs::read(&path).ok()?;
            ImageBuffer::from_png_bytes(&bytes).ok()
        } else {
            None
        }
    }

    /// Saves waypoints to JSON file: `{base_dir}/waypoints/{world}.json`
    pub fn save_waypoints(&self, world: &str, manager: &WaypointManager) -> std::io::Result<()> {
        let path = self.base_dir.join("waypoints").join(format!("{}.json", sanitize_path(world)));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = manager
            .to_json()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(path, json)
    }

    /// Loads waypoints from JSON file if present.
    pub fn load_waypoints(&self, world: &str) -> Option<WaypointManager> {
        let path = self.base_dir.join("waypoints").join(format!("{}.json", sanitize_path(world)));
        if path.exists() {
            let content = fs::read_to_string(path).ok()?;
            WaypointManager::from_json(&content).ok()
        } else {
            None
        }
    }

    /// Saves vector annotations to JSON file: `{base_dir}/annotations/{world}.json`
    pub fn save_annotations(&self, world: &str, manager: &AnnotationManager) -> std::io::Result<()> {
        let path = self.base_dir.join("annotations").join(format!("{}.json", sanitize_path(world)));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = manager
            .to_json()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(path, json)
    }

    /// Loads vector annotations from JSON file if present.
    pub fn load_annotations(&self, world: &str) -> Option<AnnotationManager> {
        let path = self.base_dir.join("annotations").join(format!("{}.json", sanitize_path(world)));
        if path.exists() {
            let content = fs::read_to_string(path).ok()?;
            AnnotationManager::from_json(&content).ok()
        } else {
            None
        }
    }
}

fn sanitize_path(name: &str) -> String {
    name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_storage_tile_paths() {
        let storage = MapStorage::new("/tmp/vkmap_test");
        let coord = TileCoord::new(1, 10, -5);
        let path = storage.tile_path("myserver.net:25565", "minecraft:overworld", coord);

        assert!(path.to_string_lossy().contains("myserver.net_25565"));
        assert!(path.to_string_lossy().contains("z1"));
        assert!(path.to_string_lossy().ends_with("10_-5.png"));
    }
}
