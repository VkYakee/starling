// Copyright (c) 2026 VkYakee
// VkYakee VoxelMap x seedcrackerX
//! Draws the HorizonVoxelCracker HUD with Bedrock touch controls and isolated world storage.

use super::horizon_voxel_cracker as r;
use horizon_client::extras::Corner;
use r::hud::HudRenderer;
use r::radar::{RadarBlip, classify_entity_type};

pub(in crate::game) fn run(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut crate::game::extras_hud::Corners,
) {
    let rt = r::get();
    let settings = *rt.settings.lock().unwrap();
    let mut touch = rt.touch.lock().unwrap();

    let Some(player) = game.player.as_ref() else {
        return;
    };
    let player_pos = [player.position.x, player.position.y, player.position.z];
    let player_yaw = game.yaw;
    let dimension = game.dim.id();

    // Determine current world ID (so worlds/servers are never mixed)
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

    r::ensure_world(&world_id);

    // 1. Resolve current biome name
    let feet_x = player_pos[0].floor() as i32;
    let feet_y = player_pos[1].floor() as i32;
    let feet_z = player_pos[2].floor() as i32;
    let biome_name = game
        .world
        .chunks
        .get(&(feet_x >> 4, feet_z >> 4))
        .and_then(|c| {
            let biome_idx = c.biome(feet_x, feet_y, feet_z) as usize;
            game.res.biomes().list.get(biome_idx).map(|b| b.name.as_str())
        });

    // 2. Gather radar blips from loaded mobs
    let mut radar_blips = Vec::new();
    if settings.show_radar {
        for mob in &game.mobs {
            let cat = classify_entity_type(&mob.kind.id);
            let blip = RadarBlip::new(
                [mob.position.x, mob.position.y, mob.position.z],
                player_pos[1],
                cat,
                &mob.kind.id,
            );
            radar_blips.push(blip);
        }
    }

    // 3. Compute corner placement avoiding status effects
    let size = settings.minimap_size;
    let corner_pos = corners.place(Corner::TopRight, size, size);

    // 4. Render HUD overlays and Bedrock Touch controls
    HudRenderer::draw(
        batch,
        &settings,
        &mut touch,
        player_pos,
        player_yaw,
        biome_name,
        dimension,
        &world_id,
        &rt.waypoints,
        &radar_blips,
        &rt.cracker,
        &game.gui.atlas,
        corner_pos,
    );
}
