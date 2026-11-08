// Copyright (c) 2026 VkYakee
// VkYakee VoxelMap x seedcrackerX
//! Settings button click handler for HorizonVoxelCracker.

pub(in crate::game) fn run(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use super::horizon_voxel_cracker as r;
    let _ = game;
    if mod_id != r::ID {
        return;
    }
    r::settings_click(button);
}
