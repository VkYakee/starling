// Copyright (c) 2026 VkYakee
// VkYakee VoxelMap x seedcrackerX
//! Intercepts `/hvc` commands in game chat.

pub(in crate::game) fn run(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    if *ran {
        return;
    }
    use super::horizon_voxel_cracker as r;
    let rt = r::get();
    let player_pos = if let Some(p) = game.player.as_ref() {
        [p.position.x, p.position.y, p.position.z]
    } else {
        [0.0, 64.0, 0.0]
    };
    let dim = game.dim.id();

    let handled = r::commands::CommandDispatcher::handle_command(
        command,
        player_pos,
        dim,
        &rt.settings,
        &rt.waypoints,
        &rt.cracker,
        |msg| {
            // Send feedback to client chat
            game.chat.push(horizon_client::text::Text::literal(msg));
        },
    );

    if handled {
        *ran = true;
    }
}
