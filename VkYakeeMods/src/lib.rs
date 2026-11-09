pub mod api;
pub mod mod_menu;
pub mod horizonmatica_adapter;
pub mod seedcracker_adapter;
pub mod storage;
pub mod verifier;
pub mod vkmap_adapter;
pub mod vlakee_ui_adapter;

pub use api::{ModMetadata, VkMod};
pub use mod_menu::{MenuAction, MenuPage, ModMenuState, ModuleStatus};
pub use horizonmatica_adapter::HorizonMaticaMod;
pub use seedcracker_adapter::SeedCrackerMod;
pub use storage::StorageManager;
pub use verifier::SeedVerifier;
pub use vkmap_adapter::VkMapMod;
pub use vlakee_ui_adapter::VlakeeUIMod;

use std::sync::{Mutex, OnceLock};
use std::collections::VecDeque;

#[derive(Clone)]
struct ChunkEvent { cx:i32, cz:i32, dimension:String, mask:Option<[u8;256]> }
static CHUNK_EVENTS: OnceLock<Mutex<VecDeque<ChunkEvent>>> = OnceLock::new();
fn chunk_events()->&'static Mutex<VecDeque<ChunkEvent>> { CHUNK_EVENTS.get_or_init(||Mutex::new(VecDeque::new())) }

// Queue solver publications while MOD_HUB is locked; dispatch after on_tick
// releases it to prevent SeedCracker -> VkMap recursive-lock deadlocks.
static SEED_PUBLICATIONS: OnceLock<Mutex<VecDeque<Vec<u64>>>> = OnceLock::new();
fn seed_publications()->&'static Mutex<VecDeque<Vec<u64>>> {
    SEED_PUBLICATIONS.get_or_init(|| Mutex::new(VecDeque::new()))
}
pub fn publish_seed_candidates(candidates: &[u64]) {
    if candidates.is_empty() { return; }
    let mut queue=seed_publications().lock().unwrap_or_else(|e|e.into_inner());
    if queue.len() >= 8 { queue.pop_front(); }
    queue.push_back(candidates.to_vec());
}
fn dispatch_seed_publications() {
    let pending:Vec<Vec<u64>> = {
        let mut queue=seed_publications().lock().unwrap_or_else(|e|e.into_inner());
        queue.drain(..).collect()
    };
    for candidates in pending { broadcast_seed_candidates(&candidates); }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudTab {
    All,
    HorizonMatica,
    SeedCracker,
    VlakeeUI,
    VkMap,
}

pub struct VkModHub {
    mods: Vec<Box<dyn VkMod>>,
    pub active_tab: HudTab,
    pub suppress_default_f3: bool,
    pub master_f3_active: bool,
}

impl VkModHub {
    pub fn new() -> Self {
        Self {
            mods: Vec::new(),
            active_tab: HudTab::All,
            suppress_default_f3: false,
            master_f3_active: true,
        }
    }

    pub fn register(&mut self, m: Box<dyn VkMod>) {
        log::info!("VkYakeeMods: Registered [{}] v{}", m.metadata().id, m.metadata().version);
        self.mods.push(m);
    }
}

static MOD_HUB: OnceLock<Mutex<VkModHub>> = OnceLock::new();

pub fn hub() -> &'static Mutex<VkModHub> {
    MOD_HUB.get_or_init(|| {
        let mut h = VkModHub::new();
        // Register default VkYakee mods: SeedCracker, HorizonMatica, VlakeeUI & VkMap
        h.register(Box::new(SeedCrackerMod::new()));
        h.register(Box::new(HorizonMaticaMod::new()));
        h.register(Box::new(VlakeeUIMod::new()));
        h.register(Box::new(VkMapMod::new()));
        Mutex::new(h)
    })
}

/// Checks if Starling's native default F3 screen is suppressed by the master killswitch
pub fn is_default_f3_suppressed() -> bool {
    if let Ok(h) = hub().lock() {
        h.suppress_default_f3
    } else {
        false
    }
}

/// Sets the suppression state of Starling's default F3 screen
pub fn set_default_f3_suppressed(val: bool) {
    if let Ok(mut h) = hub().lock() {
        h.suppress_default_f3 = val;
    }
}

/// Master toggle: turns OFF or ON both SmartF3 and Starling's default F3 screen
pub fn toggle_master_f3_killswitch() -> bool {
    if let Ok(mut h) = hub().lock() {
        h.master_f3_active = !h.master_f3_active;
        h.suppress_default_f3 = !h.master_f3_active;
        let active = h.master_f3_active;
        let cmd = if active { "vui master_f3_on" } else { "vui master_f3_off" };
        for m in &mut h.mods {
            m.on_command(cmd);
        }
        active
    } else {
        true
    }
}

// Global dispatchers called from Starling hooks

pub fn on_remote_seed(hashed_seed: u64) {
    if let Ok(mut h) = hub().lock() {
        for m in &mut h.mods {
            m.on_hashed_seed(hashed_seed);
        }
    }
}

/// Broadcasts candidate or cracked seeds (from SeedCracker) to other mods such as VkMap for predictive mapping.
pub fn broadcast_seed_candidates(candidates: &[u64]) {
    if let Ok(mut h) = hub().lock() {
        if let Some(&first) = candidates.first() {
            let cmd = format!("vkmap sync_seed {}", first);
            for m in &mut h.mods {
                m.on_command(&cmd);
            }
        }
    }
}

pub fn on_tick() {
    {
        if let Ok(mut h) = hub().lock() {
            for m in &mut h.mods { m.on_tick(); }
        }
    } // MOD_HUB released before cross-mod publications
    dispatch_seed_publications();
}

pub fn on_command(command: &str, ran: &mut bool) {
    if *ran {
        return;
    }

    let parts: Vec<&str> = command.split_whitespace().collect();
    if !parts.is_empty() && (parts[0].eq_ignore_ascii_case("vk") || parts[0].eq_ignore_ascii_case("vkyakee")) {
        if parts.len() >= 2 && matches!(parts[1].to_ascii_lowercase().as_str(), "f3off" | "f3mute" | "f3master" | "silence") {
            toggle_master_f3_killswitch();
            *ran = true;
            return;
        }

        if let Ok(mut h) = hub().lock() {
            if parts.len() >= 3 && parts[1].eq_ignore_ascii_case("tab") {
                match parts[2].to_ascii_lowercase().as_str() {
                    "hm" | "horizonmatica" | "schem" => h.active_tab = HudTab::HorizonMatica,
                    "sc" | "seedcracker" | "seed" => h.active_tab = HudTab::SeedCracker,
                    "vui" | "vlakee" | "vlakeeui" | "ui" => h.active_tab = HudTab::VlakeeUI,
                    "map" | "vkmap" => h.active_tab = HudTab::VkMap,
                    _ => h.active_tab = HudTab::All,
                }
                *ran = true;
                return;
            }
        }
    }

    if let Ok(mut h) = hub().lock() {
        for m in &mut h.mods {
            if m.on_command(command) {
                *ran = true;
                break;
            }
        }
    }
}

pub fn on_chunk_loaded(cx: i32, cz: i32, dimension: &str, bedrock_mask: Option<&[u8; 256]>) {
    if let Ok(mut h) = hub().lock() {
        for m in &mut h.mods {
            m.on_chunk_loaded(cx, cz, dimension, bedrock_mask);
        }
    }
}

pub fn on_block_snapshot(cx:i32,cz:i32,dimension:&str,blocks:&[(i32,i32,i32,String)]) {
    if let Ok(mut h)=hub().lock(){for m in &mut h.mods{m.on_block_snapshot(cx,cz,dimension,blocks);}}
}
pub fn on_block_update(x:i32,y:i32,z:i32,block:Option<&str>) {
    if let Ok(mut h)=hub().lock(){for m in &mut h.mods{m.on_block_update(x,y,z,block);}}
}

pub fn on_biome_found(x: i32, z: i32, biome_id: i32, name: &str) {
    if let Ok(mut h) = hub().lock() {
        for m in &mut h.mods {
            m.on_biome_found(x, z, biome_id, name);
        }
    }
}

pub fn on_player_pos(x: i32, y: i32, z: i32) {
    if let Ok(mut h) = hub().lock() {
        for m in &mut h.mods {
            m.on_player_pos(x, y, z);
        }
    }
}

pub fn on_structure_found(st_type: &str, chunk_x: i32, chunk_z: i32) {
    if let Ok(mut h) = hub().lock() {
        for m in &mut h.mods {
            m.on_structure_found(st_type, chunk_x, chunk_z);
        }
    }
}

pub fn get_hud_lines() -> Vec<String> {
    let mut lines = Vec::new();
    if let Ok(h) = hub().lock() {
        for m in &h.mods {
            let id = m.metadata().id;
            let show = match h.active_tab {
                HudTab::All => true,
                HudTab::HorizonMatica => id == "horizonmatica",
                HudTab::SeedCracker => id == "seedcracker",
                HudTab::VlakeeUI => id == "vlakee_ui",
                HudTab::VkMap => id == "vkmap",
            };
            if show {
                lines.extend(m.hud_lines());
            }
        }
    }
    lines
}

pub fn get_bounding_boxes() -> Vec<[f32; 10]> {
    let mut boxes = Vec::new();
    if let Ok(h) = hub().lock() {
        for m in &h.mods {
            boxes.extend(m.render_bounding_boxes());
        }
    }
    boxes
}

pub fn get_mod_list() -> Vec<ModMetadata> {
    if let Ok(h) = hub().lock() {
        h.mods.iter().map(|m| m.metadata()).collect()
    } else {
        Vec::new()
    }
}

pub fn get_settings_buttons(mod_id: &str) -> Vec<String> {
    if let Ok(h) = hub().lock() {
        for m in &h.mods {
            if m.metadata().id == mod_id {
                return m.settings_buttons();
            }
        }
    }
    Vec::new()
}

pub fn on_settings_click(mod_id: &str, button_idx: usize) {
    if let Ok(mut h) = hub().lock() {
        for m in &mut h.mods {
            if m.metadata().id == mod_id {
                m.on_settings_click(button_idx);
                break;
            }
        }
    }
}
