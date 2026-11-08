// Copyright (c) 2026 HorizonMatica Authors / Starling Android
//! In-game chat command dispatcher for /horizonmatica and /litematica.

use std::time::Instant;
use horizon_client::text::Text;

use super::format;
use super::materials::MaterialReport;
use super::placement::{LayerMode, Mirror, Rotation};
use super::runtime::{self, HorizonMaticaRuntime};
use super::storage;

pub struct CommandDispatcher;

impl CommandDispatcher {
    pub fn execute(
        game: &mut crate::game::Game<'_>,
        command: &str,
        rt: &HorizonMaticaRuntime,
    ) -> bool {
        let trimmed = command.trim_start_matches('/');
        let mut parts = trimmed.split_whitespace();

        let Some(root) = parts.next() else {
            return false;
        };

        if root != "horizonmatica" && root != "hzm" && root != "litematica" {
            return false;
        }

        let sub = parts.next().unwrap_or("help");
        let args: Vec<&str> = parts.collect();

        match sub {
            "load" => {
                let Some(name) = args.first() else {
                    send_chat(game, "[HZ-MAT] Usage: /horizonmatica load <filename>");
                    return true;
                };

                let _ = storage::ensure_schematics_dir();
                match storage::load_schematic_file(name) {
                    Ok((path, bytes)) => {
                        let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or(name);
                        match format::load_from_bytes(&bytes, fname) {
                            Ok(schem) => {
                                let total_b = schem.total_non_air_blocks();
                                let player_pos = if let Some(p) = game.player.as_ref() {
                                    [p.position.x.floor() as i32, p.position.y.floor() as i32, p.position.z.floor() as i32]
                                } else {
                                    [0, 64, 0]
                                };

                                let msg = format!("Loaded '{}' ({} blocks) at [{}, {}, {}]", schem.name, total_b, player_pos[0], player_pos[1], player_pos[2]);

                                {
                                    let mut s = rt.active_schematic.lock().unwrap();
                                    *s = Some(schem);
                                }
                                {
                                    let mut p = rt.placement.lock().unwrap();
                                    p.origin = player_pos;
                                    p.enabled = true;
                                }
                                {
                                    let mut v = rt.verifier.lock().unwrap();
                                    v.clear();
                                }

                                rt.set_toast(format!("Loaded '{}'", fname));
                                send_chat(game, &format!("[HZ-MAT] {msg}"));
                            }
                            Err(e) => {
                                send_chat(game, &format!("[HZ-MAT] Error loading schematic: {e}"));
                            }
                        }
                    }
                    Err(e) => {
                        send_chat(game, &format!("[HZ-MAT] File not found or read error: {e}"));
                    }
                }
            }

            "unload" | "clear" => {
                {
                    let mut s = rt.active_schematic.lock().unwrap();
                    *s = None;
                }
                {
                    let mut v = rt.verifier.lock().unwrap();
                    v.clear();
                }
                {
                    let mut m = rt.materials.lock().unwrap();
                    *m = None;
                }
                rt.set_toast("Schematic unloaded");
                send_chat(game, "[HZ-MAT] Active schematic unloaded.");
            }

            "list" => {
                let _ = storage::ensure_schematics_dir();
                let files = storage::list_schematics();
                if files.is_empty() {
                    send_chat(game, "[HZ-MAT] No schematics found in /sdcard/Documents/Horizonmatica/schematics");
                } else {
                    send_chat(game, &format!("[HZ-MAT] Found {} schematics:", files.len()));
                    for f in files.iter().take(10) {
                        send_chat(game, &format!(" - {} ({})", f.filename, f.formatted_size()));
                    }
                    if files.len() > 10 {
                        send_chat(game, &format!(" ... and {} more files", files.len() - 10));
                    }
                }
            }

            "rot" | "rotate" => {
                let mut p = rt.placement.lock().unwrap();
                if let Some(arg) = args.first() {
                    match *arg {
                        "0" | "none" => p.rotation = Rotation::None,
                        "90" | "cw90" => p.rotation = Rotation::Clockwise90,
                        "180" | "cw180" => p.rotation = Rotation::Clockwise180,
                        "270" | "ccw90" => p.rotation = Rotation::CounterClockwise90,
                        _ => p.rotation.cycle(),
                    }
                } else {
                    p.rotation.cycle();
                }
                let name = p.rotation.display_name();
                rt.set_toast(format!("Rotation: {name}"));
                send_chat(game, &format!("[HZ-MAT] Placement rotation set to {name}"));
            }

            "mir" | "mirror" => {
                let mut p = rt.placement.lock().unwrap();
                if let Some(arg) = args.first() {
                    match *arg {
                        "none" | "off" => p.mirror = Mirror::None,
                        "x" | "front_back" => p.mirror = Mirror::X,
                        "z" | "left_right" => p.mirror = Mirror::Z,
                        _ => p.mirror.cycle(),
                    }
                } else {
                    p.mirror.cycle();
                }
                let name = p.mirror.display_name();
                rt.set_toast(format!("Mirror: {name}"));
                send_chat(game, &format!("[HZ-MAT] Placement mirror set to {name}"));
            }

            "layer" => {
                let mut p = rt.placement.lock().unwrap();
                match args.first().copied() {
                    Some("all") => p.layer_mode = LayerMode::All,
                    Some("single") => {
                        let y = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                        p.layer_mode = LayerMode::Single(y);
                    }
                    Some("range") => {
                        let min_y = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                        let max_y = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(10);
                        p.layer_mode = LayerMode::Range(min_y, max_y);
                    }
                    _ => {
                        p.layer_mode.cycle(0, 64);
                    }
                }
                let desc = p.layer_mode.display_name();
                rt.set_toast(format!("Layer: {desc}"));
                send_chat(game, &format!("[HZ-MAT] Layer mode set to {desc}"));
            }

            "pos" | "position" => {
                let mut p = rt.placement.lock().unwrap();
                if args.first().copied() == Some("here") || args.is_empty() {
                    if let Some(pl) = game.player.as_ref() {
                        p.origin = [
                            pl.position.x.floor() as i32,
                            pl.position.y.floor() as i32,
                            pl.position.z.floor() as i32,
                        ];
                    }
                } else if args.len() >= 3 {
                    if let (Ok(x), Ok(y), Ok(z)) = (args[0].parse(), args[1].parse(), args[2].parse()) {
                        p.origin = [x, y, z];
                    }
                }
                let o = p.origin;
                rt.set_toast(format!("Origin: [{}, {}, {}]", o[0], o[1], o[2]));
                send_chat(game, &format!("[HZ-MAT] Origin placed at [{}, {}, {}]", o[0], o[1], o[2]));
            }

            "verify" | "verif" => {
                let mut v = rt.verifier.lock().unwrap();
                if let Some(arg) = args.first() {
                    match *arg {
                        "on" | "true" => v.active = true,
                        "off" | "false" => v.active = false,
                        _ => v.active = !v.active,
                    }
                } else {
                    v.active = !v.active;
                }
                let st = if v.active { "ON" } else { "OFF" };
                rt.set_toast(format!("Verifier: {st}"));
                send_chat(game, &format!("[HZ-MAT] Real-time verifier toggled {st}"));
            }

            "materials" | "mats" => {
                let s_guard = rt.active_schematic.lock().unwrap();
                let Some(schem) = s_guard.as_ref() else {
                    send_chat(game, "[HZ-MAT] No schematic loaded.");
                    return true;
                };
                let p = rt.placement.lock().unwrap();

                let report = MaterialReport::generate(schem, &p, |item_name| {
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

                send_chat(game, &format!("[HZ-MAT] Material Report for '{}':", schem.name));
                send_chat(game, &format!("Total Needed: {} | In Inventory: {} | Missing: {}", report.total_required, report.total_available, report.total_missing));
                for entry in report.entries.iter().take(6) {
                    let mark = if entry.missing == 0 { "✓" } else { "✗" };
                    send_chat(game, &format!(" {} {}: Need {}, Inv {}, Miss {}", mark, entry.display_name, entry.required, entry.available, entry.missing));
                }
            }

            "help" | _ => {
                send_chat(game, "§6--- HorizonMatica Commands ---§r");
                send_chat(game, "§e/hzm load <file>§r - Load schematic from storage");
                send_chat(game, "§e/hzm unload§r - Unload active schematic");
                send_chat(game, "§e/hzm list§r - List schematics on device");
                send_chat(game, "§e/hzm layer <all|single|range>§r - Set layer slice");
                send_chat(game, "§e/hzm rotate [0|90|180|270]§r - Rotate placement");
                send_chat(game, "§e/hzm mirror [none|x|z]§r - Mirror placement");
                send_chat(game, "§e/hzm pos [here|x y z]§r - Set placement position");
                send_chat(game, "§e/hzm verify [on|off]§r - Toggle block verifier");
                send_chat(game, "§e/hzm materials§r - Show required materials");
            }
        }

        true
    }
}

fn send_chat(game: &mut crate::game::Game<'_>, msg: &str) {
    game.chat.push(Text::literal(msg));
}
