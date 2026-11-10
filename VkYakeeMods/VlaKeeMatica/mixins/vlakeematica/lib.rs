//! VlaKeeMatica mixins for Horizon/Starling
//! 
//! This file declares the native hook entry points that Horizon's mixin system
//! will call into from the game engine.

#[cfg(feature = "mods")]
use vlakeematica::{
    vlakeematica_init,
    vlakeematica_tick,
    vlakeematica_player_pos,
    vlakeematica_block_update,
    vlakeematica_chunk_loaded,
    vlakeematica_command,
    vlakeematica_hud,
    vlakeematica_render_boxes,
    vlakeematica_settings,
    vlakeematica_settings_click,
};

// Re-export hook functions for the mixin system
#[cfg(feature = "mods")]
pub use vlakeematica::{
    vlakeematica_init as init,
    vlakeematica_tick as on_tick,
    vlakeematica_player_pos as on_player_pos,
    vlakeematica_block_update as on_block_update,
    vlakeematica_chunk_loaded as on_chunk_loaded,
    vlakeematica_command as on_command,
    vlakeematica_hud as hud_lines,
    vlakeematica_render_boxes as render_bounding_boxes,
    vlakeematica_settings as settings_buttons,
    vlakeematica_settings_click as on_settings_click,
};

/// Entry point for the native mod
#[cfg(feature = "mods")]
#[no_mangle]
pub extern "Rust" fn vlakeematica_load() {
    init();
}

/// Mod metadata for the launcher
#[cfg(feature = "mods")]
#[no_mangle]
pub extern "Rust" fn vlakeematica_metadata() -> &'static str {
    r#"{
        "id": "vlakeematica",
        "name": "VlaKeeMatica",
        "version": "1.0.0",
        "description": "Litematica 1:1 rewrite for Starling/Horizon",
        "author": "VkYakee (original: Masa)",
        "capabilities": [
            "game-read",
            "chat",
            "hud",
            "storage",
            "command",
            "ui",
            "render",
            "input"
        ]
    }"#
}
