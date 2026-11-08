// Copyright (c) 2026 HorizonMatica Authors / Starling Android
//! Registers HorizonMatica in the Options > Mods list.

pub(in crate::game) fn run(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    mods.push(crate::game::native_hooks::NativeMod {
        id: "horizonmatica".into(),
        title: "HorizonMatica 1.0.0".into(),
        description: "Litematica for Horizon Rust with Minecraft Bedrock Touch UI and .litematic / .schem support.".into(),
        keys: vec![
            ["toggle_bar".into(), "Toggle Action Bar".into(), "key.keyboard.l".into()],
            ["toggle_verifier".into(), "Toggle Verifier".into(), "key.keyboard.v".into()],
            ["cycle_rotation".into(), "Cycle Rotation".into(), "key.keyboard.r".into()],
            ["cycle_mirror".into(), "Cycle Mirror".into(), "key.keyboard.m".into()],
        ],
    });
}
