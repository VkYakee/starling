//! SeedCracker adapter implementing VkMod for Starling / Horizon Android.
//! Features live event toasts, collection toggle, persistent mobile storage,
//! and map-ready data logging.

use crate::api::{ModMetadata, VkMod};
use crate::storage::{DiscoveredBiome, DiscoveredStructure, StorageManager, WorldSessionData};
use crate::verifier::SeedVerifier;
use seedcracker::bedrock::{BedrockBlock, BlockType, HierarchicalBedrockCracker};
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
    solve_tx: Sender<CrackerState>,
    solve_rx: Receiver<Vec<i64>>,
    solving: bool,
    solver_generation: u64,
    seen_biomes: HashSet<(i32, i32)>,
    seen_structures: HashSet<(i32, i32)>,
}

impl SeedCrackerMod {
    pub fn new() -> Self {
        let storage = StorageManager::new();
        let session = storage.load_or_create("default_session");

        let (solve_tx, work_rx) = mpsc::channel::<CrackerState>();
        let (result_tx, solve_rx) = mpsc::channel::<Vec<i64>>();
        thread::Builder::new().name("SeedCracker solver".into()).spawn(move || {
            while let Ok(snapshot) = work_rx.recv() {
                let seeds = if snapshot.bedrock_blocks.len() >= 20 {
                    let candidates = HierarchicalBedrockCracker::search(&snapshot.bedrock_blocks, 4, None);
                    if let Some(hash) = snapshot.hashed_seed {
                        candidates.into_iter().flat_map(|c| HashedSeed::find_world_seeds_from_hash(c, hash)).collect()
                    } else { candidates.into_iter().map(|c| c as i64).collect() }
                } else { TimeMachine::solve(&snapshot).world_seeds };
                let _ = result_tx.send(seeds);
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
            solver_generation: 0,
            seen_biomes: HashSet::new(),
            seen_structures: HashSet::new(),
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
        if !self.collecting {
            return;
        }
        let key = (x >> 6, z >> 6); // Agrupar por áreas de 64 bloques
        if self.seen_biomes.insert(key) {
            let msg = format!("🌿 Bioma: {} en [{}, {}]", name, x, z);
            self.add_toast(msg, 0x55FF55);

            self.session.biomes.push(DiscoveredBiome {
                x,
                z,
                biome_id,
                name: name.to_string(),
            });
            let _ = self.storage.save(&self.session);
        }
    }

    pub fn record_structure(&mut self, st_type: &str, chunk_x: i32, chunk_z: i32) {
        self.verifier.on_structure_found(st_type, chunk_x, chunk_z);
        if !self.collecting {
            return;
        }
        let key = (chunk_x, chunk_z);
        if self.seen_structures.insert(key) {
            let bx = chunk_x * 16;
            let bz = chunk_z * 16;
            let msg = format!("🏛️ Estructura: {} en Chunk ({}, {})", st_type, chunk_x, chunk_z);
            self.add_toast(msg, 0xFFD700);

            self.session.structures.push(DiscoveredStructure {
                structure_type: st_type.to_string(),
                chunk_x,
                chunk_z,
                block_x: bx,
                block_z: bz,
            });
            let _ = self.storage.save(&self.session);
        }
    }

    pub fn reset(&mut self) {
        self.reset_all();
    }

    /// Hard Reset: Clears memory, verifier state, cache, and wipes saved session progress.
    pub fn reset_all(&mut self) {
        self.state = CrackerState::new();
        self.verifier = SeedVerifier::new();
        self.cracked_seed = None;
        self.bounding_boxes.clear();
        self.seen_chunks.clear();
        self.solver_generation = self.solver_generation.wrapping_add(1);
        self.solving = false;
        self.seen_biomes.clear();
        self.seen_structures.clear();
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
        if self.solving { return; }
        if self.state.bedrock_blocks.len() < 20 && self.state.structures.is_empty()
            && self.state.pillar_heights.iter().all(Option::is_none) { return; }
        self.status_message = format!("Analizando {} bloques en worker...", self.state.bedrock_blocks.len());
        self.solving = self.solve_tx.send(self.state.clone()).is_ok();
    }

    fn poll_solver(&mut self) {
        while let Ok(candidates) = self.solve_rx.try_recv() {
            self.solving = false;
            if candidates.is_empty() { self.status_message = "Análisis terminado: sin candidatos".into(); continue; }
            let candidates_u64: Vec<u64> = candidates.iter().map(|&s| s as u64).collect();
            if let Some(&seed) = candidates.first() {
                self.cracked_seed = Some(seed);
                self.session.cracked_seed = Some(seed);
                let _ = self.storage.save(&self.session);
                self.status_message = format!("⭐ Semilla/candidato encontrado: {seed}");
                self.add_toast(self.status_message.clone(), 0x00FF00);
            }
            crate::broadcast_seed_candidates(&candidates_u64);
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
        self.session.hashed_seed = Some(hashed_seed);
        let _ = self.storage.save(&self.session);

        self.add_toast(format!("🔑 Hashed Seed: 0x{:016X}", hashed_seed), 0x00FFFF);
        self.status_message = format!("Hashed Seed: 0x{:016X}", hashed_seed);
    }

    fn on_tick(&mut self) { self.poll_solver(); }

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
            for z in 0..16 {
                for x in 0..16 {
                    let idx = (z << 4) | x;
                    if mask[idx] != 0 {
                        count += 1;
                        let bx = (cx << 4) + x as i32;
                        let bz = (cz << 4) + z as i32;
                        self.state.bedrock_blocks.push(BedrockBlock::new(bx, base_y, bz, BlockType::Bedrock));
                    }
                }
            }

            if count > 0 {
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
                    log::info!("Comandos: /sc [gui | collect | solve | reset | esp | export | status]");
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
