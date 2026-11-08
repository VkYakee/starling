// Copyright (c) 2026 VkYakee
// VkYakee VoxelMap x seedcrackerX
//! In-game structure and decorator finder scanning loaded chunk blocks.
//! Replicates SeedcrackerX structure identification heuristics in Rust.

use super::cracker::{CollectedFeature, CrackerEngine, StructureType};
use super::waypoints::{Waypoint, WaypointManager};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub struct StructureFinder {
    visited_chunks: Mutex<HashSet<(i32, i32)>>,
}

impl StructureFinder {
    pub fn new() -> Self {
        Self {
            visited_chunks: Mutex::new(HashSet::new()),
        }
    }

    pub fn clear(&self) {
        self.visited_chunks.lock().unwrap().clear();
    }

    /// Check if chunk has already been analyzed
    pub fn mark_chunk_scanned(&self, cx: i32, cz: i32) -> bool {
        let mut visited = self.visited_chunks.lock().unwrap();
        visited.insert((cx, cz))
    }

    /// Inspect a loaded chunk's blocks using an arbitrary block inspector function
    pub fn scan_chunk(
        &self,
        cx: i32,
        cz: i32,
        dim: &str,
        biome_name: Option<&str>,
        get_block_name: impl Fn(i32, i32, i32) -> Option<String>,
        cracker: &Arc<CrackerEngine>,
        waypoints: &WaypointManager,
    ) {
        if !self.mark_chunk_scanned(cx, cz) {
            return;
        }

        let biome = biome_name.unwrap_or("").to_ascii_lowercase();

        // 1. Detect End Pillars in the End dimension
        if dim.contains("the_end") {
            let px = cx * 16 + 8;
            let pz = cz * 16 + 8;
            let dist_sq = px * px + pz * pz;
            // End pillars are located on a circle of radius ~42-45 blocks around (0,0)
            if (1500..=2500).contains(&dist_sq) {
                for y in (60..105).rev() {
                    if let Some(block) = get_block_name(px, y, pz) {
                        if block.contains("obsidian") {
                            let feat = CollectedFeature::new(StructureType::EndPillars, cx, cz, y);
                            if cracker.add_feature(feat) {
                                let wp = Waypoint::structure("Obsidian Pillar", [px, y, pz], dim);
                                waypoints.add(wp);
                            }
                            break;
                        }
                    }
                }
            }
        }

        // 2. Desert Pyramid in Desert biomes
        if biome.contains("desert") {
            let base_x = cx * 16;
            let base_z = cz * 16;
            let mut sandstone_count = 0;
            let mut terracotta_found = false;

            for dy in 60..75 {
                for dx in [4, 8, 12] {
                    for dz in [4, 8, 12] {
                        if let Some(name) = get_block_name(base_x + dx, dy, base_z + dz) {
                            if name.contains("sandstone") {
                                sandstone_count += 1;
                            } else if name.contains("terracotta") || name.contains("orange_terracotta") || name.contains("blue_terracotta") {
                                terracotta_found = true;
                            }
                        }
                    }
                }
            }

            if sandstone_count >= 6 && terracotta_found {
                let feat = CollectedFeature::new(StructureType::DesertPyramid, cx, cz, 64);
                if cracker.add_feature(feat) {
                    let wp = Waypoint::structure("Desert Pyramid", [base_x + 8, 64, base_z + 8], dim);
                    waypoints.add(wp);
                }
            }
        }

        // 3. Buried Treasure in Beach / Ocean biomes (Chest at chunk relative (9, y, 9))
        if biome.contains("beach") || biome.contains("ocean") {
            let tx = cx * 16 + 9;
            let tz = cz * 16 + 9;
            for y in 45..65 {
                if let Some(name) = get_block_name(tx, y, tz) {
                    if name.contains("chest") {
                        let feat = CollectedFeature::new(StructureType::BuriedTreasure, cx, cz, y);
                        if cracker.add_feature(feat) {
                            let wp = Waypoint::structure("Buried Treasure", [tx, y, tz], dim);
                            waypoints.add(wp);
                        }
                        break;
                    }
                }
            }
        }

        // 4. Jungle Temple in Jungle biomes
        if biome.contains("jungle") {
            let base_x = cx * 16;
            let base_z = cz * 16;
            let mut mossy_count = 0;
            for y in 60..75 {
                for dx in [4, 8, 12] {
                    for dz in [4, 8, 12] {
                        if let Some(name) = get_block_name(base_x + dx, y, base_z + dz) {
                            if name.contains("mossy_cobblestone") {
                                mossy_count += 1;
                            }
                        }
                    }
                }
            }
            if mossy_count >= 5 {
                let feat = CollectedFeature::new(StructureType::JungleTemple, cx, cz, 64);
                if cracker.add_feature(feat) {
                    let wp = Waypoint::structure("Jungle Temple", [base_x + 8, 64, base_z + 8], dim);
                    waypoints.add(wp);
                }
            }
        }

        // 5. Swamp Hut in Swamp biomes
        if biome.contains("swamp") {
            let base_x = cx * 16;
            let base_z = cz * 16;
            let mut spruce_planks = 0;
            for y in 62..70 {
                for dx in [2, 7, 12] {
                    for dz in [2, 7, 12] {
                        if let Some(name) = get_block_name(base_x + dx, y, base_z + dz) {
                            if name.contains("spruce_planks") || name.contains("oak_log") {
                                spruce_planks += 1;
                            }
                        }
                    }
                }
            }
            if spruce_planks >= 6 {
                let feat = CollectedFeature::new(StructureType::SwampHut, cx, cz, 64);
                if cracker.add_feature(feat) {
                    let wp = Waypoint::structure("Witch Swamp Hut", [base_x + 8, 64, base_z + 8], dim);
                    waypoints.add(wp);
                }
            }
        }

        // 6. Pillager Outpost in Plains / Savanna / Taiga / Desert / Meadow
        if biome.contains("plains") || biome.contains("savanna") || biome.contains("taiga") || biome.contains("meadow") || biome.contains("desert") {
            let base_x = cx * 16;
            let base_z = cz * 16;
            let mut dark_oak_count = 0;
            let mut cobble_count = 0;
            for y in 65..90 {
                for dx in [4, 8, 12] {
                    for dz in [4, 8, 12] {
                        if let Some(name) = get_block_name(base_x + dx, y, base_z + dz) {
                            if name.contains("dark_oak") {
                                dark_oak_count += 1;
                            } else if name.contains("cobblestone") {
                                cobble_count += 1;
                            }
                        }
                    }
                }
            }
            if dark_oak_count >= 5 && cobble_count >= 4 {
                let feat = CollectedFeature::new(StructureType::PillagerOutpost, cx, cz, 70);
                if cracker.add_feature(feat) {
                    let wp = Waypoint::structure("Pillager Outpost", [base_x + 8, 70, base_z + 8], dim);
                    waypoints.add(wp);
                }
            }
        }

        // 7. Igloo in Snowy biomes
        if biome.contains("snowy") || biome.contains("ice") {
            let base_x = cx * 16;
            let base_z = cz * 16;
            let mut snow_blocks = 0;
            let mut carpet_or_ice = false;
            for y in 60..80 {
                for dx in [4, 8, 12] {
                    for dz in [4, 8, 12] {
                        if let Some(name) = get_block_name(base_x + dx, y, base_z + dz) {
                            if name == "minecraft:snow_block" || name.contains("snow_block") {
                                snow_blocks += 1;
                            } else if name.contains("carpet") || name.contains("ice") {
                                carpet_or_ice = true;
                            }
                        }
                    }
                }
            }
            if snow_blocks >= 6 && carpet_or_ice {
                let feat = CollectedFeature::new(StructureType::Igloo, cx, cz, 64);
                if cracker.add_feature(feat) {
                    let wp = Waypoint::structure("Igloo", [base_x + 8, 64, base_z + 8], dim);
                    waypoints.add(wp);
                }
            }
        }

        // 8. Shipwreck in Ocean / Beach biomes
        if biome.contains("ocean") || biome.contains("beach") {
            let base_x = cx * 16;
            let base_z = cz * 16;
            let mut wood_blocks = 0;
            for y in 30..65 {
                for dx in [3, 8, 13] {
                    for dz in [3, 8, 13] {
                        if let Some(name) = get_block_name(base_x + dx, y, base_z + dz) {
                            if name.contains("planks") || name.contains("log") || name.contains("wood") || name.contains("trapdoor") {
                                wood_blocks += 1;
                            }
                        }
                    }
                }
            }
            if wood_blocks >= 6 {
                let feat = CollectedFeature::new(StructureType::Shipwreck, cx, cz, 45);
                if cracker.add_feature(feat) {
                    let wp = Waypoint::structure("Shipwreck", [base_x + 8, 45, base_z + 8], dim);
                    waypoints.add(wp);
                }
            }
        }

        // 9. Ocean Monument in Deep Ocean
        if biome.contains("deep_ocean") || biome.contains("monument") {
            let base_x = cx * 16;
            let base_z = cz * 16;
            let mut prismarine_count = 0;
            for y in 35..60 {
                for dx in [4, 8, 12] {
                    for dz in [4, 8, 12] {
                        if let Some(name) = get_block_name(base_x + dx, y, base_z + dz) {
                            if name.contains("prismarine") || name.contains("sea_lantern") {
                                prismarine_count += 1;
                            }
                        }
                    }
                }
            }
            if prismarine_count >= 8 {
                let feat = CollectedFeature::new(StructureType::OceanMonument, cx, cz, 40);
                if cracker.add_feature(feat) {
                    let wp = Waypoint::structure("Ocean Monument", [base_x + 8, 40, base_z + 8], dim);
                    waypoints.add(wp);
                }
            }
        }

        // 10. Underground Dungeon (Monster Spawner surrounded by cobblestone)
        let cx_mid = cx * 16 + 8;
        let cz_mid = cz * 16 + 8;
        for y in (-30..50).step_by(3) {
            if let Some(name) = get_block_name(cx_mid, y, cz_mid) {
                if name.contains("spawner") {
                    let feat = CollectedFeature::new(StructureType::Dungeon, cx, cz, y);
                    if cracker.add_feature(feat) {
                        let wp = Waypoint::structure("Monster Dungeon", [cx_mid, y, cz_mid], dim);
                        waypoints.add(wp);
                    }
                    break;
                }
            }
        }
    }
}
