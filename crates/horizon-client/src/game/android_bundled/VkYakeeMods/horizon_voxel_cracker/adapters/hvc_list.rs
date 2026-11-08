// Copyright (c) 2026 VkYakee
// VkYakee VoxelMap x seedcrackerX
// SPDX-License-Identifier: MIT
//! Registers HorizonVoxelCracker in the Options > Mods list.

pub(in crate::game) fn run(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use super::horizon_voxel_cracker as r;
    mods.push(crate::game::native_hooks::NativeMod {
        id: r::ID.into(),
        title: format!("{} {}", r::NAME, r::VERSION),
        description: r::DESCRIPTION.into(),
        keys: vec![
            ["toggle_minimap".into(), "Toggle Minimap".into(), "key.keyboard.m".into()],
            ["add_waypoint".into(), "Quick Waypoint".into(), "key.keyboard.b".into()],
            ["toggle_cave".into(), "Toggle Cave Mode".into(), "key.keyboard.c".into()],
        ],
    });
}
