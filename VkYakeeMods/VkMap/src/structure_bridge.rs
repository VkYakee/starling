use crate::render::image_buffer::ImageBuffer;
use crate::waypoints::waypoint::Dimension;
use serde::{Deserialize, Serialize};

/// Origin source of a structure marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructureSource {
    /// Discovered and verified directly from real blocks loaded in the world.
    WorldScan,
    /// Mathematically predicted by SeedCracker's seed reversal algorithms.
    SeedPrediction,
}

impl StructureSource {
    pub fn badge(&self) -> &'static str {
        match self {
            StructureSource::WorldScan => "[VERIF]",
            StructureSource::SeedPrediction => "[PRED]",
        }
    }
}

/// A structure marker on the map shared between VkMap and SeedCracker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructureInfo {
    pub id: String,
    pub structure_type: String, // e.g. "minecraft:trial_chambers", "minecraft:ancient_city"
    pub display_name: String,
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub block_x: i32,
    pub block_y: i32,
    pub block_z: i32,
    pub dimension: Dimension,
    pub source: StructureSource,
    pub discovered_at: u64,
    pub color: [u8; 4],
}

impl StructureInfo {
    pub fn new(
        structure_type: impl Into<String>,
        chunk_x: i32,
        chunk_z: i32,
        block_y: i32,
        dimension: Dimension,
        source: StructureSource,
    ) -> Self {
        let st = structure_type.into();
        let display = format_structure_display_name(&st);
        let block_x = chunk_x * 16 + 8;
        let block_z = chunk_z * 16 + 8;
        let color = get_structure_color(&st, source);
        let id = format!("{}_{}_{}", st, chunk_x, chunk_z);

        Self {
            id,
            structure_type: st,
            display_name: display,
            chunk_x,
            chunk_z,
            block_x,
            block_y,
            block_z,
            dimension,
            source,
            discovered_at: 0,
            color,
        }
    }
}

/// Bidirectional bridge coordinating structures between VkMap and SeedCracker.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructureBridge {
    pub structures: Vec<StructureInfo>,
}

impl StructureBridge {
    pub fn new() -> Self {
        Self {
            structures: Vec::new(),
        }
    }

    /// Registers a structure detected by world chunk scanning.
    /// If it was already known as a SeedPrediction, promotes it to WorldScan (Verified).
    pub fn register_world_structure(
        &mut self,
        structure_type: &str,
        chunk_x: i32,
        chunk_z: i32,
        block_y: i32,
        dimension: Dimension,
    ) -> &StructureInfo {
        let id = format!("{}_{}_{}", structure_type, chunk_x, chunk_z);
        if let Some(pos) = self.structures.iter().position(|s| s.id == id) {
            self.structures[pos].source = StructureSource::WorldScan;
            self.structures[pos].color = get_structure_color(structure_type, StructureSource::WorldScan);
            &self.structures[pos]
        } else {
            let st = StructureInfo::new(
                structure_type,
                chunk_x,
                chunk_z,
                block_y,
                dimension,
                StructureSource::WorldScan,
            );
            self.structures.push(st);
            self.structures.last().unwrap()
        }
    }

    /// Registers a structure predicted mathematically by SeedCracker.
    pub fn register_predicted_structure(
        &mut self,
        structure_type: &str,
        chunk_x: i32,
        chunk_z: i32,
        block_y: i32,
        dimension: Dimension,
    ) {
        let id = format!("{}_{}_{}", structure_type, chunk_x, chunk_z);
        if !self.structures.iter().any(|s| s.id == id) {
            let st = StructureInfo::new(
                structure_type,
                chunk_x,
                chunk_z,
                block_y,
                dimension,
                StructureSource::SeedPrediction,
            );
            self.structures.push(st);
        }
    }

    /// Retrieves all structures for a specific dimension.
    pub fn get_in_dimension(&self, dim: &Dimension) -> Vec<&StructureInfo> {
        self.structures
            .iter()
            .filter(|s| &s.dimension == dim)
            .collect()
    }

    /// Rasterizes structure marker badges onto an ImageBuffer.
    pub fn rasterize_to_buffer(
        &self,
        buf: &mut ImageBuffer,
        center_world_x: f64,
        center_world_z: f64,
        zoom: f32,
        dim: &Dimension,
    ) {
        let half_w = buf.width as f32 / 2.0;
        let half_h = buf.height as f32 / 2.0;

        for st in self.get_in_dimension(dim) {
            let sx = half_w + ((st.block_x as f64 - center_world_x) as f32 * zoom);
            let sy = half_h + ((st.block_z as f64 - center_world_z) as f32 * zoom);

            let px = sx.round() as i32;
            let py = sy.round() as i32;

            if px >= 3 && px < buf.width as i32 - 3 && py >= 3 && py < buf.height as i32 - 3 {
                // Diamond shape marker
                buf.draw_rect(px - 2, py - 2, px + 2, py + 2, st.color, true);
                let border = match st.source {
                    StructureSource::WorldScan => [255, 255, 255, 255],     // Crisp white border
                    StructureSource::SeedPrediction => [255, 215, 0, 255], // Glowing gold border
                };
                buf.draw_rect(px - 2, py - 2, px + 2, py + 2, border, false);
            }
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.structures)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let structures = serde_json::from_str(json)?;
        Ok(Self { structures })
    }

    pub fn len(&self) -> usize {
        self.structures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.structures.is_empty()
    }
}

fn format_structure_display_name(raw: &str) -> String {
    let clean = raw.strip_prefix("minecraft:").unwrap_or(raw);
    match clean {
        "trial_chambers" => "Trial Chambers (Cámaras del Juicio)".to_string(),
        "ancient_city" => "Ancient City (Ciudad Ancestral)".to_string(),
        "buried_treasure" => "Tesoro Enterrado".to_string(),
        "desert_pyramid" => "Pirámide del Desierto".to_string(),
        "swamp_hut" => "Cabaña de Bruja".to_string(),
        "fortress" => "Fortaleza del Nether".to_string(),
        "end_city" => "Ciudad del End".to_string(),
        "monument" => "Monumento Oceánico".to_string(),
        "mansion" => "Mansión del Bosque".to_string(),
        "pillager_outpost" => "Puesto de Saqueadores".to_string(),
        "village" => "Aldea".to_string(),
        _ => clean.replace('_', " "),
    }
}

fn get_structure_color(raw: &str, source: StructureSource) -> [u8; 4] {
    let clean = raw.strip_prefix("minecraft:").unwrap_or(raw);
    match clean {
        "trial_chambers" => [240, 130, 40, 255],  // Oxidized copper/orange
        "ancient_city" => [20, 180, 200, 255],    // Sculk cyan
        "buried_treasure" => [255, 215, 0, 255],  // Gold
        "desert_pyramid" => [230, 180, 70, 255],  // Sandstone
        "swamp_hut" => [120, 70, 150, 255],       // Witch purple
        "fortress" => [180, 40, 40, 255],         // Nether red
        "end_city" => [200, 120, 220, 255],       // Purpur magenta
        "monument" => [40, 200, 160, 255],        // Prismarine
        "mansion" => [90, 60, 40, 255],           // Dark oak
        "pillager_outpost" => [150, 100, 70, 255],
        "village" => [100, 210, 80, 255],         // Green
        _ => match source {
            StructureSource::WorldScan => [80, 220, 80, 255],
            StructureSource::SeedPrediction => [255, 200, 50, 255],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structure_bridge_lifecycle() {
        let mut bridge = StructureBridge::new();

        // 1. Seed prediction
        bridge.register_predicted_structure(
            "minecraft:trial_chambers",
            10,
            20,
            -20,
            Dimension::Overworld,
        );
        assert_eq!(bridge.len(), 1);
        assert_eq!(bridge.structures[0].source, StructureSource::SeedPrediction);
        assert_eq!(bridge.structures[0].source.badge(), "[PRED]");

        // 2. Verified by visiting in world
        bridge.register_world_structure(
            "minecraft:trial_chambers",
            10,
            20,
            -20,
            Dimension::Overworld,
        );
        assert_eq!(bridge.len(), 1); // Upgraded in-place!
        assert_eq!(bridge.structures[0].source, StructureSource::WorldScan);
        assert_eq!(bridge.structures[0].source.badge(), "[VERIF]");
    }
}
