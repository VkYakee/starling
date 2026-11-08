// Copyright (c) 2026 HorizonMatica Authors / Starling Android
//! Pushes depth-tested 3D outlines (NativeBlockRim) for mismatches and placement bounds.

use crate::game::native_hooks::{NativeBlockRim, NativeColoredLight};
use super::horizonmatica as r;
use r::visuals::VisualRenderer;

pub(in crate::game) fn run(
    game: &crate::game::Game<'_>,
    rims: &mut Vec<NativeBlockRim>,
    lights: &mut Vec<NativeColoredLight>,
) {
    let _ = lights;
    let rt = r::get();
    let schem_guard = rt.active_schematic.lock().unwrap();
    let placement = rt.placement.lock().unwrap().clone();
    let verifier = rt.verifier.lock().unwrap().clone();

    let player_pos = if let Some(p) = game.player.as_ref() {
        [p.position.x, p.position.y, p.position.z]
    } else {
        [0.0, 64.0, 0.0]
    };

    VisualRenderer::collect_rims(
        rims,
        schem_guard.as_ref(),
        &placement,
        &verifier,
        player_pos,
    );
}
