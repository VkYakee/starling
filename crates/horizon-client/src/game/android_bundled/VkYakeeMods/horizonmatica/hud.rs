// Copyright (c) 2026 HorizonMatica Authors / Starling Android
//! Bedrock Edition touchscreen HUD rendering pipeline for HorizonMatica.
//! Renders top pill toggle, quick action bar, real-time verifier chip,
//! file browser modal, material list modal, and toast alerts.

use std::time::Instant;
use horizon_client::{
    hud::{HudAtlas, HudBatch, rgba},
    text::Text,
};

use super::format::LoadedSchematic;
use super::materials::MaterialReport;
use super::placement::{LayerMode, SchematicPlacement};
use super::storage::{self, SchematicFileInfo};
use super::touch_ui::{ModalKind, TouchAction, TouchButton, TouchUiState};
use super::verifier::SchematicVerifier;

pub struct HudRenderer;

impl HudRenderer {
    pub fn draw(
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        touch: &mut TouchUiState,
        schematic: Option<&LoadedSchematic>,
        placement: &SchematicPlacement,
        verifier: &SchematicVerifier,
        materials: &MaterialReport,
        schematic_files: &[SchematicFileInfo],
        toast: Option<&(String, Instant)>,
    ) {
        touch.buttons.clear();
        touch.modal_buttons.clear();

        let sw = batch.width;
        let sh = batch.height;

        // =========================================================================
        // 1. Bedrock Floating Toggle Pill [▲ HZ-MAT] / [▼ HZ-MAT]
        // =========================================================================
        let pill_w = 64.0f32;
        let pill_h = 16.0f32;
        let pill_x = (sw * 0.5) - (pill_w * 0.5);
        let pill_y = 3.0f32;

        let pill_bg = if touch.bar_open {
            rgba(20, 110, 210, 230)
        } else {
            rgba(32, 36, 42, 210)
        };
        batch.fill(atlas, pill_x, pill_y, pill_w, pill_h, pill_bg);
        batch.fill(atlas, pill_x, pill_y, pill_w, 1.0, rgba(140, 170, 210, 220));
        batch.fill(atlas, pill_x, pill_y + pill_h - 1.0, pill_w, 1.0, rgba(40, 50, 60, 220));

        let pill_text = if touch.bar_open { "▲ HZ-MAT" } else { "▼ HZ-MAT" };
        let tw = HudBatch::text_width(atlas, pill_text);
        batch.text_styled(
            atlas,
            &Text::literal(pill_text),
            pill_x + (pill_w - tw) * 0.5,
            pill_y + 4.0,
            rgba(255, 255, 255, 255),
            true,
        );

        touch.buttons.push(TouchButton {
            id: "btn_pill",
            rect: [pill_x, pill_y, pill_w, pill_h],
            label: "HZ-MAT Toggle".into(),
            active: touch.bar_open,
            action: TouchAction::ToggleBar,
        });

        // =========================================================================
        // 2. Bedrock Touch Quick Action Bar
        // =========================================================================
        if touch.bar_open {
            let bar_y = pill_y + pill_h + 3.0;
            let btn_h = 18.0f32;
            let btn_gap = 3.0f32;

            let has_schem = schematic.is_some();

            let rot_str = format!("Rot:{}", placement.rotation.display_name());
            let mir_str = format!("Mir:{}", placement.mirror.display_name());
            let layer_str = format!("L:{}", placement.layer_mode.display_name());
            let verif_str = if verifier.active { "Ver:ON" } else { "Ver:OFF" };

            let mut bar_items: Vec<(&'static str, String, bool, TouchAction)> = vec![
                ("btn_load", "📁Load".to_string(), false, TouchAction::OpenFileBrowser),
            ];

            if has_schem {
                bar_items.push(("btn_layer", layer_str, false, TouchAction::ToggleLayer));
                bar_items.push(("btn_lyr_up", "▲".to_string(), false, TouchAction::LayerUp));
                bar_items.push(("btn_lyr_dn", "▼".to_string(), false, TouchAction::LayerDown));
                bar_items.push(("btn_rot", rot_str, false, TouchAction::CycleRotation));
                bar_items.push(("btn_mir", mir_str, false, TouchAction::CycleMirror));
                bar_items.push(("btn_ver", verif_str.to_string(), verifier.active, TouchAction::ToggleVerifier));
                bar_items.push(("btn_mat", "📦Mat".to_string(), false, TouchAction::OpenMaterialList));
                bar_items.push(("btn_pos", "📍Pos".to_string(), false, TouchAction::SetPositionHere));
                bar_items.push(("btn_unl", "❌Clr".to_string(), false, TouchAction::UnloadSchematic));
            }

            let mut widths = Vec::new();
            for (_, text, _, _) in &bar_items {
                let w = (HudBatch::text_width(atlas, text) + 8.0).max(18.0);
                widths.push(w);
            }
            let total_bar_w: f32 = widths.iter().sum::<f32>() + (btn_gap * (bar_items.len() - 1) as f32);
            let mut curr_x = ((sw - total_bar_w) * 0.5).max(4.0);

            for (i, (id, text, active, act)) in bar_items.into_iter().enumerate() {
                let bw = widths[i];
                let bg_col = if active {
                    rgba(30, 140, 60, 230)
                } else {
                    rgba(40, 44, 52, 215)
                };

                batch.fill(atlas, curr_x, bar_y, bw, btn_h, bg_col);
                batch.fill(atlas, curr_x, bar_y, bw, 1.0, rgba(120, 130, 145, 200));
                batch.fill(atlas, curr_x, bar_y + btn_h - 1.0, bw, 1.0, rgba(20, 24, 30, 200));

                let t_w = HudBatch::text_width(atlas, &text);
                batch.text_styled(
                    atlas,
                    &Text::literal(&text),
                    curr_x + (bw - t_w) * 0.5,
                    bar_y + 5.0,
                    if active { rgba(255, 255, 120, 255) } else { rgba(240, 240, 240, 255) },
                    true,
                );

                touch.buttons.push(TouchButton {
                    id,
                    rect: [curr_x, bar_y, bw, btn_h],
                    label: text,
                    active,
                    action: act,
                });

                curr_x += bw + btn_gap;
            }
        }

        // =========================================================================
        // 3. Verifier Floating Chip
        // =========================================================================
        if let Some(schem) = schematic {
            if verifier.active {
                let chip_x = 4.0f32;
                let chip_y = 4.0f32;
                let chip_w = 175.0f32;
                let chip_h = 28.0f32;

                batch.fill(atlas, chip_x, chip_y, chip_w, chip_h, rgba(24, 28, 36, 190));
                batch.fill(atlas, chip_x, chip_y, chip_w, 1.0, rgba(80, 120, 180, 200));

                let title = format!("[HZ-MAT] {}", truncate_str(&schem.name, 16));
                batch.text_styled(
                    atlas,
                    &Text::literal(&title),
                    chip_x + 4.0,
                    chip_y + 3.0,
                    rgba(255, 215, 0, 255),
                    true,
                );

                let stats_str = format!(
                    "✓{}  ✗{}  ⚠{}  +{} ({:.0}%)",
                    verifier.stats.matching,
                    verifier.stats.missing,
                    verifier.stats.wrong_block,
                    verifier.stats.extra,
                    verifier.stats.percent_complete()
                );
                batch.text_styled(
                    atlas,
                    &Text::literal(&stats_str),
                    chip_x + 4.0,
                    chip_y + 15.0,
                    rgba(220, 230, 240, 255),
                    true,
                );
            }
        }

        // =========================================================================
        // 4. Modal Dialogs (File Browser / Material List)
        // =========================================================================
        match touch.active_modal {
            ModalKind::None => {}
            ModalKind::FileBrowser => {
                draw_file_browser_modal(batch, atlas, touch, schematic_files, sw, sh);
            }
            ModalKind::MaterialList => {
                draw_material_list_modal(batch, atlas, touch, materials, sw, sh);
            }
            ModalKind::LayerConfig => {}
        }

        // =========================================================================
        // 5. Toast Notifications
        // =========================================================================
        if let Some((msg, created)) = toast {
            let elapsed = created.elapsed().as_secs_f32();
            if elapsed < 2.5 {
                let alpha = ((2.5 - elapsed) / 0.5).min(1.0);
                let a_u8 = (alpha * 240.0) as u8;
                let toast_w = (HudBatch::text_width(atlas, msg) + 16.0).max(80.0);
                let toast_h = 16.0f32;
                let toast_x = (sw - toast_w) * 0.5;
                let toast_y = sh - 36.0;

                batch.fill(atlas, toast_x, toast_y, toast_w, toast_h, rgba(20, 24, 30, a_u8));
                batch.fill(atlas, toast_x, toast_y, toast_w, 1.0, rgba(60, 160, 240, a_u8));
                let tw = HudBatch::text_width(atlas, msg);
                batch.text_styled(
                    atlas,
                    &Text::literal(msg),
                    toast_x + (toast_w - tw) * 0.5,
                    toast_y + 4.0,
                    rgba(255, 255, 255, (alpha * 255.0) as u8),
                    true,
                );
            }
        }
    }
}

fn draw_file_browser_modal(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    touch: &mut TouchUiState,
    files: &[SchematicFileInfo],
    sw: f32,
    sh: f32,
) {
    // Backdrop
    batch.fill(atlas, 0.0, 0.0, sw, sh, rgba(0, 0, 0, 160));

    let win_w = (sw - 40.0).clamp(240.0, 360.0);
    let win_h = (sh - 40.0).clamp(160.0, 240.0);
    let win_x = (sw - win_w) * 0.5;
    let win_y = (sh - win_h) * 0.5;

    // Window frame
    batch.fill(atlas, win_x, win_y, win_w, win_h, rgba(32, 36, 44, 245));
    batch.fill(atlas, win_x, win_y, win_w, 1.0, rgba(100, 140, 200, 255));
    batch.fill(atlas, win_x, win_y + win_h - 1.0, win_w, 1.0, rgba(20, 24, 30, 255));
    batch.fill(atlas, win_x, win_y, 1.0, win_h, rgba(100, 140, 200, 255));
    batch.fill(atlas, win_x + win_w - 1.0, win_y, 1.0, win_h, rgba(20, 24, 30, 255));

    // Title Bar
    batch.fill(atlas, win_x + 1.0, win_y + 1.0, win_w - 2.0, 18.0, rgba(20, 24, 32, 250));
    batch.text_styled(
        atlas,
        &Text::literal("📁 Schematics (/sdcard/Documents/Horizonmatica)"),
        win_x + 6.0,
        win_y + 5.0,
        rgba(255, 255, 255, 255),
        true,
    );

    // Close [X] Button
    let close_w = 16.0f32;
    let close_h = 14.0f32;
    let close_x = win_x + win_w - close_w - 3.0;
    let close_y = win_y + 3.0;
    batch.fill(atlas, close_x, close_y, close_w, close_h, rgba(180, 40, 40, 230));
    batch.text_styled(atlas, &Text::literal("✕"), close_x + 4.0, close_y + 3.0, rgba(255, 255, 255, 255), true);
    touch.modal_buttons.push(TouchButton {
        id: "btn_modal_close",
        rect: [close_x, close_y, close_w, close_h],
        label: "Close".into(),
        active: false,
        action: TouchAction::CloseModal,
    });

    // Content area
    let content_x = win_x + 4.0;
    let content_y = win_y + 22.0;
    let content_w = win_w - 8.0;
    let content_h = win_h - 44.0;

    if files.is_empty() {
        batch.text_styled(
            atlas,
            &Text::literal("No schematics found in folder."),
            content_x + 8.0,
            content_y + 20.0,
            rgba(200, 200, 200, 255),
            true,
        );
        batch.text_styled(
            atlas,
            &Text::literal("Copy .litematic, .schem or .nbt files there."),
            content_x + 8.0,
            content_y + 36.0,
            rgba(160, 160, 160, 255),
            true,
        );
    } else {
        let row_h = 20.0f32;
        let visible_rows = (content_h / row_h).floor() as usize;
        let start_idx = (touch.scroll_offset as usize).min(files.len().saturating_sub(1));
        let end_idx = (start_idx + visible_rows).min(files.len());

        let mut row_y = content_y;
        for (i, f) in files[start_idx..end_idx].iter().enumerate() {
            let actual_idx = start_idx + i;
            let bg_c = if actual_idx % 2 == 0 {
                rgba(40, 45, 55, 180)
            } else {
                rgba(30, 34, 42, 180)
            };
            batch.fill(atlas, content_x, row_y, content_w, row_h - 2.0, bg_c);

            let display_txt = format!("{} ({})", truncate_str(&f.filename, 22), f.formatted_size());
            batch.text_styled(
                atlas,
                &Text::literal(&display_txt),
                content_x + 4.0,
                row_y + 5.0,
                rgba(240, 240, 240, 255),
                true,
            );

            // [Load] Button
            let btn_w = 42.0f32;
            let btn_x = content_x + content_w - btn_w - 2.0;
            batch.fill(atlas, btn_x, row_y + 1.0, btn_w, row_h - 4.0, rgba(20, 130, 60, 220));
            batch.text_styled(
                atlas,
                &Text::literal("Load"),
                btn_x + 10.0,
                row_y + 5.0,
                rgba(255, 255, 255, 255),
                true,
            );

            touch.modal_buttons.push(TouchButton {
                id: "btn_file_load",
                rect: [btn_x, row_y + 1.0, btn_w, row_h - 4.0],
                label: f.filename.clone(),
                active: false,
                action: TouchAction::LoadFile(f.filename.clone()),
            });

            row_y += row_h;
        }

        // Scroll buttons
        let s_btn_w = 20.0f32;
        let s_btn_h = 16.0f32;
        let s_up_x = win_x + win_w - 52.0;
        let s_up_y = win_y + win_h - 20.0;
        batch.fill(atlas, s_up_x, s_up_y, s_btn_w, s_btn_h, rgba(60, 65, 75, 220));
        batch.text_styled(atlas, &Text::literal("▲"), s_up_x + 6.0, s_up_y + 4.0, rgba(255, 255, 255, 255), true);
        touch.modal_buttons.push(TouchButton {
            id: "btn_scroll_up",
            rect: [s_up_x, s_up_y, s_btn_w, s_btn_h],
            label: "Up".into(),
            active: false,
            action: TouchAction::ScrollUp,
        });

        let s_dn_x = win_x + win_w - 28.0;
        let s_dn_y = s_up_y;
        batch.fill(atlas, s_dn_x, s_dn_y, s_btn_w, s_btn_h, rgba(60, 65, 75, 220));
        batch.text_styled(atlas, &Text::literal("▼"), s_dn_x + 6.0, s_dn_y + 4.0, rgba(255, 255, 255, 255), true);
        touch.modal_buttons.push(TouchButton {
            id: "btn_scroll_dn",
            rect: [s_dn_x, s_dn_y, s_btn_w, s_btn_h],
            label: "Down".into(),
            active: false,
            action: TouchAction::ScrollDown,
        });
    }

    // [Rescan] Button at bottom left
    let rescan_w = 50.0f32;
    let rescan_h = 16.0f32;
    let rescan_x = win_x + 6.0;
    let rescan_y = win_y + win_h - 20.0;
    batch.fill(atlas, rescan_x, rescan_y, rescan_w, rescan_h, rgba(50, 70, 90, 220));
    batch.text_styled(atlas, &Text::literal("Rescan"), rescan_x + 7.0, rescan_y + 4.0, rgba(220, 230, 240, 255), true);
    touch.modal_buttons.push(TouchButton {
        id: "btn_rescan",
        rect: [rescan_x, rescan_y, rescan_w, rescan_h],
        label: "Rescan".into(),
        active: false,
        action: TouchAction::RescanFiles,
    });
}

fn draw_material_list_modal(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    touch: &mut TouchUiState,
    report: &MaterialReport,
    sw: f32,
    sh: f32,
) {
    // Backdrop
    batch.fill(atlas, 0.0, 0.0, sw, sh, rgba(0, 0, 0, 160));

    let win_w = (sw - 30.0).clamp(260.0, 380.0);
    let win_h = (sh - 40.0).clamp(160.0, 240.0);
    let win_x = (sw - win_w) * 0.5;
    let win_y = (sh - win_h) * 0.5;

    // Window frame
    batch.fill(atlas, win_x, win_y, win_w, win_h, rgba(32, 36, 44, 245));
    batch.fill(atlas, win_x, win_y, win_w, 1.0, rgba(100, 140, 200, 255));
    batch.fill(atlas, win_x, win_y + win_h - 1.0, win_w, 1.0, rgba(20, 24, 30, 255));
    batch.fill(atlas, win_x, win_y, 1.0, win_h, rgba(100, 140, 200, 255));
    batch.fill(atlas, win_x + win_w - 1.0, win_y, 1.0, win_h, rgba(20, 24, 30, 255));

    // Title Bar
    batch.fill(atlas, win_x + 1.0, win_y + 1.0, win_w - 2.0, 18.0, rgba(20, 24, 32, 250));
    let title = format!("📦 Material List (Total: {})", report.total_required);
    batch.text_styled(
        atlas,
        &Text::literal(&title),
        win_x + 6.0,
        win_y + 5.0,
        rgba(255, 215, 0, 255),
        true,
    );

    // Close [X] Button
    let close_w = 16.0f32;
    let close_h = 14.0f32;
    let close_x = win_x + win_w - close_w - 3.0;
    let close_y = win_y + 3.0;
    batch.fill(atlas, close_x, close_y, close_w, close_h, rgba(180, 40, 40, 230));
    batch.text_styled(atlas, &Text::literal("✕"), close_x + 4.0, close_y + 3.0, rgba(255, 255, 255, 255), true);
    touch.modal_buttons.push(TouchButton {
        id: "btn_modal_close",
        rect: [close_x, close_y, close_w, close_h],
        label: "Close".into(),
        active: false,
        action: TouchAction::CloseModal,
    });

    // Table Header
    let hdr_y = win_y + 20.0;
    batch.fill(atlas, win_x + 2.0, hdr_y, win_w - 4.0, 14.0, rgba(40, 48, 60, 230));
    batch.text_styled(atlas, &Text::literal("Item"), win_x + 6.0, hdr_y + 3.0, rgba(180, 200, 220, 255), true);
    batch.text_styled(atlas, &Text::literal("Need"), win_x + win_w - 120.0, hdr_y + 3.0, rgba(180, 200, 220, 255), true);
    batch.text_styled(atlas, &Text::literal("Inv"), win_x + win_w - 75.0, hdr_y + 3.0, rgba(180, 200, 220, 255), true);
    batch.text_styled(atlas, &Text::literal("Miss"), win_x + win_w - 35.0, hdr_y + 3.0, rgba(180, 200, 220, 255), true);

    // Rows
    let list_y = hdr_y + 15.0;
    let list_h = win_h - 52.0;

    if report.entries.is_empty() {
        batch.text_styled(
            atlas,
            &Text::literal("No materials needed for this layer / schematic."),
            win_x + 10.0,
            list_y + 20.0,
            rgba(200, 200, 200, 255),
            true,
        );
    } else {
        let row_h = 16.0f32;
        let visible_rows = (list_h / row_h).floor() as usize;
        let start_idx = (touch.scroll_offset as usize).min(report.entries.len().saturating_sub(1));
        let end_idx = (start_idx + visible_rows).min(report.entries.len());

        let mut curr_y = list_y;
        for (i, entry) in report.entries[start_idx..end_idx].iter().enumerate() {
            let actual_idx = start_idx + i;
            let bg_c = if actual_idx % 2 == 0 {
                rgba(35, 40, 50, 150)
            } else {
                rgba(25, 29, 36, 150)
            };
            batch.fill(atlas, win_x + 2.0, curr_y, win_w - 4.0, row_h - 1.0, bg_c);

            let txt_color = if entry.missing == 0 {
                rgba(120, 230, 120, 255) // Green
            } else if entry.available > 0 {
                rgba(240, 210, 80, 255) // Yellow
            } else {
                rgba(240, 100, 100, 255) // Red
            };

            batch.text_styled(
                atlas,
                &Text::literal(&truncate_str(&entry.display_name, 18)),
                win_x + 6.0,
                curr_y + 3.0,
                txt_color,
                true,
            );

            let need_s = format!("{}", entry.required);
            batch.text_styled(atlas, &Text::literal(&need_s), win_x + win_w - 120.0, curr_y + 3.0, rgba(220, 220, 220, 255), true);

            let inv_s = format!("{}", entry.available);
            batch.text_styled(atlas, &Text::literal(&inv_s), win_x + win_w - 75.0, curr_y + 3.0, rgba(220, 220, 220, 255), true);

            let miss_s = format!("{}", entry.missing);
            batch.text_styled(atlas, &Text::literal(&miss_s), win_x + win_w - 35.0, curr_y + 3.0, txt_color, true);

            curr_y += row_h;
        }

        // Scroll buttons
        let s_btn_w = 20.0f32;
        let s_btn_h = 14.0f32;
        let s_up_x = win_x + win_w - 52.0;
        let s_up_y = win_y + win_h - 18.0;
        batch.fill(atlas, s_up_x, s_up_y, s_btn_w, s_btn_h, rgba(60, 65, 75, 220));
        batch.text_styled(atlas, &Text::literal("▲"), s_up_x + 6.0, s_up_y + 3.0, rgba(255, 255, 255, 255), true);
        touch.modal_buttons.push(TouchButton {
            id: "btn_scroll_up",
            rect: [s_up_x, s_up_y, s_btn_w, s_btn_h],
            label: "Up".into(),
            active: false,
            action: TouchAction::ScrollUp,
        });

        let s_dn_x = win_x + win_w - 28.0;
        let s_dn_y = s_up_y;
        batch.fill(atlas, s_dn_x, s_dn_y, s_btn_w, s_btn_h, rgba(60, 65, 75, 220));
        batch.text_styled(atlas, &Text::literal("▼"), s_dn_x + 6.0, s_dn_y + 3.0, rgba(255, 255, 255, 255), true);
        touch.modal_buttons.push(TouchButton {
            id: "btn_scroll_dn",
            rect: [s_dn_x, s_dn_y, s_btn_w, s_btn_h],
            label: "Down".into(),
            active: false,
            action: TouchAction::ScrollDown,
        });
    }

    // Totals bar
    let tot_str = format!(
        "Req: {} | Inv: {} | Miss: {}",
        report.total_required, report.total_available, report.total_missing
    );
    batch.text_styled(atlas, &Text::literal(&tot_str), win_x + 6.0, win_y + win_h - 15.0, rgba(220, 230, 240, 255), true);
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let mut truncated: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        truncated.push('…');
        truncated
    }
}
