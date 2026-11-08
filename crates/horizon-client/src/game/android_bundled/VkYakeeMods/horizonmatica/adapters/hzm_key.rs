// Copyright (c) 2026 HorizonMatica Authors / Starling Android
//! Handles key shortcuts and Android touchscreen taps for HorizonMatica.

pub(in crate::game) fn run(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    if !pressed {
        return;
    }
    use super::horizonmatica as r;
    let rt = r::get();

    // 1. Android Touch Tap / Left Click
    if key == "key.mouse.left" {
        let cursor = game.native_cursor();
        let player_pos = if let Some(p) = game.player.as_ref() {
            [p.position.x, p.position.y, p.position.z]
        } else {
            [0.0, 64.0, 0.0]
        };

        if r::handle_screen_touch(cursor, player_pos) {
            return;
        }
    }

    // 2. Hardware Keyboard Shortcuts
    if key == "key.keyboard.l" {
        let mut t = rt.touch.lock().unwrap();
        t.bar_open = !t.bar_open;
        rt.set_toast(format!("Action Bar: {}", if t.bar_open { "Open" } else { "Closed" }));
    } else if key == "key.keyboard.v" {
        let mut v = rt.verifier.lock().unwrap();
        v.active = !v.active;
        rt.set_toast(format!("Verifier: {}", if v.active { "ON" } else { "OFF" }));
    } else if key == "key.keyboard.r" {
        let mut p = rt.placement.lock().unwrap();
        p.rotation.cycle();
        rt.set_toast(format!("Rotation: {}", p.rotation.display_name()));
    } else if key == "key.keyboard.m" {
        let mut p = rt.placement.lock().unwrap();
        p.mirror.cycle();
        rt.set_toast(format!("Mirror: {}", p.mirror.display_name()));
    } else if key == "key.keyboard.page_up" {
        let mut p = rt.placement.lock().unwrap();
        let max_h = {
            let s = rt.active_schematic.lock().unwrap();
            s.as_ref().map(|sc| sc.enclosing_size[1]).unwrap_or(64)
        };
        p.layer_mode.adjust_layer(1, max_h);
        rt.set_toast(format!("Layer: {}", p.layer_mode.display_name()));
    } else if key == "key.keyboard.page_down" {
        let mut p = rt.placement.lock().unwrap();
        let max_h = {
            let s = rt.active_schematic.lock().unwrap();
            s.as_ref().map(|sc| sc.enclosing_size[1]).unwrap_or(64)
        };
        p.layer_mode.adjust_layer(-1, max_h);
        rt.set_toast(format!("Layer: {}", p.layer_mode.display_name()));
    }
}
