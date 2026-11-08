// Copyright (c) 2026 HorizonMatica Authors / Starling Android
//! Settings button click handler for HorizonMatica.

pub(in crate::game) fn run(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    if mod_id != "horizonmatica" {
        return;
    }
    use super::horizonmatica as r;
    let rt = r::get();
    match button {
        0 => {
            let mut t = rt.touch.lock().unwrap();
            t.bar_open = !t.bar_open;
            rt.set_toast(format!("Action Bar: {}", if t.bar_open { "Open" } else { "Closed" }));
        }
        1 => {
            let mut v = rt.verifier.lock().unwrap();
            v.active = !v.active;
            rt.set_toast(format!("Verifier: {}", if v.active { "ON" } else { "OFF" }));
        }
        2 => {
            rt.rescan_files();
            rt.set_toast("Rescanned schematics folder");
        }
        3 => {
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
    let _ = game;
}
