// Copyright (c) 2026 VkYakee
// VkYakee VoxelMap x seedcrackerX
//! HUD rendering pipeline for HorizonVoxelCracker.
//! Features real-time terrain and dynamic layer cave rendering (Layer::Below and Layer::Surface),
//! entity radar, waypoint projection, and Minecraft Bedrock touch UI.

use super::cracker::{CrackState, CrackerEngine};
use super::radar::RadarBlip;
use super::storage;
use super::touch_ui::{TouchAction, TouchButton, TouchUiState};
use super::waypoints::WaypointManager;
use horizon_client::{
    hud::{HudBatch, MINIMAP_SPRITE, rgba},
    text::Text,
};

#[derive(Clone, Copy, Debug)]
pub struct HudSettings {
    pub show_minimap: bool,
    pub show_radar: bool,
    pub show_coords: bool,
    pub show_waypoints: bool,
    pub show_cracker_status: bool,
    pub cave_mode: bool,
    pub minimap_size: f32,
}

impl Default for HudSettings {
    fn default() -> Self {
        Self {
            show_minimap: true,
            show_radar: true,
            show_coords: true,
            show_waypoints: true,
            show_cracker_status: true,
            cave_mode: false,
            minimap_size: 110.0,
        }
    }
}

pub struct HudRenderer;

impl HudRenderer {
    pub fn draw(
        batch: &mut HudBatch,
        settings: &HudSettings,
        touch: &mut TouchUiState,
        player_pos: [f64; 3],
        player_yaw: f32,
        biome: Option<&str>,
        dimension: &str,
        world_id: &str,
        waypoints: &WaypointManager,
        radar_blips: &[RadarBlip],
        cracker: &CrackerEngine,
        atlas: &horizon_client::hud::HudAtlas,
        corner_pos: (f32, f32),
    ) {
        touch.buttons.clear();
        touch.modal_buttons.clear();

        let sw = batch.width;
        let sh = batch.height;

        // ======================================================== 1. Bedrock Floating Toggle Button
        let pill_w = 46.0f32;
        let pill_h = 16.0f32;
        let pill_x = (sw * 0.5) - (pill_w * 0.5);
        let pill_y = 4.0f32;

        let pill_bg = if touch.bar_open {
            rgba(40, 120, 220, 230)
        } else {
            rgba(30, 35, 40, 200)
        };
        batch.fill(atlas, pill_x, pill_y, pill_w, pill_h, pill_bg);
        batch.fill(atlas, pill_x, pill_y, pill_w, 1.0, rgba(160, 180, 200, 220));
        batch.fill(atlas, pill_x, pill_y + pill_h - 1.0, pill_w, 1.0, rgba(60, 70, 80, 220));

        let pill_text = if touch.bar_open { "▲ HVC" } else { "▼ HVC" };
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
            label: "HVC Toggle".into(),
            active: touch.bar_open,
            action: TouchAction::ToggleBar,
        });

        // ======================================================== 2. Bedrock Touch Quick Action Bar
        if touch.bar_open {
            let bar_y = pill_y + pill_h + 3.0;
            let btn_w = 34.0f32;
            let btn_h = 22.0f32;
            let spacing = 4.0f32;
            let total_buttons = 7;
            let total_bar_w = (btn_w * total_buttons as f32) + (spacing * (total_buttons - 1) as f32) + 8.0;
            let bar_x = (sw * 0.5) - (total_bar_w * 0.5);

            batch.fill(atlas, bar_x, bar_y, total_bar_w, btn_h + 6.0, rgba(20, 24, 28, 230));
            batch.fill(atlas, bar_x, bar_y, total_bar_w, 1.0, rgba(90, 105, 120, 255));
            batch.fill(atlas, bar_x, bar_y + btn_h + 5.0, total_bar_w, 1.0, rgba(30, 35, 40, 255));

            let mut bx = bar_x + 4.0;
            let by = bar_y + 3.0;

            let mut add_bar_btn = |batch: &mut HudBatch, id: &'static str, label: &str, active: bool, action: TouchAction| {
                let btn_bg = if active {
                    rgba(50, 140, 240, 240)
                } else {
                    rgba(55, 60, 68, 220)
                };
                batch.fill(atlas, bx, by, btn_w, btn_h, btn_bg);
                batch.fill(atlas, bx, by, btn_w, 1.0, rgba(140, 150, 165, 200));
                batch.fill(atlas, bx, by + btn_h - 1.0, btn_w, 1.0, rgba(25, 28, 32, 200));

                let tw = HudBatch::text_width(atlas, label);
                batch.text_styled(
                    atlas,
                    &Text::literal(label),
                    bx + (btn_w - tw) * 0.5,
                    by + 7.0,
                    if active { rgba(255, 255, 255, 255) } else { rgba(220, 225, 230, 255) },
                    true,
                );

                touch.buttons.push(TouchButton {
                    id,
                    rect: [bx, by, btn_w, btn_h],
                    label: label.into(),
                    active,
                    action,
                });
                bx += btn_w + spacing;
            };

            let map_label = match touch.map_mode {
                0 => "Map",
                1 => "Big",
                _ => "Off",
            };
            add_bar_btn(batch, "btn_map", map_label, touch.map_mode != 2, TouchAction::CycleMapMode);
            add_bar_btn(batch, "btn_cave", "Cave", settings.cave_mode, TouchAction::ToggleCave);
            add_bar_btn(batch, "btn_wp", "+WP", false, TouchAction::QuickWaypoint);
            add_bar_btn(batch, "btn_radar", "Radar", settings.show_radar, TouchAction::ToggleRadar);
            add_bar_btn(batch, "btn_cracker", "Crk", touch.modal_open, TouchAction::OpenCrackerModal);
            add_bar_btn(batch, "btn_crack_now", "Run", false, TouchAction::ForceCrack);
            add_bar_btn(batch, "btn_reset", "Del", false, TouchAction::ResetData);
        }

        let mut last_map_bottom = corner_pos.1;

        // ======================================================== 3. Real-time Terrain & Layer Minimap
        if touch.map_mode != 2 && settings.show_minimap {
            let (map_x, map_y, map_w, map_h) = if touch.map_mode == 1 {
                // Large Big Map Mode: centered on screen
                let size = 180.0f32.min(sh - 60.0);
                ((sw - size) * 0.5, (sh - size) * 0.5, size, size)
            } else {
                // Normal Corner Mode
                let size = settings.minimap_size;
                (corner_pos.0, corner_pos.1, size, size)
            };
            last_map_bottom = map_y + map_h;

            // 3.1 Background shade
            let bg_color = if settings.cave_mode {
                rgba(22, 12, 32, 230)
            } else {
                rgba(12, 16, 22, 210)
            };
            batch.fill(atlas, map_x, map_y, map_w, map_h, bg_color);

            // 3.2 Real-time terrain layer render: draw the dynamic composed block map texture
            if let Some(terrain_sprite) = atlas.sprites.get(MINIMAP_SPRITE).copied() {
                batch.rect_uv(map_x, map_y, map_w, map_h, terrain_sprite, u32::MAX);
            }

            // 3.3 Border
            let border_col = if settings.cave_mode {
                rgba(160, 90, 240, 255) // Purple for cave layer
            } else {
                rgba(70, 130, 210, 255) // Blue for surface
            };
            batch.fill(atlas, map_x, map_y, map_w, 1.0, border_col);
            batch.fill(atlas, map_x, map_y + map_h - 1.0, map_w, 1.0, border_col);
            batch.fill(atlas, map_x, map_y, 1.0, map_h, border_col);
            batch.fill(atlas, map_x + map_w - 1.0, map_y, 1.0, map_h, border_col);

            let cx = map_x + map_w * 0.5;
            let cy = map_y + map_h * 0.5;

            // 3.4 Cardinal Directions
            let rot_rad = player_yaw.to_radians();
            let cos_y = (-rot_rad).cos();
            let sin_y = (-rot_rad).sin();

            let draw_cardinal = |batch: &mut HudBatch, label: &str, dir_x: f32, dir_z: f32, color: u32| {
                let rx = dir_x * cos_y - dir_z * sin_y;
                let rz = dir_x * sin_y + dir_z * cos_y;
                let radius = map_w * 0.44;
                let px = cx + rx * radius - 3.0;
                let py = cy + rz * radius - 4.0;
                batch.text_styled(atlas, &Text::literal(label), px, py, color, true);
            };

            draw_cardinal(batch, "N", 0.0, -1.0, rgba(255, 80, 80, 255));
            draw_cardinal(batch, "S", 0.0, 1.0, rgba(220, 220, 220, 255));
            draw_cardinal(batch, "E", 1.0, 0.0, rgba(220, 220, 220, 255));
            draw_cardinal(batch, "W", -1.0, 0.0, rgba(220, 220, 220, 255));

            // Player Marker
            batch.fill(atlas, cx - 1.5, cy - 1.5, 3.0, 3.0, rgba(255, 255, 255, 255));

            // 3.5 Radar Blips projected over terrain
            if settings.show_radar {
                let range = if touch.map_mode == 1 { 128.0f64 } else { 64.0f64 };
                for blip in radar_blips {
                    let dx = blip.pos[0] - player_pos[0];
                    let dz = blip.pos[2] - player_pos[2];
                    if dx.abs() > range || dz.abs() > range {
                        continue;
                    }
                    let rdx = dx * cos_y as f64 - dz * sin_y as f64;
                    let rdz = dx * sin_y as f64 + dz * cos_y as f64;

                    let screen_x = cx + (rdx / range) as f32 * (map_w * 0.44);
                    let screen_y = cy + (rdz / range) as f32 * (map_h * 0.44);

                    let [_, r, g, b] = blip.color.to_be_bytes();
                    batch.fill(atlas, screen_x - 1.5, screen_y - 1.5, 3.0, 3.0, rgba(r, g, b, 240));
                }
            }

            // 3.6 Waypoints projected over terrain
            if settings.show_waypoints {
                for wp in waypoints.all() {
                    if !wp.enabled || wp.dim != dimension {
                        continue;
                    }
                    let dx = wp.pos[0] as f64 - player_pos[0];
                    let dz = wp.pos[2] as f64 - player_pos[2];

                    let rdx = dx * cos_y as f64 - dz * sin_y as f64;
                    let rdz = dx * sin_y as f64 + dz * cos_y as f64;

                    let mut clamped_x = (rdx / 64.0) as f32 * (map_w * 0.44);
                    let mut clamped_y = (rdz / 64.0) as f32 * (map_h * 0.44);
                    let dist = (clamped_x * clamped_x + clamped_y * clamped_y).sqrt();
                    let max_r = map_w * 0.44;
                    if dist > max_r {
                        clamped_x = (clamped_x / dist) * max_r;
                        clamped_y = (clamped_y / dist) * max_r;
                    }

                    let [_, r, g, b] = wp.color.to_be_bytes();
                    batch.fill(atlas, cx + clamped_x - 2.0, cy + clamped_y - 2.0, 4.0, 4.0, rgba(r, g, b, 255));
                }
            }

            // 3.7 Coordinates, Current Biome & Layer Indicator
            if settings.show_coords && touch.map_mode == 0 {
                let text_y = map_y + map_h + 3.0;
                let layer_str = if settings.cave_mode {
                    format!(" [Cave Y:{:.0}]", player_pos[1])
                } else {
                    format!(" [Y:{:.0}]", player_pos[1])
                };
                let coords_text = format!(
                    "XYZ: {:.0} / {:.0} / {:.0}{}",
                    player_pos[0], player_pos[1], player_pos[2], layer_str
                );
                batch.text_styled(atlas, &Text::literal(&coords_text), map_x, text_y, rgba(230, 230, 230, 255), true);

                if let Some(b) = biome {
                    let biome_text = format!("Biome: {}", b);
                    batch.text_styled(atlas, &Text::literal(&biome_text), map_x, text_y + 10.0, rgba(160, 210, 255, 255), true);
                }
            }
        }

        // ======================================================== 4. In-World Waypoints Projections
        if settings.show_waypoints && touch.map_mode != 1 {
            let mut wp_y = if touch.map_mode == 0 && settings.show_minimap {
                last_map_bottom + 26.0
            } else {
                corner_pos.1 + 8.0
            };
            let list = waypoints.all();
            for wp in list.iter().filter(|w| w.enabled && w.dim == dimension) {
                let dist = wp.distance_from(player_pos[0], player_pos[1], player_pos[2]);
                let wp_line = format!("[{}] {:.0}m", wp.name, dist);
                let [_, r, g, b] = wp.color.to_be_bytes();
                batch.text_styled(atlas, &Text::literal(&wp_line), corner_pos.0, wp_y, rgba(r, g, b, 255), true);
                wp_y += 10.0;
                if wp_y > sh - 45.0 {
                    break;
                }
            }
        }

        // ======================================================== 5. Seedcracker Bedrock Modal Dialog
        if touch.modal_open {
            let mw = 270.0f32.min(sw - 20.0);
            let mh = 185.0f32.min(sh - 20.0);
            let mx = (sw - mw) * 0.5;
            let my = (sh - mh) * 0.5;

            batch.fill(atlas, mx, my, mw, mh, rgba(28, 32, 38, 250));
            batch.fill(atlas, mx, my, mw, 2.0, rgba(80, 140, 220, 255));
            batch.fill(atlas, mx, my + mh - 2.0, mw, 2.0, rgba(20, 24, 28, 255));
            batch.fill(atlas, mx, my, 2.0, mh, rgba(80, 140, 220, 255));
            batch.fill(atlas, mx + mw - 2.0, my, 2.0, mh, rgba(20, 24, 28, 255));

            let title = "SeedcrackerX (Bedrock Touch)";
            let tw = HudBatch::text_width(atlas, title);
            batch.text_styled(atlas, &Text::literal(title), mx + (mw - tw) * 0.5, my + 8.0, rgba(255, 215, 0, 255), true);

            // Close button
            let close_btn_w = 18.0f32;
            let close_btn_h = 16.0f32;
            let close_x = mx + mw - close_btn_w - 6.0;
            let close_y = my + 6.0;
            batch.fill(atlas, close_x, close_y, close_btn_w, close_btn_h, rgba(180, 40, 40, 230));
            batch.text_styled(atlas, &Text::literal("X"), close_x + 5.0, close_y + 4.0, rgba(255, 255, 255, 255), true);

            touch.modal_buttons.push(TouchButton {
                id: "btn_modal_close",
                rect: [close_x, close_y, close_btn_w, close_btn_h],
                label: "Close".into(),
                active: false,
                action: TouchAction::CloseCrackerModal,
            });

            // Content
            let feats_count = cracker.features.lock().unwrap().len();
            let total_bits = cracker.total_bits();
            let state = cracker.state.lock().unwrap().clone();
            let s_seeds = cracker.candidate_structure_seeds.lock().unwrap().clone();
            let w_seeds = cracker.candidate_world_seeds.lock().unwrap().clone();
            let cur_version = *cracker.version.lock().unwrap();

            let mut line_y = my + 26.0;

            // World name & device storage path
            let storage_root = storage::device_storage_root();
            let world_line = format!("World: {} (in {})", world_id, storage_root.display());
            batch.text_styled(atlas, &Text::literal(&world_line), mx + 12.0, line_y, rgba(140, 200, 255, 255), true);
            line_y += 11.0;

            // Version selector touch button
            let ver_label = format!("Ver: {}", cur_version.name());
            let ver_w = HudBatch::text_width(atlas, &ver_label) + 12.0;
            let ver_h = 13.0f32;
            batch.fill(atlas, mx + 12.0, line_y, ver_w, ver_h, rgba(45, 55, 68, 240));
            batch.fill(atlas, mx + 12.0, line_y, ver_w, 1.0, rgba(100, 140, 200, 200));
            batch.text_styled(atlas, &Text::literal(&ver_label), mx + 18.0, line_y + 3.0, rgba(255, 255, 255, 255), true);

            touch.modal_buttons.push(TouchButton {
                id: "btn_modal_version",
                rect: [mx + 12.0, line_y, ver_w, ver_h],
                label: "Version".into(),
                active: false,
                action: TouchAction::CycleVersion,
            });
            line_y += 16.0;

            // Entropy bits progress bar
            let bits_str = format!("Entropy: {:.1} / 48 bits ({} structures)", total_bits, feats_count);
            batch.text_styled(atlas, &Text::literal(&bits_str), mx + 12.0, line_y, rgba(220, 225, 230, 255), true);
            line_y += 11.0;

            let bar_w = mw - 24.0;
            let fill_w = (total_bits / 48.0).clamp(0.0, 1.0) * bar_w;
            batch.fill(atlas, mx + 12.0, line_y, bar_w, 6.0, rgba(40, 45, 52, 255));
            batch.fill(atlas, mx + 12.0, line_y, fill_w, 6.0, rgba(50, 205, 50, 255));
            line_y += 12.0;

            // Structure Seed
            let struct_str = if let Some(&ss) = s_seeds.first() {
                format!("Structure Seed: {}", ss)
            } else {
                "Structure Seed: Need more structures".into()
            };
            batch.text_styled(atlas, &Text::literal(&struct_str), mx + 12.0, line_y, rgba(140, 210, 255, 255), true);
            line_y += 11.0;

            // World Seed
            let (world_str, world_col) = if let Some(&ws) = w_seeds.first() {
                (format!("★ WORLD SEED: {}", ws), rgba(85, 255, 85, 255))
            } else {
                ("World Seed: Resolving from hash...".into(), rgba(180, 180, 180, 255))
            };
            batch.text_styled(atlas, &Text::literal(&world_str), mx + 12.0, line_y, world_col, true);
            line_y += 12.0;

            // Status message
            let status_msg = match state {
                CrackState::Idle => "Status: Idle - explore structures in chunks".into(),
                CrackState::CollectingData { .. } => "Status: Collecting structures in world...".into(),
                CrackState::Searching { progress_percent, phase } => {
                    format!("Status: Cracking {:.0}% ({})", progress_percent, phase)
                }
                CrackState::Success { .. } => "Status: Successfully cracked and saved!".into(),
                CrackState::Failed { reason } => format!("Status: Failed ({})", reason),
            };
            batch.text_styled(atlas, &Text::literal(&status_msg), mx + 12.0, line_y, rgba(255, 215, 0, 255), true);

            // Action Touch Button
            let crack_btn_w = mw - 24.0;
            let crack_btn_h = 22.0f32;
            let crack_btn_x = mx + 12.0;
            let crack_btn_y = my + mh - crack_btn_h - 8.0;

            batch.fill(atlas, crack_btn_x, crack_btn_y, crack_btn_w, crack_btn_h, rgba(40, 130, 50, 240));
            batch.fill(atlas, crack_btn_x, crack_btn_y, crack_btn_w, 1.0, rgba(100, 220, 120, 255));
            let crack_txt = "▶ Run Seedcracker Engine";
            let ctw = HudBatch::text_width(atlas, crack_txt);
            batch.text_styled(
                atlas,
                &Text::literal(crack_txt),
                crack_btn_x + (crack_btn_w - ctw) * 0.5,
                crack_btn_y + 7.0,
                rgba(255, 255, 255, 255),
                true,
            );

            touch.modal_buttons.push(TouchButton {
                id: "btn_modal_crack",
                rect: [crack_btn_x, crack_btn_y, crack_btn_w, crack_btn_h],
                label: "Run Engine".into(),
                active: false,
                action: TouchAction::ForceCrack,
            });
        }

        // ======================================================== 6. Touch Screen Feedback Toast
        if let Some(feedback) = touch.current_feedback() {
            let toast_w = HudBatch::text_width(atlas, feedback) + 16.0;
            let toast_h = 16.0f32;
            let toast_x = (sw - toast_w) * 0.5;
            let toast_y = sh - 45.0;

            batch.fill(atlas, toast_x, toast_y, toast_w, toast_h, rgba(15, 20, 25, 235));
            batch.fill(atlas, toast_x, toast_y, toast_w, 1.0, rgba(70, 140, 220, 255));
            batch.text_styled(
                atlas,
                &Text::literal(feedback),
                toast_x + 8.0,
                toast_y + 4.0,
                rgba(255, 255, 255, 255),
                true,
            );
        }
    }
}
