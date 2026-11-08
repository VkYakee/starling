// Copyright (c) 2026 VkYakee
// VkYakee VoxelMap x seedcrackerX
//! Handles key shortcuts and Android touchscreen taps for HorizonVoxelCracker.

pub(in crate::game) fn run(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    if !pressed {
        return;
    }
    use super::horizon_voxel_cracker as r;
    let rt = r::get();

    let world_id = if let Some(integrated) = game.integrated.as_ref() {
        if !integrated.world_name.is_empty() {
            format!("local_{}", integrated.world_name)
        } else {
            "local_world".into()
        }
    } else if let Some(addr) = game.server_address() {
        format!("server_{}", addr)
    } else if let Some(remote) = game.remote.as_ref() {
        if !remote.session.address.is_empty() {
            format!("server_{}", remote.session.address)
        } else {
            format!("server_{}", remote.brand)
        }
    } else {
        "default_world".into()
    };

    // 1. Check for Android Touch Tap or Left Click
    if key == "key.mouse.left" {
        let cursor = game.native_cursor();
        let player_pos = if let Some(p) = game.player.as_ref() {
            [p.position.x, p.position.y, p.position.z]
        } else {
            [0.0, 64.0, 0.0]
        };
        let dim = game.dim.id();

        if r::handle_screen_touch(cursor, player_pos, dim, &world_id) {
            return;
        }
    }

    // 2. Hardware Keyboard Shortcuts
    if key == "key.keyboard.m" {
        let mut s = rt.settings.lock().unwrap();
        s.show_minimap = !s.show_minimap;
        game.chat.push(horizon_client::text::Text::literal(format!(
            "[HVC] Minimap toggled {}",
            if s.show_minimap { "ON" } else { "OFF" }
        )));
    } else if key == "key.keyboard.c" {
        let mut s = rt.settings.lock().unwrap();
        s.cave_mode = !s.cave_mode;
        game.chat.push(horizon_client::text::Text::literal(format!(
            "[HVC] Cave Mode toggled {}",
            if s.cave_mode { "ON" } else { "OFF" }
        )));
    } else if key == "key.keyboard.b" {
        if let Some(p) = game.player.as_ref() {
            let pos = [p.position.x.round() as i32, p.position.y.round() as i32, p.position.z.round() as i32];
            let dim = game.dim.id();
            let wp = r::waypoints::Waypoint::new("Quick Point", pos, dim, 0xFF55_FFFF);
            rt.waypoints.add(wp);
            game.chat.push(horizon_client::text::Text::literal(format!(
                "[HVC] Saved quick waypoint at [{}, {}, {}]",
                pos[0], pos[1], pos[2]
            )));
        }
    }
}
