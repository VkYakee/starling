//! SeedCracker adapter implementing VkMod for Starling / Horizon Android.
//! Features live event toasts, collection toggle, persistent mobile storage,
//! and map-ready data logging.

use crate::api::{ModMetadata, VkMod};
use crate::storage::{DiscoveredBiome, DiscoveredStructure, StorageManager, WorldSessionData};
use crate::verifier::SeedVerifier;
use seedcracker::bedrock::{BedrockBlock, BlockType, HierarchicalBedrockCracker};
use seedcracker::finders::{FinderPipeline, FinderKind, BlockObservation, BlockUpdateBatch, PatternKind};
use seedcracker::math::hashed_seed::HashedSeed;
use seedcracker::structures::TreasureDatabase;
use seedcracker::timemachine::{CrackerState, TimeMachine};

use std::collections::{HashSet, VecDeque};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct ToastLog {
    pub text: String,
    pub rgb: u32,
    pub created_at: Instant,
}

pub struct SeedCrackerMod {
    state: CrackerState,
    pub treasure_db: TreasureDatabase,
    pub verifier: SeedVerifier,
    storage: StorageManager,
    session: WorldSessionData,
    active: bool,
    collecting: bool,
    show_hud: bool,
    show_esp: bool,
    cracked_seed: Option<i64>,
    status_message: String,
    bounding_boxes: Vec<[f32; 10]>,
    toasts: VecDeque<ToastLog>,
    seen_chunks: HashSet<(String, i32, i32)>,
    solve_tx: Sender<(u64, CrackerState)>,
    solve_rx: Receiver<(u64, Vec<i64>)>,
    solving: bool,
    solve_requested: bool,
    solver_generation: u64,
    seen_biomes: HashSet<(i32, i32)>,
    seen_structures: HashSet<(String, i32, i32)>,
    seen_treasures: HashSet<(i32,i32)>,
    finder_pipeline: FinderPipeline,
}

impl SeedCrackerMod {
    pub fn new() -> Self {
        let storage = StorageManager::new();
        let session = storage.load_or_create("default_session");

        let (solve_tx, work_rx) = mpsc::channel::<(u64, CrackerState)>();
        let (result_tx, solve_rx) = mpsc::channel::<(u64, Vec<i64>)>();
        thread::Builder::new().name("SeedCracker solver".into()).spawn(move || {
            while let Ok((generation, snapshot)) = work_rx.recv() {
                // TimeMachine is always executed: it coordinates structures,
                // pillars, decorators, trees, biome/hash lifting and candidate
                // Bedrock checks. A large Bedrock sample must never disable it.
                let mut seeds = TimeMachine::solve(&snapshot).world_seeds;

                // Hierarchical Bedrock is a complementary search, not an
                // alternative solver. Merge its candidates into the same result.
                if snapshot.bedrock_blocks.len() >= 30 {
                    let bedrock_candidates =
                        HierarchicalBedrockCracker::search(&snapshot.bedrock_blocks, 4, None);
                    if let Some(hash) = snapshot.hashed_seed {
                        for candidate in bedrock_candidates {
                            seeds.extend(HashedSeed::find_world_seeds_from_hash(candidate, hash));
                        }
                    } else {
                        // Until a hash is available these remain unconfirmed
                        // 48-bit candidates; don't discard them.
                        seeds.extend(bedrock_candidates.into_iter().map(|seed| seed as i64));
                    }
                }

                seeds.sort_unstable();
                seeds.dedup();
                let _ = result_tx.send((generation, seeds));
            }
        }).expect("SeedCracker solver thread");

        Self {
            state: CrackerState::new(),
            treasure_db: TreasureDatabase::load_embedded(),
            verifier: SeedVerifier::new(),
            storage,
            session,
            active: true,
            collecting: true,
            show_hud: true,
            show_esp: true,
            cracked_seed: None,
            status_message: "SeedCracker: Listo. Recopilando datos en segundo plano...".to_string(),
            bounding_boxes: Vec::new(),
            toasts: VecDeque::new(),
            seen_chunks: HashSet::new(),
            solve_tx,
            solve_rx,
            solving: false,
            solve_requested: false,
            solver_generation: 0,
            seen_biomes: HashSet::new(),
            seen_structures: HashSet::new(),
            seen_treasures: HashSet::new(),
            finder_pipeline: FinderPipeline::default(),
        }
    }

    pub fn add_toast(&mut self, text: impl Into<String>, rgb: u32) {
        let msg = text.into();
        log::info!("SeedCracker Toast: {}", msg);
        if self.toasts.len() >= 5 {
            self.toasts.pop_front();
        }
        self.toasts.push_back(ToastLog {
            text: msg,
            rgb,
            created_at: Instant::now(),
        });
    }

    pub fn toggle_collection(&mut self) {
        self.collecting = !self.collecting;
        if self.collecting { self.finder_pipeline.resume(); } else { self.finder_pipeline.pause(); }
        if self.collecting {
            self.add_toast("▶ Recopilacion de datos ACTIVADA", 0x00FF88);
            self.status_message = "Recopilacion activa...".to_string();
        } else {
            self.add_toast("⏸ Recopilacion de datos PAUSADA", 0xFFAA00);
            self.status_message = "Recopilacion en pausa".to_string();
        }
    }

    pub fn record_biome(&mut self, x: i32, z: i32, biome_id: i32, name: &str) {
        self.verifier.update_player_pos(x, z);
        if !self.collecting || !self.finder_pipeline.finder_enabled(seedcracker::FinderType::Biome) { return; }
        let key = (x, z); // Cada coordenada biome observada aporta una restricción independiente
        if self.seen_biomes.insert(key) {
            self.state.biome_samples.push(seedcracker::biomes::BiomeSample::new(x, 0, z, biome_id));
            let msg = format!("🌿 Bioma: {} en [{}, {}]", name, x, z);
            self.add_toast(msg, 0x55FF55);

            self.session.biomes.push(DiscoveredBiome {
                x,
                z,
                biome_id,
                name: name.to_string(),
            });
            let _ = self.storage.save(&self.session);
            if self.state.structures.len()>=1 && (self.state.biome_samples.len()>=3 || self.state.hashed_seed.is_some()) {self.solve();}
        }
    }

    pub fn record_structure(&mut self, st_type: &str, chunk_x: i32, chunk_z: i32) {
        self.verifier.on_structure_found(st_type, chunk_x, chunk_z);
        if !self.collecting { return; }
        let normalized=st_type.to_ascii_lowercase().replace(' ', "_");
        let finder_type=match normalized.as_str(){
            "desert_pyramid"|"desert_temple"=>Some(seedcracker::FinderType::DesertPyramid),
            "jungle_temple"|"jungle_pyramid"=>Some(seedcracker::FinderType::JungleTemple),
            "swamp_hut"=>Some(seedcracker::FinderType::SwampHut),"igloo"=>Some(seedcracker::FinderType::Igloo),
            "village"=>Some(seedcracker::FinderType::Village),"shipwreck"=>Some(seedcracker::FinderType::Shipwreck),
            "ocean_monument"|"monument"=>Some(seedcracker::FinderType::Monument),
            "pillager_outpost"|"outpost"=>Some(seedcracker::FinderType::Outpost),
            "end_city"=>Some(seedcracker::FinderType::EndCity),"trial_chambers"=>Some(seedcracker::FinderType::TrialChambers),"ancient_city"=>Some(seedcracker::FinderType::AncientCity),_=>None};
        if finder_type.is_some_and(|kind|!self.finder_pipeline.finder_enabled(kind)){return;}
        let key=(normalized.clone(),chunk_x,chunk_z);
        if !self.seen_structures.insert(key){return;}
        let structure_type=match normalized.as_str(){
            "desert_pyramid"|"desert_temple"=>Some(seedcracker::StructureType::DesertPyramid),
            "jungle_temple"|"jungle_pyramid"=>Some(seedcracker::StructureType::JungleTemple),
            "swamp_hut"=>Some(seedcracker::StructureType::SwampHut),"igloo"=>Some(seedcracker::StructureType::Igloo),
            "village"=>Some(seedcracker::StructureType::Village),"shipwreck"=>Some(seedcracker::StructureType::Shipwreck),
            "ocean_monument"|"monument"=>Some(seedcracker::StructureType::OceanMonument),
            "pillager_outpost"|"outpost"=>Some(seedcracker::StructureType::PillagerOutpost),
            "end_city"=>Some(seedcracker::StructureType::EndCity),"trial_chambers"=>Some(seedcracker::StructureType::TrialChambers),
            "ancient_city"=>Some(seedcracker::StructureType::AncientCity),_=>None};
        if let Some(t)=structure_type {if !self.state.structures.iter().any(|s|s.structure_type==t&&s.chunk_x==chunk_x&&s.chunk_z==chunk_z){self.state.structures.push(seedcracker::StructureInstance::new(t,chunk_x,chunk_z));}}
        let bx=chunk_x*16;let bz=chunk_z*16;
        self.add_toast(format!("🏛️ Estructura: {} en Chunk ({}, {})",st_type,chunk_x,chunk_z),0xFFD700);
        self.session.structures.push(DiscoveredStructure{structure_type:st_type.to_string(),chunk_x,chunk_z,block_x:bx,block_z:bz});
        let _=self.storage.save(&self.session);
        if self.state.structures.len()>=2 {self.solve();}
    }

    /// Feeds decoded block palettes to the Rust finder engine (structures/decorators/pillars).
    pub fn on_block_snapshot(&mut self, blocks: &[BlockObservation]) {
        if !self.active || !self.collecting { return; }
        let snapshot=blocks.to_vec();
        let output=self.finder_pipeline.scan_blocks(blocks);
        for s in output.structures { if !self.state.structures.iter().any(|v|v.structure_type==s.structure_type&&v.chunk_x==s.chunk_x&&v.chunk_z==s.chunk_z){self.state.structures.push(s);} }
        for hit in output.observations.patterns {
            match hit.kind {
                PatternKind::BuriedTreasure => {
                    let chunk=(hit.x.div_euclid(16),hit.z.div_euclid(16));
                    if self.seen_treasures.insert(chunk) {
                        let observed:Vec<(i32,i32)>=self.seen_treasures.iter().copied().collect();
                        for formation in &self.treasure_db.formations {
                            if formation.matches(&observed) {for &seed in &formation.seeds {if !self.state.candidate_seeds.contains(&seed){self.state.candidate_seeds.push(seed);}}}
                        }
                    }
                },
                PatternKind::EmeraldOre => { let data=seedcracker::EmeraldOreData::new(hit.x.div_euclid(16),hit.z.div_euclid(16),hit.x,hit.y,hit.z);if !self.state.emerald_ores.iter().any(|e|e.block_x==data.block_x&&e.block_y==data.block_y&&e.block_z==data.block_z){self.state.emerald_ores.push(data);} },
                PatternKind::DesertWell => {let data=seedcracker::DesertWellData::new(hit.x.div_euclid(16),hit.z.div_euclid(16),hit.x,hit.z);if !self.state.desert_wells.iter().any(|w|w.block_x==data.block_x&&w.block_z==data.block_z){self.state.desert_wells.push(data);} },
                PatternKind::Dungeon => {
                    let mut calls=Vec::with_capacity(81);
                    for dx in -4..=4 {for dz in -4..=4 {let block=snapshot.iter().find(|b|b.x==hit.x+dx&&b.y==hit.y-1&&b.z==hit.z+dz).map(|b|b.block.to_ascii_lowercase()).unwrap_or_default();calls.push(if block.contains("mossy_cobblestone"){seedcracker::dungeon::DungeonFloorBlock::MossyCobblestone}else if block.contains("cobblestone"){seedcracker::dungeon::DungeonFloorBlock::Cobblestone}else{seedcracker::dungeon::DungeonFloorBlock::Unknown});}}
                    if !self.state.dungeon_floors.iter().any(|d|d.block_x==hit.x&&d.block_y==hit.y&&d.block_z==hit.z){self.state.dungeon_floors.push(seedcracker::dungeon::DungeonFloor::new(hit.x.div_euclid(16),hit.z.div_euclid(16),hit.x,hit.y,hit.z,9,9,calls));}
                },
                PatternKind::EndPillar => {
                    let radius=((hit.x as f64).powi(2)+(hit.z as f64).powi(2)).sqrt();
                    if (38.0..=46.0).contains(&radius) {
                        let angle=(hit.z as f64).atan2(hit.x as f64);
                        let idx=(((angle+std::f64::consts::PI)/(2.0*std::f64::consts::PI))*10.0).round() as usize%10;
                        if (76..=103).contains(&hit.y) { self.state.pillar_heights[idx]=Some(self.state.pillar_heights[idx].unwrap_or(0).max(hit.y)); }
                    }
                }, _=>{}
            }
        }
        if !self.state.candidate_seeds.is_empty() || self.state.structures.len()>=2 || self.state.emerald_ores.len()>=2 || self.state.desert_wells.len()>=2 || self.state.dungeon_floors.len()>=1 || self.state.pillar_heights.iter().filter(|p|p.is_some()).count()>=3 { self.solve(); }
    }
    /// Incremental world block mutation invalidates and replaces detector observations.
    pub fn on_block_updates(&mut self, updates: Vec<BlockObservation>, removed: Vec<(i32,i32,i32)>) {
        let mut changed=removed;
        changed.extend(updates.iter().map(|b|(b.x,b.y,b.z)));
        for (x,y,z) in changed {
            self.state.emerald_ores.retain(|o|o.block_x!=x||o.block_y!=y||o.block_z!=z);
            self.state.desert_wells.retain(|w|w.block_x!=x||w.block_z!=z);
            self.state.dungeon_floors.retain(|d|d.block_x!=x||d.block_y!=y||d.block_z!=z);
            let radius=((x as f64).powi(2)+(z as f64).powi(2)).sqrt();
            if (38.0..=46.0).contains(&radius)&&(76..=103).contains(&y) {
                let angle=(z as f64).atan2(x as f64);
                let idx=(((angle+std::f64::consts::PI)/(2.0*std::f64::consts::PI))*10.0).round() as usize%10;
                self.state.pillar_heights[idx]=None;
            }
        }
        self.finder_pipeline.apply_block_updates(BlockUpdateBatch{updates:updates.clone(),removed});
        if self.collecting && !updates.is_empty() {self.on_block_snapshot(&updates);}
    }
    /// Rebuild detector caches when dimension, version, or collection settings change.
    pub fn reload_finders(&mut self) { self.finder_pipeline.reload_finders(); self.seen_chunks.clear(); self.seen_biomes.clear(); self.seen_structures.clear(); }
    pub fn reset(&mut self) {
        self.reset_all();
    }

    /// Hard Reset: Clears memory, verifier state, cache, and wipes saved session progress.
    pub fn reset_all(&mut self) {
        self.finder_pipeline.stop();
        self.finder_pipeline = FinderPipeline::default();
        self.state = CrackerState::new();
        self.verifier = SeedVerifier::new();
        self.cracked_seed = None;
        self.bounding_boxes.clear();
        self.seen_chunks.clear();
        self.solver_generation = self.solver_generation.wrapping_add(1);
        self.solving = false;
        self.solve_requested = false;
        self.seen_biomes.clear();
        self.seen_structures.clear();
        self.seen_treasures.clear();
        self.session.cracked_seed = None;
        self.session.bedrock_chunks.clear();
        self.session.biomes.clear();
        self.session.structures.clear();
        self.session.total_scanned_chunks = 0;
        let _ = self.storage.save(&self.session);
        self.status_message = "SeedCracker: Todo reiniciado desde cero".to_string();
        self.add_toast("⚠️ REINICIO TOTAL: Todo reseteado desde cero", 0xFF5555);
    }

    /// Queue a snapshot; expensive seed searches never run on the game thread.
    pub fn solve(&mut self) {
        if self.solving { self.solve_requested=true; return; }
        let has_bedrock = !self.state.bedrock_blocks.is_empty();
        let has_treasure_candidates = !self.state.candidate_seeds.is_empty();
        let has_structures = !self.state.structures.is_empty();
        let has_pillars = self.state.pillar_heights.iter().any(Option::is_some);
        let has_other_observations = !self.state.dungeon_floors.is_empty()
            || !self.state.trees.is_empty()
            || !self.state.emerald_ores.is_empty()
            || !self.state.desert_wells.is_empty()
            || !self.state.biome_samples.is_empty();
        if !(has_bedrock || has_treasure_candidates || has_structures || has_pillars || has_other_observations) {
            return;
        }

        self.status_message = format!(
            "Coordinando cracking: {} bloques Bedrock + estructuras/pistas...",
            self.state.bedrock_blocks.len()
        );
        self.solver_generation = self.solver_generation.wrapping_add(1);
        self.solving = self
            .solve_tx
            .send((self.solver_generation, self.state.clone()))
            .is_ok();
    }

    fn poll_finders(&mut self) {
        // Drain a bounded amount per tick to keep the game thread responsive.
        for _ in 0..4 {
            let Some((_sequence, output)) = self.finder_pipeline.run_one() else { break; };
            for block in output.bedrock {
                if !self.state.bedrock_blocks.iter().any(|b| b.x==block.x && b.y==block.y && b.z==block.z) {
                    self.state.bedrock_blocks.push(block);
                }
            }
            for structure in output.structures {
                if !self.state.structures.iter().any(|s|s.structure_type==structure.structure_type&&s.chunk_x==structure.chunk_x&&s.chunk_z==structure.chunk_z){self.state.structures.push(structure);}
            }
            if self.state.bedrock_blocks.len()>=30 && !self.solving {self.solve();}
        }
    }

    fn poll_solver(&mut self) {
        while let Ok((generation, candidates)) = self.solve_rx.try_recv() {
            // Ignore results from a reset or a superseded solve request.
            if generation != self.solver_generation {
                continue;
            }
            self.solving = false;
            let rerun=self.solve_requested; self.solve_requested=false;
            if rerun { self.solve(); continue; }
            if candidates.is_empty() {
                self.status_message = "Análisis terminado: sin candidatos".into();
                continue;
            }
            let candidates_u64: Vec<u64> =
                candidates.iter().map(|&seed| seed as u64).collect();
            if let Some(&seed) = candidates.first() {
                self.cracked_seed = Some(seed);
                self.session.cracked_seed = Some(seed);
                let _ = self.storage.save(&self.session);
                self.status_message = format!("⭐ Semilla/candidato encontrado: {seed}");
                self.add_toast(self.status_message.clone(), 0x00FF00);
            }
            crate::publish_seed_candidates(&candidates_u64);
            self.verifier.set_candidates(candidates_u64);
        }
    }

}

impl VkMod for SeedCrackerMod {
    fn metadata(&self) -> ModMetadata {
        ModMetadata {
            id: "seedcracker",
            name: "SeedCracker (Horizon Rust)",
            version: "1.0.0",
            description: "Cracker de semillas in-game nativo de ultra-alto rendimiento para Starling Android",
            author: "VkYakee & SeedCracker Team",
        }
    }

    fn on_hashed_seed(&mut self, hashed_seed: u64) {
        if !self.active {
            return;
        }
        log::info!("SeedCracker: Interceptado Hashed Seed = 0x{:016X}", hashed_seed);
        self.state.hashed_seed = Some(hashed_seed as i64);
        self.finder_pipeline.resume();
        self.session.hashed_seed = Some(hashed_seed);
        let _ = self.storage.save(&self.session);

        self.add_toast(format!("🔑 Hashed Seed: 0x{:016X}", hashed_seed), 0x00FFFF);
        self.status_message = format!("Hashed Seed: 0x{:016X}", hashed_seed);
    }

    fn on_tick(&mut self) { self.poll_finders(); self.poll_solver(); }

    fn on_chunk_loaded(
        &mut self,
        cx: i32,
        cz: i32,
        dimension: &str,
        bedrock_mask: Option<&[u8; 256]>,
    ) {
        if !self.active || !self.collecting || self.cracked_seed.is_some() {
            return;
        }

        let chunk_key = (dimension.to_owned(), cx, cz);
        if !self.seen_chunks.insert(chunk_key) { return; }
        self.session.total_scanned_chunks += 1;

        if let Some(mask) = bedrock_mask {
            let is_nether = dimension.contains("nether");
            let base_y = if is_nether { 123 } else { 0 };

            let mut count = 0;
            let mut observed_bedrock = Vec::new();
            for z in 0..16 {
                for x in 0..16 {
                    let idx = (z << 4) | x;
                    if mask[idx] != 0 {
                        count += 1;
                        let bx = (cx << 4) + x as i32;
                        let bz = (cz << 4) + z as i32;
                        observed_bedrock.push(BedrockBlock::new(bx, base_y, bz, BlockType::Bedrock));
                    }
                }
            }

            if count > 0 {
                let _ = self.finder_pipeline.enqueue_chunk(FinderKind::Combined, dimension, cx, cz, observed_bedrock);
                let hex_mask: String = mask.iter().map(|b| if *b != 0 { '1' } else { '0' }).collect();
                self.session.bedrock_chunks.insert(format!("{},{}", cx, cz), hex_mask);
                let _ = self.storage.save(&self.session);

                self.add_toast(format!("⬛ Bedrock: {} bloques en Chunk ({}, {})", count, cx, cz), 0xAA55FF);

                if self.state.bedrock_blocks.len() >= 30 && self.cracked_seed.is_none() {
                    self.solve();
                }
            }
        }
    }

    fn on_block_snapshot(&mut self, chunk_x:i32, chunk_z:i32, dimension:&str, blocks:&[(i32,i32,i32,String)]) {
        let _=(chunk_x,chunk_z,dimension);
        let observations:Vec<BlockObservation>=blocks.iter().map(|(x,y,z,name)|BlockObservation::new(*x,*y,*z,name.clone())).collect();
        self.on_block_snapshot(&observations);
    }

    fn on_block_update(&mut self, x:i32, y:i32, z:i32, block:Option<&str>) {
        let updates=block.map(|name|vec![BlockObservation::new(x,y,z,name)]).unwrap_or_default();
        let removed=if block.is_none(){vec![(x,y,z)]}else{Vec::new()};
        self.on_block_updates(updates,removed);
    }

    fn on_biome_found(&mut self, x: i32, z: i32, biome_id: i32, name: &str) {
        self.record_biome(x, z, biome_id, name);
    }

    fn on_player_pos(&mut self, x: i32, y: i32, z: i32) {
        let _ = y;
        self.verifier.update_player_pos(x, z);
    }

    fn on_structure_found(&mut self, st_type: &str, chunk_x: i32, chunk_z: i32) {
        self.record_structure(st_type, chunk_x, chunk_z);
    }

    fn on_command(&mut self, command: &str) -> bool {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return false;
        }

        let cmd = parts[0];
        if cmd == "seedcracker" || cmd == "cracker" || cmd == "sc" {
            let sub = if parts.len() > 1 { parts[1] } else { "gui" };
            match sub {
                "gui" | "menu" => {
                    self.show_hud = !self.show_hud;
                    self.add_toast(if self.show_hud { "HUD: Visible" } else { "HUD: Oculto" }, 0xFFFFFF);
                }
                "collect" | "toggle" => {
                    self.toggle_collection();
                }
                "reset" | "reset_all" | "clear" => {
                    self.reset_all();
                }
                "finder" => {
                    let name=parts.get(2).copied().unwrap_or("reload");
                    if name=="reload"||name=="reset" { self.reload_finders(); }
                    else {
                        let kind=match name {
                            "treasure"|"buried_treasure"=>Some(seedcracker::FinderType::BuriedTreasure),
                            "desert_pyramid"|"desert_temple"=>Some(seedcracker::FinderType::DesertPyramid),
                            "jungle_temple"=>Some(seedcracker::FinderType::JungleTemple),"swamp_hut"=>Some(seedcracker::FinderType::SwampHut),
                            "igloo"=>Some(seedcracker::FinderType::Igloo),"village"=>Some(seedcracker::FinderType::Village),"biome"=>Some(seedcracker::FinderType::Biome),
                            "shipwreck"=>Some(seedcracker::FinderType::Shipwreck),"monument"=>Some(seedcracker::FinderType::Monument),
                            "outpost"=>Some(seedcracker::FinderType::Outpost),"end_city"=>Some(seedcracker::FinderType::EndCity),
                            "trial_chambers"=>Some(seedcracker::FinderType::TrialChambers),"ancient_city"=>Some(seedcracker::FinderType::AncientCity),"end_pillars"=>Some(seedcracker::FinderType::EndPillars),
                            "end_gateway"=>Some(seedcracker::FinderType::EndGateway),"dungeon"=>Some(seedcracker::FinderType::Dungeon),
                            "emerald_ore"=>Some(seedcracker::FinderType::EmeraldOre),"desert_well"=>Some(seedcracker::FinderType::DesertWell),
                            "warped_fungus"=>Some(seedcracker::FinderType::WarpedFungus),_=>None};
                        if let Some(kind)=kind {let enabled=parts.get(3).copied().unwrap_or("on")!="off";self.finder_pipeline.set_finder_enabled(kind,enabled);self.add_toast(format!("Finder {} {}",name,if enabled{"activado"}else{"desactivado"}),0x00FF88);}
                    }
                }
                "solve" => {
                    self.solve();
                }
                "esp" => {
                    self.show_esp = !self.show_esp;
                    self.add_toast(if self.show_esp { "ESP: Activado" } else { "ESP: Desactivado" }, 0xFFFFFF);
                }
                "export" => {
                    if let Ok(path) = self.storage.export_map_csv(&self.session) {
                        self.add_toast(format!("Mapa exportado a {:?}", path), 0x00FF88);
                    }
                }
                "status" => {
                    log::info!("SeedCracker Estado: {}", self.status_message);
                }
                "help" => {
                    log::info!("Comandos: /sc [gui | collect | solve | finder reload|finder <name> on/off | reset | esp | export | status]");
                }
                _ => {}
            }
            return true;
        }

        false
    }

    fn hud_lines(&self) -> Vec<String> {
        if !self.show_hud {
            return Vec::new();
        }

        let mut lines = Vec::new();
        lines.push("┌─ [ VkYakeeMods ] SeedCracker v1.0.0 ─┐".to_string());
        
        let col_status = if self.collecting { "[▶ RECOPILACION: ACTIVA]" } else { "[⏸ RECOPILACION: PAUSADA]" };
        lines.push(format!("│ Modo: {} (Toca p/ alternar)", col_status));

        if let Some(seed) = self.cracked_seed {
            lines.push(format!("│ ⭐ SEMILLA DESCIFRADA: {}", seed));
        } else {
            lines.push(format!("│ Estado: {}", self.status_message));
            lines.push(format!("│ Chunks: {} | Bedrock: {} blqs | Biomas: {}", 
                self.session.total_scanned_chunks, 
                self.state.bedrock_blocks.len(),
                self.session.biomes.len()
            ));
        }

        let verifier_lines = self.verifier.hud_lines();
        if !verifier_lines.is_empty() {
            lines.push("├─ Asistente de Verificación:".to_string());
            for vl in verifier_lines {
                lines.push(format!("│  {}", vl));
            }
        }

        // Mostrar últimos eventos / toasts
        if !self.toasts.is_empty() {
            lines.push("├─ Registros Recientes:".to_string());
            for toast in self.toasts.iter().rev().take(3) {
                lines.push(format!("│  {}", toast.text));
            }
        }
        lines.push("└─ (/sc collect | /sc solve | /sc reset_all) ─┘".to_string());

        lines
    }

    fn render_bounding_boxes(&self) -> Vec<[f32; 10]> {
        if !self.show_esp {
            Vec::new()
        } else {
            self.bounding_boxes.clone()
        }
    }

    fn settings_buttons(&self) -> Vec<String> {
        vec![
            if self.collecting { "Recopilacion: [PAUSAR]" } else { "Recopilacion: [INICIAR]" }.to_string(),
            if self.active { "Mod: [DESACTIVAR]" } else { "Mod: [ACTIVAR]" }.to_string(),
            if self.show_hud { "HUD: [OCULTAR]" } else { "HUD: [MOSTRAR]" }.to_string(),
            if self.show_esp { "ESP: [DESACTIVAR]" } else { "ESP: [ACTIVAR]" }.to_string(),
            "Resolver Semilla Ahora".to_string(),
            "Exportar Mapa CSV".to_string(),
            "🔄 Reiniciar Todo (Hard Reset)".to_string(),
        ]
    }

    fn on_settings_click(&mut self, button_idx: usize) {
        match button_idx {
            0 => self.toggle_collection(),
            1 => self.active = !self.active,
            2 => self.show_hud = !self.show_hud,
            3 => self.show_esp = !self.show_esp,
            4 => self.solve(),
            5 => {
                if let Ok(path) = self.storage.export_map_csv(&self.session) {
                    self.add_toast(format!("Mapa guardado en {:?}", path), 0x00FF88);
                }
            }
            6 => self.reset_all(),
            _ => {}
        }
    }
}
