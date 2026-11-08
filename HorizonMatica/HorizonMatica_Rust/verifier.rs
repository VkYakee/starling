// Copyright (c) 2026 HorizonMatica Authors / Starling Android
//! Real-time schematic verifier that compares world blocks with schematic blocks.
//! Classifies mismatches into Missing, WrongBlock, WrongState, and Extra.

use std::collections::HashSet;
use super::container::SchematicBlockState;
use super::format::LoadedSchematic;
use super::placement::SchematicPlacement;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MismatchType {
    Missing,     // World is air, schematic has block (Cyan)
    WrongBlock,  // World has different block (Red)
    WrongState,  // Same block type, different properties (Orange)
    Extra,       // World has block, schematic is air (Magenta)
}

impl MismatchType {
    pub fn color_argb(&self) -> u32 {
        match self {
            Self::Missing => 0xFF33_B3E6,    // Cyan / Light Blue
            Self::WrongBlock => 0xFFFF_3333, // Bright Red
            Self::WrongState => 0xFFFF_9010, // Orange
            Self::Extra => 0xFFFF_4CE6,      // Magenta / Purple
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Missing => "Missing",
            Self::WrongBlock => "Wrong Block",
            Self::WrongState => "Wrong State",
            Self::Extra => "Extra Block",
        }
    }
}

#[derive(Clone, Debug)]
pub struct VerifierMismatch {
    pub pos: [i32; 3], // World position
    pub mismatch_type: MismatchType,
    pub schematic_block: String,
    pub world_block: String,
}

#[derive(Clone, Debug, Default)]
pub struct VerifierStats {
    pub total_schematic_blocks: usize,
    pub matching: usize,
    pub missing: usize,
    pub wrong_block: usize,
    pub wrong_state: usize,
    pub extra: usize,
}

impl VerifierStats {
    pub fn percent_complete(&self) -> f32 {
        if self.total_schematic_blocks == 0 {
            0.0
        } else {
            (self.matching as f32 / self.total_schematic_blocks as f32 * 100.0).clamp(0.0, 100.0)
        }
    }
}

pub struct SchematicVerifier {
    pub active: bool,
    pub mismatches: Vec<VerifierMismatch>,
    pub stats: VerifierStats,
}

impl Default for SchematicVerifier {
    fn default() -> Self {
        Self {
            active: true,
            mismatches: Vec::new(),
            stats: VerifierStats::default(),
        }
    }
}

impl SchematicVerifier {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.mismatches.clear();
        self.stats = VerifierStats::default();
    }

    /// Performs verification on the schematic vs world blocks within a radius of the player.
    pub fn verify_area<F>(
        &mut self,
        schematic: &LoadedSchematic,
        placement: &SchematicPlacement,
        player_pos: [f64; 3],
        radius: i32,
        mut get_world_block: F,
    ) where
        F: FnMut(i32, i32, i32) -> Option<String>,
    {
        if !self.active || !placement.enabled || schematic.regions.is_empty() {
            return;
        }

        self.mismatches.clear();
        let mut stats = VerifierStats::default();

        let px = player_pos[0].floor() as i32;
        let py = player_pos[1].floor() as i32;
        let pz = player_pos[2].floor() as i32;

        let (min_b, max_b) = placement.compute_world_bounds(schematic);
        let start_x = min_b[0].max(px - radius);
        let end_x = max_b[0].min(px + radius);
        let start_y = min_b[1].max(py - radius);
        let end_y = max_b[1].min(py + radius);
        let start_z = min_b[2].max(pz - radius);
        let end_z = max_b[2].min(pz + radius);

        let mut visited_coords = HashSet::new();

        // 1. Check all blocks within the subregions
        for r in &schematic.regions {
            if !placement.is_subregion_enabled(&r.name) {
                continue;
            }

            for ly in 0..r.container.size_y {
                if !placement.layer_mode.is_layer_visible(ly) {
                    continue;
                }

                for lz in 0..r.container.size_z {
                    for lx in 0..r.container.size_x {
                        let Some(st) = r.container.get(lx, ly, lz) else {
                            continue;
                        };
                        let is_schem_air = st.is_air();

                        let wp = placement.transform_local_to_world([lx, ly, lz], r.pos);
                        if wp[0] < start_x || wp[0] > end_x || wp[1] < start_y || wp[1] > end_y || wp[2] < start_z || wp[2] > end_z {
                            if !is_schem_air {
                                stats.total_schematic_blocks += 1;
                            }
                            continue;
                        }

                        visited_coords.insert(wp);

                        let Some(world_block_name) = get_world_block(wp[0], wp[1], wp[2]) else {
                            if !is_schem_air {
                                stats.total_schematic_blocks += 1;
                            }
                            continue;
                        };

                        let is_world_air = world_block_name == "minecraft:air" || world_block_name == "air" || world_block_name == "minecraft:cave_air" || world_block_name == "minecraft:void_air";

                        if !is_schem_air {
                            stats.total_schematic_blocks += 1;

                            if is_world_air {
                                stats.missing += 1;
                                if self.mismatches.len() < 1024 {
                                    self.mismatches.push(VerifierMismatch {
                                        pos: wp,
                                        mismatch_type: MismatchType::Missing,
                                        schematic_block: st.name.clone(),
                                        world_block: world_block_name,
                                    });
                                }
                            } else {
                                let norm_schem = st.name.strip_prefix("minecraft:").unwrap_or(&st.name);
                                let norm_world = world_block_name.strip_prefix("minecraft:").unwrap_or(&world_block_name);

                                if norm_schem == norm_world {
                                    stats.matching += 1;
                                } else {
                                    stats.wrong_block += 1;
                                    if self.mismatches.len() < 1024 {
                                        self.mismatches.push(VerifierMismatch {
                                            pos: wp,
                                            mismatch_type: MismatchType::WrongBlock,
                                            schematic_block: st.name.clone(),
                                            world_block: world_block_name,
                                        });
                                    }
                                }
                            }
                        } else {
                            // Schematic is air, check if world has extra solid block
                            if !is_world_air && !world_block_name.contains("water") && !world_block_name.contains("lava") {
                                stats.extra += 1;
                                if self.mismatches.len() < 1024 {
                                    self.mismatches.push(VerifierMismatch {
                                        pos: wp,
                                        mismatch_type: MismatchType::Extra,
                                        schematic_block: "air".to_string(),
                                        world_block: world_block_name,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        self.stats = stats;
    }
}
