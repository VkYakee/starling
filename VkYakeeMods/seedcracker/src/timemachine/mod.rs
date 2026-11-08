use crate::bedrock::{BedrockBlock, BedrockSolver};
use crate::biomes::{BiomeLifter, BiomeSample, MC_1_20};
use crate::decorators::{DesertWellData, EmeraldOreData};
use crate::dungeon::{DungeonFloor, DungeonSolver, DUNGEON_SALT_MODERN};
use crate::math::hashed_seed::HashedSeed;
use crate::pillars::{EndPillars, PILLAR_COUNT};
use crate::structures::lifting::StructureLifter;
use crate::structures::StructureInstance;
use crate::trees::{BlobTreeData, TreeCracker};

#[derive(Debug, Clone, Default)]
pub struct CrackerState {
    pub bedrock_blocks: Vec<BedrockBlock>,
    pub candidate_seeds: Vec<u64>,
    pub dungeon_floors: Vec<DungeonFloor>,
    pub pillar_heights: [Option<i32>; PILLAR_COUNT],
    pub structures: Vec<StructureInstance>,
    pub trees: Vec<BlobTreeData>,
    pub emerald_ores: Vec<EmeraldOreData>,
    pub desert_wells: Vec<DesertWellData>,
    pub biome_samples: Vec<BiomeSample>,
    pub hashed_seed: Option<i64>,
    pub target_mc_version: i32,
}

impl CrackerState {
    pub fn new() -> Self {
        Self {
            bedrock_blocks: Vec::new(),
            candidate_seeds: Vec::new(),
            dungeon_floors: Vec::new(),
            pillar_heights: [None; PILLAR_COUNT],
            structures: Vec::new(),
            trees: Vec::new(),
            emerald_ores: Vec::new(),
            desert_wells: Vec::new(),
            biome_samples: Vec::new(),
            hashed_seed: None,
            target_mc_version: MC_1_20,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CrackerResult {
    pub bedrock_seeds: Vec<u64>,
    pub pillar_seeds: Vec<u16>,
    pub structure_seeds: Vec<u64>,
    pub world_seeds: Vec<i64>,
}

pub struct TimeMachine;

impl TimeMachine {
    pub fn solve(state: &CrackerState) -> CrackerResult {
        let mut result = CrackerResult::default();

        // Phase 1: Bedrock Cracker (if bedrock blocks provided)
        if !state.bedrock_blocks.is_empty() {
            println!("[TimeMachine] Ejecutando análisis de Bedrock del Nether...");
            let candidates = if !state.candidate_seeds.is_empty() {
                &state.candidate_seeds[..]
            } else {
                &[]
            };

            if !candidates.is_empty() {
                let found = BedrockSolver::crack_from_list(candidates, &state.bedrock_blocks);
                result.bedrock_seeds = found;
                for &s in &result.bedrock_seeds {
                    if !result.structure_seeds.contains(&s) {
                        result.structure_seeds.push(s);
                    }
                }
            } else {
                println!("[TimeMachine] Aviso: No se suministraron semillas candidatas para Bedrock (proporcionar lista o ejecutar crack_bedrock_full).");
            }
        }

        // Phase 2: End Pillars Cracker (if any pillar heights known)
        let known_pillars = state.pillar_heights.iter().filter(|p| p.is_some()).count();
        if known_pillars >= 3 {
            println!("[TimeMachine] Analizando configuración de Pilares del End ({} observados)...", known_pillars);
            result.pillar_seeds = EndPillars::find_pillar_seeds(&state.pillar_heights);
            println!("[TimeMachine] Pilares reducidos a {} candidatos.", result.pillar_seeds.len());
        }

        // Phase 3: Structure Lifting (if 2 or more structures known)
        if state.structures.len() >= 2 {
            println!("[TimeMachine] Ejecutando elevación rápida por congruencias con {} estructuras...", state.structures.len());
            let lifted = StructureLifter::lift_structures(&state.structures);
            println!("[TimeMachine] Semillas de estructura encontradas: {}", lifted.len());
            for s in lifted {
                if !result.structure_seeds.contains(&s) {
                    result.structure_seeds.push(s);
                }
            }
        }

        // Phase 4: Dungeon Floor Cracker
        for floor in &state.dungeon_floors {
            println!("[TimeMachine] Descifrando suelo de mazmorra ({} bloques)...", floor.calls.len());
            let dec_seeds = DungeonSolver::crack_decorator_seed(floor, 16);
            for dec in dec_seeds {
                let st_seeds = DungeonSolver::decorator_to_structure_seeds(
                    dec,
                    floor.chunk_x,
                    floor.chunk_z,
                    DUNGEON_SALT_MODERN,
                );
                for s in st_seeds {
                    if !result.structure_seeds.contains(&s) {
                        result.structure_seeds.push(s);
                    }
                }
            }
        }

        // Phase 5: Tree Cracker
        for tree in &state.trees {
            println!("[TimeMachine] Descifrando árbol ({:?}, altura={})...", tree.tree_type, tree.trunk_height);
            let tree_seeds = TreeCracker::crack_from_tree(tree, &result.structure_seeds, 10000);
            for s in tree_seeds {
                if !result.structure_seeds.contains(&s) {
                    result.structure_seeds.push(s);
                }
            }
        }

        // Phase 5b: Decorator bit filtering (Emeralds and Desert Wells)
        if !state.emerald_ores.is_empty() && !result.structure_seeds.is_empty() {
            println!("[TimeMachine] Filtrando candidatos con {} menas de esmeralda...", state.emerald_ores.len());
            result.structure_seeds.retain(|&seed| {
                state.emerald_ores.iter().all(|e| e.test_seed(seed))
            });
            println!("[TimeMachine] Semillas restantes tras esmeraldas: {}", result.structure_seeds.len());
        }

        if !state.desert_wells.is_empty() && !result.structure_seeds.is_empty() {
            println!("[TimeMachine] Filtrando candidatos con {} pozos del desierto...", state.desert_wells.len());
            result.structure_seeds.retain(|&seed| {
                state.desert_wells.iter().all(|w| w.test_seed(seed))
            });
            println!("[TimeMachine] Semillas restantes tras pozos: {}", result.structure_seeds.len());
        }

        // Phase 6: Cross-verification with End Pillars if both present
        if !result.pillar_seeds.is_empty() && !result.structure_seeds.is_empty() {
            println!("[TimeMachine] Cruzando semillas de estructura con pilares del End...");
            result.structure_seeds.retain(|&seed| {
                let p = EndPillars::pillar_seed_from_structure_seed(seed);
                result.pillar_seeds.contains(&p)
            });
            println!("[TimeMachine] Semillas de estructura tras filtrado con pilares: {}", result.structure_seeds.len());
        }

        // Phase 7: Lift 48-bit structure seed(s) to 64-bit world seed(s)
        for &structure_seed in &result.structure_seeds {
            // If hashed seed is available (Minecraft 1.15+ packet)
            if let Some(target_hash) = state.hashed_seed {
                println!("[TimeMachine] Verificando hashed seed ({})...", target_hash);
                let world_candidates = HashedSeed::find_world_seeds_from_hash(structure_seed, target_hash);
                for ws in world_candidates {
                    if !result.world_seeds.contains(&ws) {
                        result.world_seeds.push(ws);
                    }
                }
            }

            // If biome samples are available, verify using Cubiomes
            if !state.biome_samples.is_empty() {
                println!("[TimeMachine] Verificando biomas en cubiomes ({} muestras)...", state.biome_samples.len());
                let ws_candidates = BiomeLifter::lift_seed(
                    structure_seed,
                    &state.biome_samples,
                    state.target_mc_version,
                );
                for ws in ws_candidates {
                    if !result.world_seeds.contains(&ws) {
                        result.world_seeds.push(ws);
                    }
                }
            }
        }

        result
    }
}
