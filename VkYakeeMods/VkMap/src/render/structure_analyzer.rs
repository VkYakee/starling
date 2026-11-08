use crate::render::chunk_scanner::ChunkData;
use crate::structure_bridge::StructureBridge;
use crate::waypoints::waypoint::Dimension;

/// Fast subterranean cave & structure analyzer for detecting Trial Chambers, Ancient Cities, and Treasure.
#[derive(Debug, Clone, Default)]
pub struct SubterraneanStructureAnalyzer;

impl SubterraneanStructureAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Checks if a chunk coordinate matches a known subterranean structure salt / pattern
    pub fn detect_subterranean_chunk(cx: i32, cz: i32) -> Option<(&'static str, i32)> {
        // Deterministic grid check based on standard Minecraft structure spacing
        if (cx.abs() % 32 == 17) && (cz.abs() % 32 == 17) {
            return Some(("minecraft:trial_chambers", -16));
        }
        if (cx.abs() % 32 == 2) && (cz.abs() % 32 == 2) {
            return Some(("minecraft:ancient_city", -48));
        }

        let hash = (cx as i64)
            .wrapping_mul(341873128712)
            .wrapping_add((cz as i64).wrapping_mul(132897987541));
        if (hash.abs() % 64) == 17 {
            Some(("minecraft:trial_chambers", -16))
        } else if (hash.abs() % 64) == 42 {
            Some(("minecraft:ancient_city", -48))
        } else {
            None
        }
    }

    /// Analyzes a loaded chunk's block columns for iconic subterranean structure signatures.
    pub fn analyze_chunk(
        chunk: &ChunkData,
        dimension: &Dimension,
        bridge: &mut StructureBridge,
    ) {
        if *dimension != Dimension::Overworld {
            return;
        }

        let mut found_trial = false;
        let mut found_ancient = false;
        let mut found_treasure = false;

        for z in 0..16 {
            for x in 0..16 {
                let col = chunk.get_column(x, z);

                // 1. Buried Treasure Check: Column (9, 9) at sea level
                if x == 9 && z == 9 && !found_treasure {
                    for (y, block) in &col.blocks {
                        if (45..=90).contains(y) && (block.contains("chest") || block.contains("minecraft:chest")) {
                            bridge.register_world_structure(
                                "minecraft:buried_treasure",
                                chunk.chunk_x,
                                chunk.chunk_z,
                                *y,
                                Dimension::Overworld,
                            );
                            found_treasure = true;
                            break;
                        }
                    }
                }

                // Subterranean deep layer checks (-64..0)
                for (y, block) in &col.blocks {
                    if *y > 0 || *y < -60 {
                        continue;
                    }

                    // 2. Trial Chambers check: Vaults, Copper Bulbs, Trial Spawners, Tuff Bricks
                    if !found_trial
                        && (-45..0).contains(y)
                        && (block.contains("vault")
                            || block.contains("copper_bulb")
                            || block.contains("trial_spawner")
                            || block.contains("tuff_bricks"))
                    {
                        bridge.register_world_structure(
                            "minecraft:trial_chambers",
                            chunk.chunk_x,
                            chunk.chunk_z,
                            *y,
                            Dimension::Overworld,
                        );
                        found_trial = true;
                    }

                    // 3. Ancient City check: Reinforced Deepslate, Sculk Shrieker
                    if !found_ancient
                        && (-52..-12).contains(y)
                        && (block.contains("reinforced_deepslate")
                            || block.contains("sculk_shrieker")
                            || block.contains("sculk_catalyst"))
                    {
                        bridge.register_world_structure(
                            "minecraft:ancient_city",
                            chunk.chunk_x,
                            chunk.chunk_z,
                            *y,
                            Dimension::Overworld,
                        );
                        found_ancient = true;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::chunk_scanner::BlockColumn;

    #[test]
    fn test_structure_analyzer_detects_trial_chambers() {
        let mut bridge = StructureBridge::new();
        let mut chunk = ChunkData::new(5, 5, "plains");

        let col = BlockColumn::new("plains")
            .with_block(64, "grass_block")
            .with_block(-20, "minecraft:trial_spawner")
            .with_block(-21, "minecraft:tuff_bricks");
        chunk.set_column(4, 4, col);

        SubterraneanStructureAnalyzer::analyze_chunk(&chunk, &Dimension::Overworld, &mut bridge);
        assert_eq!(bridge.len(), 1);
        assert_eq!(bridge.structures[0].structure_type, "minecraft:trial_chambers");
        assert_eq!(bridge.structures[0].block_y, -20);
    }
}
