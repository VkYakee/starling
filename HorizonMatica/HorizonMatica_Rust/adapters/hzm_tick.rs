// Copyright (c) 2026 HorizonMatica Authors / Starling Android
//! Periodically verifies world blocks vs schematic blocks.

use super::horizonmatica as r;
use std::sync::atomic::{AtomicU32, Ordering};

static TICK_COUNTER: AtomicU32 = AtomicU32::new(0);

pub(in crate::game) fn run(game: &mut crate::game::Game<'_>) {
    let tick = TICK_COUNTER.fetch_add(1, Ordering::Relaxed);

    // Verify every 10 ticks (0.5s) if schematic is active and verifier is enabled
    if tick % 10 == 0 {
        let rt = r::get();
        let schem_guard = rt.active_schematic.lock().unwrap();
        let Some(schem) = schem_guard.as_ref() else {
            return;
        };

        let placement = rt.placement.lock().unwrap().clone();
        if !placement.enabled {
            return;
        }

        let player_pos = if let Some(p) = game.player.as_ref() {
            [p.position.x, p.position.y, p.position.z]
        } else {
            [0.0, 64.0, 0.0]
        };

        let facts = game.res.facts();

        let mut v = rt.verifier.lock().unwrap();
        if v.active {
            v.verify_area(
                schem,
                &placement,
                player_pos,
                48, // 48-block radius around player
                |x, y, z| {
                    let cx = x >> 4;
                    let cz = z >> 4;
                    if !game.world.chunks.contains_key(&(cx, cz)) {
                        return None;
                    }
                    let state = game.world.block(x, y, z);
                    if facts.is_air(state) {
                        return Some("minecraft:air".to_string());
                    }
                    Some(facts.get(state).name.to_string())
                },
            );
        }
    }
}
