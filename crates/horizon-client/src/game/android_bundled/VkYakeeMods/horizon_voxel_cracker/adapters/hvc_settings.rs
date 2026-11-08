// Copyright (c) 2026 VkYakee
// VkYakee VoxelMap x seedcrackerX
//! Settings button labels for HorizonVoxelCracker.

pub(in crate::game) fn run(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use super::horizon_voxel_cracker as r;
    let _ = game;
    if mod_id != r::ID {
        return;
    }
    buttons.extend(r::settings_labels());
}
