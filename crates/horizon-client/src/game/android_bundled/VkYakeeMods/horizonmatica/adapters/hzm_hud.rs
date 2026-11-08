// Copyright (c) 2026 HorizonMatica Authors / Starling Android
//! Draws the HorizonMatica HUD with Bedrock touch controls, chip, and modals.

use super::horizonmatica as r;
use r::hud::HudRenderer;
use r::materials::MaterialReport;
use r::touch_ui::ModalKind;

pub(in crate::game) fn run(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    let _ = corners;
    let rt = r::get();
    let schem_guard = rt.active_schematic.lock().unwrap();
    let placement = rt.placement.lock().unwrap().clone();
    let verifier = rt.verifier.lock().unwrap().clone();
    let files_guard = rt.schematic_files.lock().unwrap();
    let toast_guard = rt.toast.lock().unwrap();
    let mut touch = rt.touch.lock().unwrap();

    // If material modal is open, ensure materials report is calculated
    let mut materials = rt.materials.lock().unwrap();
    if touch.active_modal == ModalKind::MaterialList && schem_guard.is_some() {
        if materials.is_none() {
            if let Some(sc) = schem_guard.as_ref() {
                let rep = MaterialReport::generate(sc, &placement, |item_name| {
                    let mut count = 0;
                    for slot in &game.inventory {
                        if let Some(stack) = slot {
                            if stack.item.as_ref() == item_name || stack.item.strip_prefix("minecraft:").unwrap_or(&stack.item) == item_name.strip_prefix("minecraft:").unwrap_or(item_name) {
                                count += stack.count as usize;
                            }
                        }
                    }
                    count
                });
                *materials = Some(rep);
            }
        }
    }

    let default_rep = MaterialReport::default();
    let active_mats = materials.as_ref().unwrap_or(&default_rep);

    HudRenderer::draw(
        batch,
        &game.gui.atlas,
        &mut touch,
        schem_guard.as_ref(),
        &placement,
        &verifier,
        active_mats,
        &files_guard,
        toast_guard.as_ref(),
    );
}
