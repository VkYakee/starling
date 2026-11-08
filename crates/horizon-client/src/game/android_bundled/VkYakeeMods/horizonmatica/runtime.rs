// Copyright (c) 2026 HorizonMatica Authors / Starling Android
//! Global runtime state management for HorizonMatica on Android.

pub mod commands;
pub mod container;
pub mod format;
pub mod hud;
pub mod materials;
pub mod placement;
pub mod storage;
pub mod touch_ui;
pub mod verifier;
pub mod visuals;

use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use format::LoadedSchematic;
use materials::MaterialReport;
use placement::{LayerMode, SchematicPlacement};
use storage::SchematicFileInfo;
use touch_ui::{ModalKind, TouchAction, TouchUiState};
use verifier::SchematicVerifier;

pub struct HorizonMaticaRuntime {
    pub active_schematic: Mutex<Option<LoadedSchematic>>,
    pub placement: Mutex<SchematicPlacement>,
    pub verifier: Mutex<SchematicVerifier>,
    pub materials: Mutex<Option<MaterialReport>>,
    pub touch: Mutex<TouchUiState>,
    pub schematic_files: Mutex<Vec<SchematicFileInfo>>,
    pub toast: Mutex<Option<(String, Instant)>>,
}

static RUNTIME: OnceLock<Arc<HorizonMaticaRuntime>> = OnceLock::new();

pub fn get() -> &'static Arc<HorizonMaticaRuntime> {
    RUNTIME.get_or_init(|| {
        let _ = storage::ensure_schematics_dir();
        let files = storage::list_schematics();

        Arc::new(HorizonMaticaRuntime {
            active_schematic: Mutex::new(None),
            placement: Mutex::new(SchematicPlacement::default()),
            verifier: Mutex::new(SchematicVerifier::new()),
            materials: Mutex::new(None),
            touch: Mutex::new(TouchUiState::new()),
            schematic_files: Mutex::new(files),
            toast: Mutex::new(None),
        })
    })
}

pub fn reset() {
    let rt = get();
    {
        let mut s = rt.active_schematic.lock().unwrap();
        *s = None;
    }
    {
        let mut p = rt.placement.lock().unwrap();
        *p = SchematicPlacement::default();
    }
    {
        let mut v = rt.verifier.lock().unwrap();
        v.clear();
    }
    {
        let mut m = rt.materials.lock().unwrap();
        *m = None;
    }
    {
        let mut t = rt.touch.lock().unwrap();
        t.close_modal();
        t.bar_open = false;
    }
    {
        let mut f = rt.schematic_files.lock().unwrap();
        *f = storage::list_schematics();
    }
}

impl HorizonMaticaRuntime {
    pub fn set_toast(&self, msg: impl Into<String>) {
        let mut t = self.toast.lock().unwrap();
        *t = Some((msg.into(), Instant::now()));
    }

    pub fn rescan_files(&self) {
        let files = storage::list_schematics();
        let mut f = self.schematic_files.lock().unwrap();
        *f = files;
    }

    pub fn load_schematic_by_filename(&self, filename: &str, player_pos: [f64; 3]) -> Result<String, String> {
        let (path, bytes) = storage::load_schematic_file(filename)
            .map_err(|e| format!("Failed to read file: {e}"))?;
        let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or(filename);
        let schem = format::load_from_bytes(&bytes, fname)?;

        let count = schem.total_non_air_blocks();
        let name = schem.name.clone();

        let origin = [
            player_pos[0].floor() as i32,
            player_pos[1].floor() as i32,
            player_pos[2].floor() as i32,
        ];

        {
            let mut s = self.active_schematic.lock().unwrap();
            *s = Some(schem);
        }
        {
            let mut p = self.placement.lock().unwrap();
            p.origin = origin;
            p.enabled = true;
        }
        {
            let mut v = self.verifier.lock().unwrap();
            v.clear();
        }

        self.set_toast(format!("Loaded '{}'", fname));
        Ok(format!("Loaded '{name}' ({count} blocks) at [{}, {}, {}]", origin[0], origin[1], origin[2]))
    }
}

/// Handles Android touchscreen tap on Bedrock UI buttons and modal dialogs.
pub fn handle_screen_touch(cursor: [f32; 2], player_pos: [f64; 3]) -> bool {
    let rt = get();
    let mut touch = rt.touch.lock().unwrap();

    // 1. If modal is open, check modal buttons
    if touch.active_modal != ModalKind::None {
        for btn in &touch.modal_buttons {
            if btn.contains(cursor) {
                let action = btn.action.clone();
                match action {
                    TouchAction::CloseModal => {
                        touch.close_modal();
                    }
                    TouchAction::LoadFile(fname) => {
                        let _ = rt.load_schematic_by_filename(&fname, player_pos);
                        touch.close_modal();
                    }
                    TouchAction::ScrollUp => {
                        touch.scroll_offset = (touch.scroll_offset - 3.0).max(0.0);
                    }
                    TouchAction::ScrollDown => {
                        touch.scroll_offset += 3.0;
                    }
                    TouchAction::RescanFiles => {
                        rt.rescan_files();
                    }
                    _ => {}
                }
                return true;
            }
        }
        // Touch on modal backdrop absorbs the click
        return true;
    }

    // 2. Main bar buttons & Pill button
    for btn in &touch.buttons {
        if btn.contains(cursor) {
            let action = btn.action.clone();
            match action {
                TouchAction::ToggleBar => {
                    touch.bar_open = !touch.bar_open;
                }
                TouchAction::OpenFileBrowser => {
                    rt.rescan_files();
                    touch.active_modal = ModalKind::FileBrowser;
                    touch.scroll_offset = 0.0;
                }
                TouchAction::OpenMaterialList => {
                    touch.active_modal = ModalKind::MaterialList;
                    touch.scroll_offset = 0.0;
                }
                TouchAction::ToggleLayer => {
                    let mut p = rt.placement.lock().unwrap();
                    let max_h = {
                        let s = rt.active_schematic.lock().unwrap();
                        s.as_ref().map(|sc| sc.enclosing_size[1]).unwrap_or(64)
                    };
                    p.layer_mode.cycle(0, max_h);
                    rt.set_toast(format!("Layer: {}", p.layer_mode.display_name()));
                }
                TouchAction::LayerUp => {
                    let mut p = rt.placement.lock().unwrap();
                    let max_h = {
                        let s = rt.active_schematic.lock().unwrap();
                        s.as_ref().map(|sc| sc.enclosing_size[1]).unwrap_or(64)
                    };
                    p.layer_mode.adjust_layer(1, max_h);
                    rt.set_toast(format!("Layer: {}", p.layer_mode.display_name()));
                }
                TouchAction::LayerDown => {
                    let mut p = rt.placement.lock().unwrap();
                    let max_h = {
                        let s = rt.active_schematic.lock().unwrap();
                        s.as_ref().map(|sc| sc.enclosing_size[1]).unwrap_or(64)
                    };
                    p.layer_mode.adjust_layer(-1, max_h);
                    rt.set_toast(format!("Layer: {}", p.layer_mode.display_name()));
                }
                TouchAction::CycleRotation => {
                    let mut p = rt.placement.lock().unwrap();
                    p.rotation.cycle();
                    rt.set_toast(format!("Rotation: {}", p.rotation.display_name()));
                }
                TouchAction::CycleMirror => {
                    let mut p = rt.placement.lock().unwrap();
                    p.mirror.cycle();
                    rt.set_toast(format!("Mirror: {}", p.mirror.display_name()));
                }
                TouchAction::ToggleVerifier => {
                    let mut v = rt.verifier.lock().unwrap();
                    v.active = !v.active;
                    rt.set_toast(format!("Verifier: {}", if v.active { "ON" } else { "OFF" }));
                }
                TouchAction::SetPositionHere => {
                    let mut p = rt.placement.lock().unwrap();
                    p.origin = [
                        player_pos[0].floor() as i32,
                        player_pos[1].floor() as i32,
                        player_pos[2].floor() as i32,
                    ];
                    rt.set_toast(format!("Origin: [{}, {}, {}]", p.origin[0], p.origin[1], p.origin[2]));
                }
                TouchAction::UnloadSchematic => {
                    let mut s = rt.active_schematic.lock().unwrap();
                    *s = None;
                    let mut v = rt.verifier.lock().unwrap();
                    v.clear();
                    let mut m = rt.materials.lock().unwrap();
                    *m = None;
                    rt.set_toast("Schematic unloaded");
                }
                _ => {}
            }
            return true;
        }
    }

    false
}
