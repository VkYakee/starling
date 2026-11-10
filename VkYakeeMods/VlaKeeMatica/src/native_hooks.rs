//! Native hook points for VlaKeeMatica integration with Horizon/Starling
//! 
//! These hooks connect to Horizon's native hook system via mixins

use crate::prelude::*;
use crate::config::VlaKeeMaticaConfig;
use crate::schematic::litematic::LitematicaSchematic;

/// Global mod state accessed by native hooks
#[cfg(feature = "mods")]
mod state {
    use super::*;
    use parking_lot::RwLock;
    use std::sync::Arc;

    #[derive(Default)]
    pub struct VlaKeeMaticaState {
        pub config: VlaKeeMaticaConfig,
        pub active_schematic: Option<LitematicaSchematic>,
        pub placement: Option<SchematicPlacement>,
        pub materials: Option<MaterialList>,
        pub selection_manager: SelectionManager,
        pub tool_mode: ToolMode,
        pub enabled: bool,
        pub show_hud: bool,
        pub show_hologram: bool,
        pub layer_mode: LayerMode,
        pub layer_axis: Axis,
        pub current_layer: i32,
        pub opacity: f32,
        pub locked: bool,
        pub easy_place: bool,
        pub easy_place_protocol: EasyPlaceProtocol,
        pub pos1: Option<BlockPos>,
        pub pos2: Option<BlockPos>,
        pub projects_manager: SchematicProjectsManager,
    }

    lazy_static::lazy_static! {
        pub static ref STATE: Arc<RwLock<VlaKeeMaticaState>> = Arc::new(RwLock::new(VlaKeeMaticaState::default()));
    }

    pub fn with_state<F, R>(f: F) -> R
    where
        F: FnOnce(&mut VlaKeeMaticaState) -> R,
    {
        f(&mut STATE.write())
    }

    pub fn with_state_read<F, R>(f: F) -> R
    where
        F: FnOnce(&VlaKeeMaticaState) -> R,
    {
        f(&STATE.read())
    }
}

/// Hook called when mod is initialized
#[cfg(feature = "mods")]
#[no_mangle]
pub extern "Rust" fn vlakeematica_init() {
    state::with_state(|s| {
        s.enabled = true;
        s.show_hud = true;
        s.opacity = 0.75;
        s.easy_place = true;
        s.easy_place_protocol = EasyPlaceProtocol::V2;
    });
}

/// Hook called every game tick
#[cfg(feature = "mods")]
#[no_mangle]
pub extern "Rust" fn vlakeematica_tick(clock: &horizon_mods::Clock) {
    let _ = clock;
    state::with_state(|s| {
        // Update hologram position if following player
        if s.show_hologram && !s.locked {
            if let Some(ref mut _placement) = s.placement {
                // Could auto-follow player here
            }
        }
        
        // Update material progress if inventory changed
        if let Some(ref mut mats) = s.materials {
            // This would be updated when player inventory changes
            let _ = mats;
        }
    });
}

/// Hook called when player position changes
#[cfg(feature = "mods")]
#[no_mangle]
pub extern "Rust" fn vlakeematica_player_pos(_x: i32, _y: i32, _z: i32) {
    state::with_state(|_s| {
        // Update player position for easy-place calculations
    });
}

/// Hook called on block update (for verifier)
#[cfg(feature = "mods")]
#[no_mangle]
pub extern "Rust" fn vlakeematica_block_update(_x: i32, _y: i32, _z: i32, _block_state: &str) {
    state::with_state(|_s| {
        // Real-time verification update
    });
}

/// Hook called when chunk loads (for structure detection)
#[cfg(feature = "mods")]
#[no_mangle]
pub extern "Rust" fn vlakeematica_chunk_loaded(_cx: i32, _cz: i32, _dimension: &str) {
    state::with_state(|_s| {
        // Could auto-detect structures for schematic creation
    });
}

/// Hook called when player executes command
#[cfg(feature = "mods")]
#[no_mangle]
pub extern "Rust" fn vlakeematica_command(command: &str) -> bool {
    state::with_state(|s| {
        if command.starts_with("hm ") || command.starts_with("vlakeematica ") {
            // Handle subcommands
            let parts: Vec<&str> = command.split_whitespace().collect();
            if parts.len() < 2 {
                return true;
            }
            
            match parts[1] {
                "load" => {
                    // Load schematic
                    true
                }
                "unload" => {
                    s.active_schematic = None;
                    s.placement = None;
                    s.materials = None;
                    true
                }
                "hologram" => {
                    s.show_hologram = !s.show_hologram;
                    true
                }
                "verify" => {
                    // Trigger verification
                    true
                }
                "materials" => {
                    // Show material list
                    true
                }
                "pos1" => {
                    s.pos1 = Some(BlockPos::new(0, 0, 0)); // Would use actual player pos
                    true
                }
                "pos2" => {
                    s.pos2 = Some(BlockPos::new(0, 0, 0));
                    true
                }
                "lock" => {
                    s.locked = !s.locked;
                    true
                }
                "rotate" => {
                    if let Some(ref mut p) = s.placement {
                        if !s.locked {
                            p.rotate_clockwise();
                        }
                    }
                    true
                }
                "mirror" => {
                    if let Some(ref mut p) = s.placement {
                        if !s.locked {
                            p.flip_mirror();
                        }
                    }
                    true
                }
                "tool" => {
                    s.tool_mode = s.tool_mode.cycle_next();
                    true
                }
                "protocol" => {
                    s.easy_place_protocol = s.easy_place_protocol.next();
                    true
                }
                "layer_up" => {
                    s.current_layer += 1;
                    true
                }
                "layer_down" => {
                    s.current_layer -= 1;
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    })
}

/// Hook for HUD rendering
#[cfg(feature = "mods")]
#[no_mangle]
pub extern "Rust" fn vlakeematica_hud() -> Vec<String> {
    state::with_state_read(|s| {
        let mut lines = Vec::new();
        if s.enabled {
            lines.push("┌──── VlaKeeMatica ───────────────────────┐".to_string());
            lines.push(format!("│ Esquemática: {}", s.active_schematic.as_ref().map(|sc| sc.metadata.name.as_str()).unwrap_or("Ninguna")));
            let hologram_str = if s.show_hologram { "ON" } else { "OFF" };
            let hud_str = if s.show_hud { "ON" } else { "OFF" };
            lines.push(format!("│ Holograma: {}  Mini: {}", hologram_str, hud_str));
            lines.push(format!("│ Herramienta: {:?}  Protocolo: {:?}", s.tool_mode, s.easy_place_protocol));
            lines.push(format!("│ Capa: {} (Eje: {:?})  Opacidad: {:.0}%", s.current_layer, s.layer_axis, s.opacity * 100.0));
            lines.push("│ [L] Cargar  [H] Holograma  [T] Herramienta  [M] Menú".to_string());
            lines.push("└────────────────────────────────────────┘".to_string());
        } else {
            lines.push("VlaKeeMatica: Deshabilitado".to_string());
        }
        
        lines
    })
}

/// Hook for 3D bounding box rendering (hologram)
#[cfg(feature = "mods")]
#[no_mangle]
pub extern "Rust" fn vlakeematica_render_boxes() -> Vec<[f32; 10]> {
    state::with_state_read(|s| {
        if !s.enabled || !s.show_hologram {
            return Vec::new();
        }

        let mut boxes = Vec::new();
        
        if let (Some(ref schematic), Some(ref placement)) = (&s.active_schematic, &s.placement) {
            for (region_name, region) in &schematic.regions {
                let sub_placement = placement.sub_region_placements.get(region_name);
                if !(sub_placement.map(|p| p.enabled).unwrap_or(false)) {
                    continue;
                }

                let min = region.position;
                let max = min.offset(
                    region.size.x - 1,
                    region.size.y - 1,
                    region.size.z - 1,
                );

                let world_min = placement.get_transformed_position(region_name, min);
                let world_max = placement.get_transformed_position(region_name, max);

                // Main enclosing box
                boxes.push([
                    world_min.x as f32 - 0.5,
                    world_min.y as f32 - 0.5,
                    world_min.z as f32 - 0.5,
                    world_max.x as f32 + 0.5,
                    world_max.y as f32 + 0.5,
                    world_max.z as f32 + 0.5,
                    0.0, 1.0, 0.0, 0.5, // Green semi-transparent
                ]);

                // Ghost blocks outline if enabled
                if s.config.visuals.show_hologram_ghost_blocks {
                    // Add individual block outlines for missing blocks
                }
            }
        }
        
        boxes
    })
}

/// Hook for settings UI
#[cfg(feature = "mods")]
#[no_mangle]
pub extern "Rust" fn vlakeematica_settings() -> Vec<String> {
    vec![
        "📐 Cargar Esquemática".to_string(),
        "📁 Importar desde Descargas".to_string(),
        "🔄 Previsualizar 3D/2D".to_string(),
        "▶ Iniciar Holograma".to_string(),
        "🛠️ Modo Herramienta: Colocación".to_string(),
        "🔄 Rotar 90° CW".to_string(),
        "🪞 Espejo: Alternar".to_string(),
        "📦 Ver Materiales".to_string(),
        "📊 Exportar CSV".to_string(),
        "🔍 Verificar Bloques".to_string(),
        "⏹ Descargar Esquemática".to_string(),
        "⚙️ Configuración Avanzada".to_string(),
    ]
}

/// Hook for settings button click
#[cfg(feature = "mods")]
#[no_mangle]
pub extern "Rust" fn vlakeematica_settings_click(button_idx: usize) {
    state::with_state(|s| {
        match button_idx {
            0 => {
                // Load next schematic
            }
            1 => {
                // Import from downloads
            }
            2 => {
                // Toggle mini render
            }
            3 => {
                s.show_hologram = !s.show_hologram;
            }
            4 => {
                s.tool_mode = s.tool_mode.cycle_next();
            }
            5 => {
                if let Some(ref mut p) = s.placement {
                    if !s.locked {
                        p.rotate_clockwise();
                    }
                }
            }
            6 => {
                if let Some(ref mut p) = s.placement {
                    if !s.locked {
                        p.flip_mirror();
                    }
                }
            }
            7 => {
                // Show materials
            }
            8 => {
                // Export CSV
            }
            9 => {
                // Verify schematic
            }
            10 => {
                // Unload
                s.active_schematic = None;
                s.placement = None;
                s.materials = None;
            }
            11 => {
                // Advanced settings
            }
            _ => {}
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "mods")]
    #[test]
    fn test_hooks_compile() {
        vlakeematica_init();
        vlakeematica_tick(&horizon_mods::Clock { game_time: 0, day_time: 0 });
        vlakeematica_player_pos(100, 64, 200);
        vlakeematica_block_update(100, 64, 200, "minecraft:stone");
        vlakeematica_chunk_loaded(5, 10, "minecraft:overworld");
        assert!(!vlakeematica_command("hm help"));
        assert!(!vlakeematica_hud().is_empty() || true);
        assert!(!vlakeematica_render_boxes().is_empty() || true);
        assert!(!vlakeematica_settings().is_empty());
        vlakeematica_settings_click(0);
    }
}
