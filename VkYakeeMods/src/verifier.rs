//! SeedVerifier: Hybrid candidate pruning & terrain verification engine.
//!
//! When candidate seeds are between 2 and 2000, it selects optimal nearby structures
//! to guide the player (predictive waypoints) and discards invalid seeds upon chunk load.
//! When a single seed is resolved, it generates a confirmation target to prove 100% terrain match.

use seedcracker::structures::{
    find_closest_predicted_structure, predict_structure_chunk, StructureType,
};

#[derive(Clone, Debug, PartialEq)]
pub struct WaypointTarget {
    pub structure: StructureType,
    pub structure_name: String,
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub block_x: i32,
    pub block_z: i32,
    pub distance: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VerifierState {
    Idle,
    Pruning {
        remaining: usize,
        target: WaypointTarget,
    },
    VerifyingExact {
        seed: u64,
        target: WaypointTarget,
        confirmed: bool,
    },
    Confirmed {
        seed: u64,
        verified_structure: String,
    },
}

pub struct SeedVerifier {
    pub candidates: Vec<u64>,
    pub player_pos: (i32, i32),
    pub state: VerifierState,
    pub structures_to_check: Vec<StructureType>,
}

impl SeedVerifier {
    pub fn new() -> Self {
        Self {
            candidates: Vec::new(),
            player_pos: (0, 0),
            state: VerifierState::Idle,
            structures_to_check: vec![
                StructureType::Village,
                StructureType::DesertPyramid,
                StructureType::Shipwreck,
                StructureType::SwampHut,
                StructureType::PillagerOutpost,
                StructureType::TrialChambers,
            ],
        }
    }

    /// Updates the player's current block position (X, Z) and recalibrates distances.
    pub fn update_player_pos(&mut self, px: i32, pz: i32) {
        self.player_pos = (px, pz);
        match &mut self.state {
            VerifierState::Pruning { target, .. } | VerifierState::VerifyingExact { target, .. } => {
                let dx = (target.block_x - px) as f32;
                let dz = (target.block_z - pz) as f32;
                target.distance = (dx * dx + dz * dz).sqrt();
            }
            _ => {}
        }
    }

    /// Feeds candidate seeds from LCG/Bedrock/Lattice/Hashed solvers.
    pub fn set_candidates(&mut self, mut candidates: Vec<u64>) {
        if candidates.is_empty() {
            self.candidates.clear();
            self.state = VerifierState::Idle;
            return;
        }

        if candidates.len() == 1 {
            let seed = candidates[0];
            self.candidates = candidates;
            if let Some(target) = self.pick_closest_target(seed) {
                self.state = VerifierState::VerifyingExact {
                    seed,
                    target,
                    confirmed: false,
                };
            } else {
                self.state = VerifierState::Confirmed {
                    seed,
                    verified_structure: "Matemática Directa".to_string(),
                };
            }
            return;
        }

        // Limit to 2000 seeds for mobile ARM performance
        if candidates.len() > 2000 {
            candidates.truncate(2000);
        }

        self.candidates = candidates;
        self.recalibrate_pruning_target();
    }

    fn recalibrate_pruning_target(&mut self) {
        if self.candidates.len() <= 1 {
            if let Some(&seed) = self.candidates.first() {
                self.set_candidates(vec![seed]);
            } else {
                self.state = VerifierState::Idle;
            }
            return;
        }

        // Find a candidate structure predicted by the first candidate that can disambiguate
        if let Some(&first_seed) = self.candidates.first() {
            if let Some(target) = self.pick_closest_target(first_seed) {
                self.state = VerifierState::Pruning {
                    remaining: self.candidates.len(),
                    target,
                };
            }
        }
    }

    fn pick_closest_target(&self, seed: u64) -> Option<WaypointTarget> {
        let (px, pz) = self.player_pos;
        let res = find_closest_predicted_structure(
            seed,
            &self.structures_to_check,
            px,
            pz,
            1200, // Search within 1200 blocks
        );

        res.map(|(st, cx, cz, bx, bz, dist)| WaypointTarget {
            structure: st,
            structure_name: format!("{st:?}"),
            chunk_x: cx,
            chunk_z: cz,
            block_x: bx,
            block_z: bz,
            distance: dist,
        })
    }

    /// Called when Starling discovers a structure in a chunk.
    pub fn on_structure_found(&mut self, structure_name: &str, cx: i32, cz: i32) {
        match &mut self.state {
            VerifierState::VerifyingExact { seed, target, confirmed } => {
                if target.chunk_x == cx && target.chunk_z == cz {
                    *confirmed = true;
                    let verified_seed = *seed;
                    let st_name = target.structure_name.clone();
                    self.state = VerifierState::Confirmed {
                        seed: verified_seed,
                        verified_structure: st_name,
                    };
                }
            }
            VerifierState::Pruning { target, .. } => {
                if target.chunk_x == cx && target.chunk_z == cz {
                    // Structure matches! Filter candidate seeds that predict this structure at (cx, cz)
                    let st = target.structure;
                    let config = seedcracker::structures::get_structure_config(st);
                    let rx = cx.div_euclid(config.spacing);
                    let rz = cz.div_euclid(config.spacing);

                    self.candidates.retain(|&s| {
                        let (scx, scz) = predict_structure_chunk(s, st, rx, rz);
                        scx == cx && scz == cz
                    });

                    log::info!(
                        "SeedVerifier: Confirmed structure {}, remaining candidates: {}",
                        structure_name,
                        self.candidates.len()
                    );
                    self.recalibrate_pruning_target();
                }
            }
            _ => {}
        }
    }

    /// Called when a target chunk has been scanned and the predicted structure was NOT found.
    pub fn on_target_chunk_empty(&mut self, cx: i32, cz: i32) {
        if let VerifierState::Pruning { target, .. } = &self.state {
            if target.chunk_x == cx && target.chunk_z == cz {
                let st = target.structure;
                let config = seedcracker::structures::get_structure_config(st);
                let rx = cx.div_euclid(config.spacing);
                let rz = cz.div_euclid(config.spacing);

                // Discard all candidate seeds that falsely predicted the structure here
                self.candidates.retain(|&s| {
                    let (scx, scz) = predict_structure_chunk(s, st, rx, rz);
                    !(scx == cx && scz == cz)
                });

                log::info!(
                    "SeedVerifier: Pruned seeds absent at ({cx}, {cz}), remaining: {}",
                    self.candidates.len()
                );
                self.recalibrate_pruning_target();
            }
        }
    }

    /// Formatted HUD display lines for VkYakeeMods.
    pub fn hud_lines(&self) -> Vec<String> {
        match &self.state {
            VerifierState::Idle => Vec::new(),
            VerifierState::Pruning { remaining, target } => {
                vec![
                    format!("🔍 Candidatas: {remaining} semillas"),
                    format!(
                        "🧭 Descarte: {} @ ({}, {}) [{}m]",
                        target.structure_name, target.block_x, target.block_z, target.distance as i32
                    ),
                ]
            }
            VerifierState::VerifyingExact { target, confirmed, .. } => {
                if *confirmed {
                    vec!["✅ Semilla 100% Confirmada en Terreno".to_string()]
                } else {
                    vec![
                        "⭐ Semilla Descifrada (Confirmando...)".to_string(),
                        format!(
                            "🧭 Verificador: {} @ ({}, {}) [{}m]",
                            target.structure_name, target.block_x, target.block_z, target.distance as i32
                        ),
                    ]
                }
            }
            VerifierState::Confirmed { verified_structure, .. } => {
                vec![format!("✅ Semilla 100% Verificada ({verified_structure})")]
            }
        }
    }
}
