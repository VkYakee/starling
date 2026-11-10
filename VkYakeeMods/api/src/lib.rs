//! VkMod API: Unified trait interface for all mods inside VkYakeeMods.

#[derive(Clone, Debug)]
pub struct ModMetadata {
    pub id: &'static str,
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub author: &'static str,
}

pub trait VkMod: Send + Sync {
    fn metadata(&self) -> ModMetadata;

    /// Called when the player receives a new Hashed Seed from the server (Login / Respawn).
    fn on_hashed_seed(&mut self, hashed_seed: u64) {
        let _ = hashed_seed;
    }

    /// Called on every client game tick.
    fn on_tick(&mut self) {}

    /// Intercepts client-side commands (/seedcracker, /seed, etc.).
    /// Returns true if the command was claimed and handled.
    fn on_command(&mut self, command: &str) -> bool {
        let _ = command;
        false
    }

    /// Called when a chunk is loaded and parsed.
    /// `bedrock_mask`: optional 16x16 boolean bitmask (256 bytes) of bedrock blocks at key Y layers.
    fn on_chunk_loaded(
        &mut self,
        cx: i32,
        cz: i32,
        dimension: &str,
        bedrock_mask: Option<&[u8; 256]>,
    ) {
        let _ = (cx, cz, dimension, bedrock_mask);
    }

    /// Supplies a decoded chunk palette to mods that perform block-pattern analysis.
    fn on_block_snapshot(&mut self, chunk_x:i32, chunk_z:i32, dimension:&str, blocks:&[(i32,i32,i32,String)]) {
        let _=(chunk_x,chunk_z,dimension,blocks);
    }

    /// Incremental block-state mutation (including block removal).
    fn on_block_update(&mut self, x:i32, y:i32, z:i32, block:Option<&str>) { let _=(x,y,z,block); }

    /// Called when the player steps into or discovers a new biome.
    fn on_biome_found(&mut self, x: i32, z: i32, biome_id: i32, name: &str) {
        let _ = (x, z, biome_id, name);
    }

    /// Called on each tick with current player coordinates.
    fn on_player_pos(&mut self, x: i32, y: i32, z: i32) {
        let _ = (x, y, z);
    }

    /// Called when a structure or structure piece is detected.
    fn on_structure_found(&mut self, st_type: &str, chunk_x: i32, chunk_z: i32) {
        let _ = (st_type, chunk_x, chunk_z);
    }

    /// Returns informational lines to render on the client HUD.
    fn hud_lines(&self) -> Vec<String> {
        Vec::new()
    }

    /// Returns 3D bounding boxes to render in the world (e.g. structure ESP).
    /// Format: [min_x, min_y, min_z, max_x, max_y, max_z, r, g, b, a]
    fn render_bounding_boxes(&self) -> Vec<[f32; 10]> {
        Vec::new()
    }

    /// Buttons to present on Options > Mods > [Mod Name].
    fn settings_buttons(&self) -> Vec<String> {
        Vec::new()
    }

    /// Called when a settings button is clicked.
    fn on_settings_click(&mut self, button_idx: usize) {
        let _ = button_idx;
    }
}
