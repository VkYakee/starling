//! Fixed MIT-licensed Android mod bundle. No downloaded code or phone compiler.

use super::*;
use horizon_client::extras_hud::Corners;

use super::native_hooks::*;

#[cfg(target_os = "android")]
#[path = "android_bundled/zoom/runtime.rs"]
pub(super) mod official_zoom;

#[cfg(target_os = "android")]
#[path = "android_bundled/zoom/zoom_key.rs"]
mod zoom_key;

#[cfg(target_os = "android")]
#[path = "android_bundled/zoom/zoom_wheel.rs"]
mod zoom_wheel;

#[cfg(target_os = "android")]
#[path = "android_bundled/zoom/zoom_zoom.rs"]
mod zoom_zoom;

#[cfg(target_os = "android")]
#[path = "android_bundled/zoom/zoom_list.rs"]
mod zoom_list;

#[cfg(target_os = "android")]
#[path = "android_bundled/zoom/zoom_settings.rs"]
mod zoom_settings;

#[cfg(target_os = "android")]
#[path = "android_bundled/zoom/zoom_click.rs"]
mod zoom_click;

#[cfg(target_os = "android")]
#[path = "android_bundled/minimap/runtime.rs"]
pub(super) mod official_minimap;

#[cfg(target_os = "android")]
#[path = "android_bundled/minimap/minimap_list.rs"]
mod minimap_list;

#[cfg(target_os = "android")]
#[path = "android_bundled/minimap/minimap_settings.rs"]
mod minimap_settings;

#[cfg(target_os = "android")]
#[path = "android_bundled/minimap/minimap_click.rs"]
mod minimap_click;

#[cfg(target_os = "android")]
#[path = "android_bundled/minimap/minimap_minimap.rs"]
mod minimap_minimap;

#[cfg(target_os = "android")]
#[path = "android_bundled/recipes/runtime.rs"]
pub(super) mod official_recipes;

#[cfg(target_os = "android")]
#[path = "android_bundled/recipes/glue.rs"]
pub(super) mod official_recipes_glue;

#[cfg(target_os = "android")]
#[path = "android_bundled/recipes/recipes_list.rs"]
mod recipes_list;

#[cfg(target_os = "android")]
#[path = "android_bundled/recipes/recipes_settings.rs"]
mod recipes_settings;

#[cfg(target_os = "android")]
#[path = "android_bundled/recipes/recipes_click.rs"]
mod recipes_click;

#[cfg(target_os = "android")]
#[path = "android_bundled/recipes/recipes_frame.rs"]
mod recipes_frame;

#[cfg(target_os = "android")]
#[path = "android_bundled/recipes/recipes_event.rs"]
mod recipes_event;

#[cfg(target_os = "android")]
#[path = "android_bundled/smartf3/runtime.rs"]
pub(super) mod smartf3;

#[cfg(target_os = "android")]
#[path = "android_bundled/smartf3/native.rs"]
pub(super) mod smartf3_native;

#[cfg(target_os = "android")]
#[path = "android_bundled/smartf3/smartf3_command.rs"]
mod smartf3_command;

#[cfg(target_os = "android")]
#[path = "android_bundled/smartf3/smartf3_list.rs"]
mod smartf3_list;

#[cfg(target_os = "android")]
#[path = "android_bundled/smartf3/smartf3_settings.rs"]
mod smartf3_settings;

#[cfg(target_os = "android")]
#[path = "android_bundled/smartf3/smartf3_click.rs"]
mod smartf3_click;

#[cfg(target_os = "android")]
#[path = "android_bundled/VkYakeeMods/horizon_voxel_cracker/runtime.rs"]
pub(super) mod horizon_voxel_cracker;

#[cfg(target_os = "android")]
#[path = "android_bundled/VkYakeeMods/horizon_voxel_cracker/adapters/hvc_list.rs"]
mod hvc_list;

#[cfg(target_os = "android")]
#[path = "android_bundled/VkYakeeMods/horizon_voxel_cracker/adapters/hvc_settings.rs"]
mod hvc_settings;

#[cfg(target_os = "android")]
#[path = "android_bundled/VkYakeeMods/horizon_voxel_cracker/adapters/hvc_click.rs"]
mod hvc_click;

#[cfg(target_os = "android")]
#[path = "android_bundled/VkYakeeMods/horizon_voxel_cracker/adapters/hvc_command.rs"]
mod hvc_command;

#[cfg(target_os = "android")]
#[path = "android_bundled/VkYakeeMods/horizon_voxel_cracker/adapters/hvc_key.rs"]
mod hvc_key;

#[cfg(target_os = "android")]
#[path = "android_bundled/VkYakeeMods/horizon_voxel_cracker/adapters/hvc_hud.rs"]
mod hvc_hud;

#[cfg(target_os = "android")]
#[path = "android_bundled/VkYakeeMods/horizon_voxel_cracker/adapters/hvc_tick.rs"]
mod hvc_tick;

#[cfg(target_os = "android")]
#[path = "android_bundled/VkYakeeMods/horizonmatica/runtime.rs"]
pub(super) mod horizonmatica;

#[cfg(target_os = "android")]
#[path = "android_bundled/VkYakeeMods/horizonmatica/adapters/hzm_list.rs"]
mod hzm_list;

#[cfg(target_os = "android")]
#[path = "android_bundled/VkYakeeMods/horizonmatica/adapters/hzm_settings.rs"]
mod hzm_settings;

#[cfg(target_os = "android")]
#[path = "android_bundled/VkYakeeMods/horizonmatica/adapters/hzm_click.rs"]
mod hzm_click;

#[cfg(target_os = "android")]
#[path = "android_bundled/VkYakeeMods/horizonmatica/adapters/hzm_command.rs"]
mod hzm_command;

#[cfg(target_os = "android")]
#[path = "android_bundled/VkYakeeMods/horizonmatica/adapters/hzm_key.rs"]
mod hzm_key;

#[cfg(target_os = "android")]
#[path = "android_bundled/VkYakeeMods/horizonmatica/adapters/hzm_hud.rs"]
mod hzm_hud;

#[cfg(target_os = "android")]
#[path = "android_bundled/VkYakeeMods/horizonmatica/adapters/hzm_tick.rs"]
mod hzm_tick;

#[cfg(target_os = "android")]
#[path = "android_bundled/VkYakeeMods/horizonmatica/adapters/hzm_visuals.rs"]
mod hzm_visuals;

pub(super) fn native_key(game: &mut Game<'_>, key: &str, pressed: bool) {
    let _ = (&game, &key, &pressed);
    #[cfg(target_os = "android")]
    {
        zoom_key::run(&mut *game, key, pressed);
        hvc_key::run(&mut *game, key, pressed);
        hzm_key::run(&mut *game, key, pressed);
    }
}

pub(super) fn native_wheel(game: &mut Game<'_>, steps: f32, claimed: &mut bool) {
    let _ = (&game, &steps, &claimed);
    #[cfg(target_os = "android")]
    {
        zoom_wheel::run(&*game, steps, &mut *claimed);
    }
}

pub(super) fn native_zoom(game: &Game<'_>, zoom: &mut Option<NativeZoom>) {
    let _ = (&game, &zoom);
    #[cfg(target_os = "android")]
    {
        zoom_zoom::run(game, &mut *zoom);
    }
}

pub(super) fn native_mods(game: &mut Game<'_>, mods: &mut Vec<NativeMod>) {
    let _ = (&game, &mods);
    #[cfg(target_os = "android")]
    {
        zoom_list::run(&mut *mods);
        minimap_list::run(&mut *mods);
        recipes_list::run(&mut *mods);
        smartf3_list::run(&mut *mods);
        hvc_list::run(&mut *mods);
        hzm_list::run(&mut *mods);
    }
}

pub(super) fn native_settings(game: &mut Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    let _ = (&game, &mod_id, &buttons);
    #[cfg(target_os = "android")]
    {
        zoom_settings::run(&mut *game, mod_id, &mut *buttons);
        minimap_settings::run(&mut *game, mod_id, &mut *buttons);
        recipes_settings::run(&mut *game, mod_id, &mut *buttons);
        smartf3_settings::run(&mut *game, mod_id, &mut *buttons);
        hvc_settings::run(&mut *game, mod_id, &mut *buttons);
        hzm_settings::run(&mut *game, mod_id, &mut *buttons);
    }
}

pub(super) fn native_settings_click(game: &mut Game<'_>, mod_id: &str, button: usize) {
    let _ = (&game, &mod_id, &button);
    #[cfg(target_os = "android")]
    {
        zoom_click::run(&mut *game, mod_id, button);
        minimap_click::run(&mut *game, mod_id, button);
        recipes_click::run(&mut *game, mod_id, button);
        smartf3_click::run(&mut *game, mod_id, button);
        hvc_click::run(&mut *game, mod_id, button);
        hzm_click::run(&mut *game, mod_id, button);
    }
}

pub(super) fn native_minimap(game: &Game<'_>, minimap: &mut Option<NativeMinimap>) {
    let _ = (&game, &minimap);
    #[cfg(target_os = "android")]
    {
        minimap_minimap::run(game, &mut *minimap);
    }
}

pub(super) fn native_frame(game: &mut Game<'_>, elapsed: Duration) {
    let _ = (&game, &elapsed);
    #[cfg(target_os = "android")]
    {
        recipes_frame::run(&mut *game);
    }
}

pub(super) fn native_panel_event(game: &mut Game<'_>, mod_id: &str, event: &NativePanelEvent) {
    let _ = (&game, &mod_id, &event);
    #[cfg(target_os = "android")]
    {
        recipes_event::run(&mut *game, mod_id, event);
    }
}

pub(super) fn native_command(game: &mut Game<'_>, command: &str, ran: &mut bool) {
    let _ = (&game, &command, &ran);
    #[cfg(target_os = "android")]
    {
        smartf3_command::run(&mut *game, command, &mut *ran);
        hvc_command::run(&mut *game, command, &mut *ran);
        hzm_command::run(&mut *game, command, &mut *ran);
    }
}

pub(super) fn native_commands(game: &mut Game<'_>, commands: &mut Vec<&'static str>) {
    let _ = (&game, &commands);
    #[cfg(target_os = "android")]
    {
        commands.push("hvc");
        commands.push("horizonvoxelcracker");
        commands.push("seedcracker");
        commands.push("horizonmatica");
        commands.push("hzm");
        commands.push("litematica");
    }
}

pub(super) fn native_hud(game: &mut Game<'_>, batch: &mut HudBatch, corners: &mut Corners) {
    let _ = (&game, &batch, &corners);
    #[cfg(target_os = "android")]
    {
        hvc_hud::run(&mut *game, batch, corners);
        hzm_hud::run(&mut *game, batch, corners);
    }
}

pub(super) fn native_tick(game: &mut Game<'_>) {
    let _ = &game;
    #[cfg(target_os = "android")]
    {
        hvc_tick::run(&mut *game);
        hzm_tick::run(&mut *game);
    }
}

pub(super) fn native_visuals(
    game: &Game<'_>,
    rims: &mut Vec<NativeBlockRim>,
    lights: &mut Vec<NativeColoredLight>,
) {
    let _ = (&game, &rims, &lights);
    #[cfg(target_os = "android")]
    {
        hzm_visuals::run(&*game, rims, lights);
    }
}

#[cfg(target_os = "android")]
pub(super) fn reset() {
    official_zoom::reset();
    official_minimap::reset();
    official_recipes::set_viewer(None);
    smartf3_native::reset();
    horizon_voxel_cracker::reset();
    horizonmatica::reset();
}
