// Copyright (c) 2026 VkYakee
// VkYakee VoxelMap x seedcrackerX
//! Ticks the HorizonVoxelCracker logic with MC version inference and chunk scanning.

use super::horizon_voxel_cracker::cracker::MCVersion;
use super::horizon_voxel_cracker as r;
use std::sync::atomic::{AtomicU32, Ordering};

static TICK_COUNTER: AtomicU32 = AtomicU32::new(0);
static WAS_ALIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

pub(in crate::game) fn run(game: &mut crate::game::Game<'_>) {
    let rt = r::get();
    let tick = TICK_COUNTER.fetch_add(1, Ordering::Relaxed);

    let Some(player) = game.player.as_ref() else {
        return;
    };
    let px = player.position.x;
    let py = player.position.y;
    let pz = player.position.z;
    let dim = game.dim.id();

    // 1. Detect death to place automatic death waypoint
    let is_dead = player.health.health() <= 0.0;
    let was_alive = WAS_ALIVE.swap(!is_dead, Ordering::SeqCst);
    if is_dead && was_alive {
        let death_pos = [px.round() as i32, py.round() as i32, pz.round() as i32];
        rt.waypoints.on_death(death_pos, dim);
        game.system_message(horizon_client::text::Text::literal(format!(
            "☠ [HVC] Death waypoint saved at [{}, {}, {}]",
            death_pos[0], death_pos[1], death_pos[2]
        )));
    }

    // 2. Automatically capture the obfuscated/hashed seed from the server's SpawnInfo
    let hashed = game.streaming.biome_zoom.seed();
    if hashed != 0 {
        rt.cracker.set_hashed_seed(hashed);
    }

    // 3. Periodically scan nearby chunks (every 20 ticks = 1 second)
    if tick % 20 == 0 {
        let center_cx = (px.floor() as i32) >> 4;
        let center_cz = (pz.floor() as i32) >> 4;

        // Auto-detect Minecraft version from depth or world bounds if in Auto mode
        {
            let mut ver = rt.cracker.version.lock().unwrap();
            if *ver == MCVersion::Auto && (py < 0.0 || game.world.bottom_y() < 0) {
                *ver = MCVersion::V1_18_Plus;
            }
        }

        let facts = game.res.facts();

        for dx in -2..=2 {
            for dz in -2..=2 {
                let cx = center_cx + dx;
                let cz = center_cz + dz;
                let block_x = cx * 16 + 8;
                let block_z = cz * 16 + 8;

                let biome_name = game
                    .world
                    .chunks
                    .get(&(cx, cz))
                    .and_then(|c| {
                        let b_idx = c.biome(block_x, 64, block_z) as usize;
                        game.res.biomes().list.get(b_idx).map(|b| b.name.as_str())
                    });

                rt.finder.scan_chunk(
                    cx,
                    cz,
                    dim,
                    biome_name,
                    |x, y, z| {
                        let state = game.world.block(x, y, z);
                        if facts.is_air(state) {
                            return None;
                        }
                        Some(facts.get(state).name.to_string())
                    },
                    &rt.cracker,
                    &rt.waypoints,
                );
            }
        }
    }
}
