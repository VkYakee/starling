use super::tile::TileCoord;
use crate::render::image_buffer::ImageBuffer;
use std::collections::HashMap;

/// High-performance in-memory chunk tile cache with server and dimension namespaces (XaeroPlus & BlueMap architecture).
#[derive(Debug, Clone)]
pub struct ChunkTileCache {
    tiles: HashMap<(String, String, TileCoord), ImageBuffer>,
    max_capacity: usize,
}

impl Default for ChunkTileCache {
    fn default() -> Self {
        Self::new(1024)
    }
}

impl ChunkTileCache {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            tiles: HashMap::new(),
            max_capacity,
        }
    }

    /// Retrieves a cached tile buffer if present.
    pub fn get(&self, world_id: &str, dimension: &str, coord: TileCoord) -> Option<&ImageBuffer> {
        let key = (world_id.to_string(), dimension.to_string(), coord);
        self.tiles.get(&key)
    }

    /// Stores a rendered tile buffer in cache.
    pub fn put(&mut self, world_id: &str, dimension: &str, coord: TileCoord, buffer: ImageBuffer) {
        if self.tiles.len() >= self.max_capacity {
            // Simple eviction of arbitrary oldest item
            if let Some(first_key) = self.tiles.keys().next().cloned() {
                self.tiles.remove(&first_key);
            }
        }
        let key = (world_id.to_string(), dimension.to_string(), coord);
        self.tiles.insert(key, buffer);
    }

    pub fn clear_dimension(&mut self, world_id: &str, dimension: &str) {
        self.tiles.retain(|(w, d, _), _| w != world_id || d != dimension);
    }

    pub fn clear_all(&mut self) {
        self.tiles.clear();
    }

    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_tile_cache() {
        let mut cache = ChunkTileCache::new(2);
        let tile = TileCoord::new(0, 5, -2);
        let buf = ImageBuffer::new_filled(16, 16, [100, 150, 200, 255]);

        cache.put("singleplayer", "overworld", tile, buf.clone());
        assert_eq!(cache.len(), 1);

        let retrieved = cache.get("singleplayer", "overworld", tile);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().get_pixel(0, 0), [100, 150, 200, 255]);

        // Dimension mismatch
        assert!(cache.get("singleplayer", "the_nether", tile).is_none());
    }
}
