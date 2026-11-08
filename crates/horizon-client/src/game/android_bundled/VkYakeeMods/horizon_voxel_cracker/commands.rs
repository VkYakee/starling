// Copyright (c) 2026 VkYakee
// VkYakee VoxelMap x seedcrackerX
//! In-game chat command handler for `/hvc` and `/horizonvoxelcracker`.

use super::cracker::{CrackState, CrackerEngine};
use super::hud::HudSettings;
use super::waypoints::{Waypoint, WaypointManager};
use std::sync::{Arc, Mutex};

pub struct CommandDispatcher;

impl CommandDispatcher {
    pub fn handle_command(
        cmd_line: &str,
        player_pos: [f64; 3],
        dimension: &str,
        settings: &Mutex<HudSettings>,
        waypoints: &WaypointManager,
        cracker: &Arc<CrackerEngine>,
        mut send_feedback: impl FnMut(&str),
    ) -> bool {
        let trimmed = cmd_line.trim();
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            return false;
        }

        let root = parts[0].to_ascii_lowercase();
        if root != "hvc" && root != "horizonvoxelcracker" && root != "seedcracker" {
            return false;
        }

        let sub = parts.get(1).copied().unwrap_or("help").to_ascii_lowercase();
        match sub.as_str() {
            "help" => {
                send_feedback("=== HorizonVoxelCracker Commands ===");
                send_feedback("/hvc status     - Show cracker and map state");
                send_feedback("/hvc seed       - Print cracked seed results");
                send_feedback("/hvc crack      - Run seed cracking on gathered data");
                send_feedback("/hvc data       - List detected structures");
                send_feedback("/hvc reset      - Clear collected structure data");
                send_feedback("/hvc cave       - Toggle cave mapping mode");
                send_feedback("/hvc radar      - Toggle entity radar");
                send_feedback("/hvc wp add <n> - Add waypoint at player location");
                send_feedback("/hvc wp list    - List saved waypoints");
                send_feedback("/hvc wp del <n> - Delete waypoint");
                send_feedback("/hvc wp clear   - Delete all user waypoints");
            }
            "status" => {
                let state = cracker.state.lock().unwrap().clone();
                match state {
                    CrackState::Idle => send_feedback("Cracker: Idle (no structures detected yet)"),
                    CrackState::CollectingData { bits, count } => {
                        send_feedback(&format!("Cracker: Collected {:.1}/48 bits from {} structures", bits, count));
                    }
                    CrackState::Searching { progress_percent, phase } => {
                        send_feedback(&format!("Cracker: Running {:.0}% ({})", progress_percent, phase));
                    }
                    CrackState::Success { structure_seed, world_seed } => {
                        send_feedback(&format!("Structure Seed: {}", structure_seed));
                        if let Some(ws) = world_seed {
                            send_feedback(&format!("World Seed: {}", ws));
                        }
                    }
                    CrackState::Failed { reason } => {
                        send_feedback(&format!("Cracker: Failed ({})", reason));
                    }
                }
                let w_count = waypoints.all().len();
                send_feedback(&format!("Waypoints active: {}", w_count));
            }
            "seed" => {
                let struct_seeds = cracker.candidate_structure_seeds.lock().unwrap().clone();
                let world_seeds = cracker.candidate_world_seeds.lock().unwrap().clone();
                if let Some(&first_w) = world_seeds.first() {
                    send_feedback(&format!("★ Found World Seed: {}", first_w));
                } else if let Some(&first_s) = struct_seeds.first() {
                    send_feedback(&format!("★ Found Structure Seed: {}", first_s));
                } else {
                    send_feedback("No seed has been cracked yet. Explore more chunks or run /hvc crack.");
                }
            }
            "crack" => {
                send_feedback("Starting seed cracking job in background...");
                if !cracker.start_cracking() {
                    send_feedback("Not enough data to start cracking (need at least 3 structures or 32 bits).");
                }
            }
            "reset" => {
                cracker.delete_world_data();
                send_feedback("Cracker world data and cached features deleted from storage.");
            }
            "version" | "ver" => {
                if let Some(arg) = parts.get(2).copied() {
                    let mut ver = cracker.version.lock().unwrap();
                    match arg.to_ascii_lowercase().as_str() {
                        "pre" | "legacy" | "1.17" => *ver = super::cracker::MCVersion::Pre1_18,
                        "1.18" | "modern" | "caves" => *ver = super::cracker::MCVersion::V1_18_Plus,
                        "1.21" | "trials" => *ver = super::cracker::MCVersion::V1_21_Plus,
                        _ => *ver = super::cracker::MCVersion::Auto,
                    }
                    send_feedback(&format!("Target Minecraft version set to: {}", ver.name()));
                } else {
                    let mut ver = cracker.version.lock().unwrap();
                    *ver = ver.cycle();
                    send_feedback(&format!("Target Minecraft version cycled to: {}", ver.name()));
                }
            }
            "data" => {
                let feats = cracker.features.lock().unwrap().clone();
                if feats.is_empty() {
                    send_feedback("No structures collected yet.");
                } else {
                    send_feedback(&format!("=== Collected Features ({}) ===", feats.len()));
                    for f in feats {
                        send_feedback(&format!(
                            "- {} at chunk ({}, {}), offset ({}, {}), {:.1} bits",
                            f.kind.name(),
                            f.chunk_x,
                            f.chunk_z,
                            f.offset_x,
                            f.offset_z,
                            f.kind.bits()
                        ));
                    }
                }
            }
            "cave" => {
                let mut s = settings.lock().unwrap();
                s.cave_mode = !s.cave_mode;
                send_feedback(&format!("Cave mode set to {}", if s.cave_mode { "ON" } else { "OFF" }));
            }
            "radar" => {
                let mut s = settings.lock().unwrap();
                s.show_radar = !s.show_radar;
                send_feedback(&format!("Entity radar set to {}", if s.show_radar { "ON" } else { "OFF" }));
            }
            "wp" | "waypoint" => {
                let action = parts.get(2).copied().unwrap_or("list").to_ascii_lowercase();
                match action.as_str() {
                    "add" => {
                        let name = parts[3..].join(" ");
                        let wp_name = if name.is_empty() { "Waypoint" } else { &name };
                        let block_pos = [player_pos[0].round() as i32, player_pos[1].round() as i32, player_pos[2].round() as i32];
                        let wp = Waypoint::new(wp_name, block_pos, dimension, 0xFF55_FFFF);
                        waypoints.add(wp);
                        send_feedback(&format!("Added waypoint '{}' at [{}, {}, {}]", wp_name, block_pos[0], block_pos[1], block_pos[2]));
                    }
                    "del" | "delete" | "remove" => {
                        let name = parts[3..].join(" ");
                        if waypoints.remove(&name) {
                            send_feedback(&format!("Removed waypoint '{}'", name));
                        } else {
                            send_feedback(&format!("Waypoint '{}' not found", name));
                        }
                    }
                    "clear" => {
                        waypoints.clear_user();
                        send_feedback("Cleared all user waypoints.");
                    }
                    "list" => {
                        let all = waypoints.all();
                        if all.is_empty() {
                            send_feedback("No waypoints saved.");
                        } else {
                            send_feedback(&format!("=== Waypoints ({}) ===", all.len()));
                            for w in all {
                                let dist = w.distance_from(player_pos[0], player_pos[1], player_pos[2]);
                                send_feedback(&format!("- [{}] at [{}, {}, {}] in {} ({:.0}m)", w.name, w.pos[0], w.pos[1], w.pos[2], w.dim, dist));
                            }
                        }
                    }
                    _ => {
                        send_feedback("Usage: /hvc wp [add <name> | del <name> | list | clear]");
                    }
                }
            }
            _ => {
                send_feedback("Unknown subcommand. Type /hvc help for help.");
            }
        }
        true
    }
}
