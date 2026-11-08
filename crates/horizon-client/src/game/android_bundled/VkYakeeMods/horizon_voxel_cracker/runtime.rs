// Copyright (c) 2026 VkYakee
// VkYakee VoxelMap x seedcrackerX
// SPDX-License-Identifier: MIT
//! Runtime core for HorizonVoxelCracker mod in Horizon Rust with Android storage and MCVersion support.

pub mod commands;
pub mod cracker;
pub mod finder;
pub mod hud;
pub mod radar;
pub mod storage;
pub mod touch_ui;
pub mod waypoints;

use cracker::{CrackerEngine, MCVersion};
use finder::StructureFinder;
use hud::HudSettings;
use touch_ui::{TouchAction, TouchUiState};
use waypoints::WaypointManager;

use std::sync::{Arc, Mutex, OnceLock};

pub const ID: &str = "horizon_voxel_cracker";
pub const NAME: &str = "HorizonVoxelCracker";
pub const VERSION: &str = "1.0.0";
pub const AUTHORS: &str = "VkYakee";
pub const DESCRIPTION: &str = "VkYakee VoxelMap x seedcrackerX";

pub struct Runtime {
    pub cracker: Arc<CrackerEngine>,
    pub waypoints: WaypointManager,
    pub finder: StructureFinder,
    pub settings: Mutex<HudSettings>,
    pub touch: Mutex<TouchUiState>,
}

impl Runtime {
    pub fn new() -> Self {
        let (config, map_mode) = storage::load_global_config();
        let mut touch_state = TouchUiState::new();
        touch_state.map_mode = map_mode;

        Self {
            cracker: Arc::new(CrackerEngine::new()),
            waypoints: WaypointManager::new(),
            finder: StructureFinder::new(),
            settings: Mutex::new(config),
            touch: Mutex::new(touch_state),
        }
    }
}

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn get() -> &'static Runtime {
    RUNTIME.get_or_init(Runtime::new)
}

/// Ensure current world context is active
pub fn ensure_world(world_id: &str) {
    let rt = get();
    rt.cracker.switch_world(world_id);
    rt.waypoints.switch_world(world_id);
}

/// Handle a touch event on the screen at GUI coordinates `cursor`
pub fn handle_screen_touch(cursor: [f32; 2], player_pos: [f64; 3], dim: &str, world_id: &str) -> bool {
    let rt = get();
    ensure_world(world_id);
    let mut touch = rt.touch.lock().unwrap();

    // 1. If modal is open, check modal buttons first
    if touch.modal_open {
        let mut action_opt = None;
        for btn in &touch.modal_buttons {
            if btn.contains(cursor) {
                action_opt = Some(btn.action);
                break;
            }
        }
        if let Some(action) = action_opt {
            match action {
                TouchAction::CloseCrackerModal => {
                    touch.modal_open = false;
                    touch.set_feedback("Cerrado");
                }
                TouchAction::ForceCrack => {
                    touch.set_feedback("Iniciando busqueda de semilla...");
                    rt.cracker.start_cracking();
                }
                TouchAction::CycleVersion => {
                    let mut ver = rt.cracker.version.lock().unwrap();
                    *ver = ver.cycle();
                    touch.set_feedback(format!("Version: {}", ver.name()));
                }
                _ => {}
            }
            return true;
        }
    }

    // 2. Check regular touch buttons (Pill button and quick action bar)
    let mut action_opt = None;
    for btn in &touch.buttons {
        if btn.contains(cursor) {
            action_opt = Some(btn.action);
            break;
        }
    }

    let Some(action) = action_opt else {
        return false;
    };

    match action {
        TouchAction::ToggleBar => {
            touch.bar_open = !touch.bar_open;
        }
        TouchAction::CycleMapMode => {
            touch.map_mode = (touch.map_mode + 1) % 3;
            let label = match touch.map_mode {
                0 => "Mapa Normal",
                1 => "Mapa Grande Bedrock",
                _ => "Mapa Oculto",
            };
            touch.set_feedback(label);
            let s = rt.settings.lock().unwrap();
            storage::save_global_config(&*s, touch.map_mode);
        }
        TouchAction::ToggleCave => {
            let mut s = rt.settings.lock().unwrap();
            s.cave_mode = !s.cave_mode;
            touch.set_feedback(if s.cave_mode { "Modo Cueva: ON" } else { "Modo Cueva: OFF" });
            storage::save_global_config(&*s, touch.map_mode);
        }
        TouchAction::QuickWaypoint => {
            let block_pos = [
                player_pos[0].round() as i32,
                player_pos[1].round() as i32,
                player_pos[2].round() as i32,
            ];
            let wp = waypoints::Waypoint::new("Touch Point", block_pos, dim, 0xFF55_FFFF);
            rt.waypoints.add(wp);
            touch.set_feedback(format!("Waypoint guardado en [{}, {}, {}]", block_pos[0], block_pos[1], block_pos[2]));
        }
        TouchAction::ToggleRadar => {
            let mut s = rt.settings.lock().unwrap();
            s.show_radar = !s.show_radar;
            touch.set_feedback(if s.show_radar { "Radar: ON" } else { "Radar: OFF" });
            storage::save_global_config(&*s, touch.map_mode);
        }
        TouchAction::OpenCrackerModal => {
            touch.modal_open = true;
        }
        TouchAction::CloseCrackerModal => {
            touch.modal_open = false;
        }
        TouchAction::ForceCrack => {
            touch.set_feedback("Iniciando cracking de semilla...");
            rt.cracker.start_cracking();
        }
        TouchAction::CycleVersion => {
            let mut ver = rt.cracker.version.lock().unwrap();
            *ver = ver.cycle();
            touch.set_feedback(format!("Version: {}", ver.name()));
        }
        TouchAction::ResetData => {
            rt.cracker.delete_world_data();
            rt.finder.clear();
            touch.set_feedback("Datos del Cracker eliminados para este mundo");
        }
    }

    true
}

pub fn settings_labels() -> Vec<String> {
    let rt = get();
    let s = rt.settings.lock().unwrap();
    let touch = rt.touch.lock().unwrap();
    let ver = *rt.cracker.version.lock().unwrap();
    vec![
        format!("Minimap: {}", if s.show_minimap { "ON" } else { "OFF" }),
        format!("Cave Mode: {}", if s.cave_mode { "ON" } else { "OFF" }),
        format!("Radar: {}", if s.show_radar { "ON" } else { "OFF" }),
        format!("Waypoints: {}", if s.show_waypoints { "ON" } else { "OFF" }),
        format!("Coordinates: {}", if s.show_coords { "ON" } else { "OFF" }),
        format!("Cracker Status: {}", if s.show_cracker_status { "ON" } else { "OFF" }),
        format!("Touch Bar: {}", if touch.bar_open { "Open" } else { "Closed" }),
        format!("Version: {}", ver.name()),
        "Start Cracking Job".into(),
        "Delete World Cracker Data".into(),
    ]
}

pub fn settings_click(button: usize) {
    let rt = get();
    let mut s = rt.settings.lock().unwrap();
    let mut touch = rt.touch.lock().unwrap();
    match button {
        0 => s.show_minimap = !s.show_minimap,
        1 => s.cave_mode = !s.cave_mode,
        2 => s.show_radar = !s.show_radar,
        3 => s.show_waypoints = !s.show_waypoints,
        4 => s.show_coords = !s.show_coords,
        5 => s.show_cracker_status = !s.show_cracker_status,
        6 => touch.bar_open = !touch.bar_open,
        7 => {
            let mut ver = rt.cracker.version.lock().unwrap();
            *ver = ver.cycle();
        }
        8 => {
            drop(s);
            drop(touch);
            rt.cracker.start_cracking();
            return;
        }
        9 => {
            drop(s);
            drop(touch);
            rt.cracker.delete_world_data();
            rt.finder.clear();
            return;
        }
        _ => {}
    }
    storage::save_global_config(&*s, touch.map_mode);
}

pub fn reset() {
    let rt = get();
    rt.cracker.reset_memory();
    rt.finder.clear();
    rt.waypoints.reset_memory();
}
