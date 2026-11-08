use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// RGBA color representation for block faces (Dynmap style).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockFaceColor {
    pub top: [u8; 4],
    pub north: [u8; 4],
    pub east: [u8; 4],
    pub south: [u8; 4],
}

impl Default for BlockFaceColor {
    fn default() -> Self {
        Self {
            top: [128, 128, 128, 255],
            north: [100, 100, 100, 255],
            east: [80, 80, 80, 255],
            south: [60, 60, 60, 255],
        }
    }
}

impl BlockFaceColor {
    pub fn uniform(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            top: [r, g, b, a],
            north: [(r as f32 * 0.8) as u8, (g as f32 * 0.8) as u8, (b as f32 * 0.8) as u8, a],
            east: [(r as f32 * 0.6) as u8, (g as f32 * 0.6) as u8, (b as f32 * 0.6) as u8, a],
            south: [(r as f32 * 0.5) as u8, (g as f32 * 0.5) as u8, (b as f32 * 0.5) as u8, a],
        }
    }
}

/// Comprehensive Minecraft Block Color Palette (Dynmap & Squaremap standard).
#[derive(Debug, Clone)]
pub struct BlockPalette {
    colors: HashMap<String, BlockFaceColor>,
    fallback: BlockFaceColor,
}

impl Default for BlockPalette {
    fn default() -> Self {
        Self::new_with_defaults()
    }
}

impl BlockPalette {
    /// Creates an empty block palette with default fallback (magenta/stone).
    pub fn new() -> Self {
        Self {
            colors: HashMap::new(),
            fallback: BlockFaceColor::uniform(128, 128, 128, 255),
        }
    }

    /// Initializes a block palette pre-populated with standard vanilla Minecraft block colors.
    pub fn new_with_defaults() -> Self {
        let mut palette = Self::new();
        palette.register_vanilla_defaults();
        palette
    }

    /// Registers standard vanilla block definitions.
    pub fn register_vanilla_defaults(&mut self) {
        self.register("stone", BlockFaceColor::uniform(125, 125, 125, 255));
        self.register("granite", BlockFaceColor::uniform(149, 103, 85, 255));
        self.register("polished_granite", BlockFaceColor::uniform(154, 106, 89, 255));
        self.register("diorite", BlockFaceColor::uniform(188, 188, 188, 255));
        self.register("polished_diorite", BlockFaceColor::uniform(192, 193, 194, 255));
        self.register("andesite", BlockFaceColor::uniform(136, 136, 136, 255));
        self.register("polished_andesite", BlockFaceColor::uniform(132, 134, 133, 255));
        self.register("grass_block", BlockFaceColor::uniform(90, 140, 60, 255));
        self.register("dirt", BlockFaceColor::uniform(134, 96, 67, 255));
        self.register("coarse_dirt", BlockFaceColor::uniform(119, 85, 59, 255));
        self.register("podzol", BlockFaceColor::uniform(91, 63, 24, 255));
        self.register("cobblestone", BlockFaceColor::uniform(127, 127, 127, 255));
        self.register("oak_planks", BlockFaceColor::uniform(162, 130, 78, 255));
        self.register("spruce_planks", BlockFaceColor::uniform(114, 84, 48, 255));
        self.register("birch_planks", BlockFaceColor::uniform(192, 175, 121, 255));
        self.register("jungle_planks", BlockFaceColor::uniform(160, 115, 80, 255));
        self.register("acacia_planks", BlockFaceColor::uniform(168, 90, 50, 255));
        self.register("dark_oak_planks", BlockFaceColor::uniform(66, 43, 20, 255));
        self.register("mangrove_planks", BlockFaceColor::uniform(118, 54, 49, 255));
        self.register("cherry_planks", BlockFaceColor::uniform(226, 178, 174, 255));
        self.register("bedrock", BlockFaceColor::uniform(85, 85, 85, 255));
        self.register("water", BlockFaceColor::uniform(64, 100, 200, 180));
        self.register("lava", BlockFaceColor::uniform(230, 100, 20, 255));
        self.register("sand", BlockFaceColor::uniform(219, 207, 159, 255));
        self.register("red_sand", BlockFaceColor::uniform(178, 97, 36, 255));
        self.register("gravel", BlockFaceColor::uniform(131, 127, 126, 255));
        self.register("gold_ore", BlockFaceColor::uniform(143, 139, 125, 255));
        self.register("deepslate_gold_ore", BlockFaceColor::uniform(112, 108, 98, 255));
        self.register("iron_ore", BlockFaceColor::uniform(135, 130, 126, 255));
        self.register("deepslate_iron_ore", BlockFaceColor::uniform(105, 102, 99, 255));
        self.register("coal_ore", BlockFaceColor::uniform(115, 115, 115, 255));
        self.register("deepslate_coal_ore", BlockFaceColor::uniform(85, 85, 85, 255));
        self.register("nether_gold_ore", BlockFaceColor::uniform(140, 80, 75, 255));
        self.register("oak_log", BlockFaceColor::uniform(107, 85, 52, 255));
        self.register("spruce_log", BlockFaceColor::uniform(59, 39, 22, 255));
        self.register("birch_log", BlockFaceColor::uniform(214, 214, 211, 255));
        self.register("jungle_log", BlockFaceColor::uniform(85, 67, 25, 255));
        self.register("acacia_log", BlockFaceColor::uniform(107, 98, 89, 255));
        self.register("dark_oak_log", BlockFaceColor::uniform(58, 44, 28, 255));
        self.register("oak_leaves", BlockFaceColor::uniform(75, 120, 45, 255));
        self.register("spruce_leaves", BlockFaceColor::uniform(55, 95, 55, 255));
        self.register("birch_leaves", BlockFaceColor::uniform(110, 155, 60, 255));
        self.register("jungle_leaves", BlockFaceColor::uniform(65, 135, 40, 255));
        self.register("acacia_leaves", BlockFaceColor::uniform(90, 130, 50, 255));
        self.register("dark_oak_leaves", BlockFaceColor::uniform(45, 90, 30, 255));
        self.register("glass", BlockFaceColor::uniform(200, 220, 230, 64));
        self.register("lapis_ore", BlockFaceColor::uniform(102, 112, 134, 255));
        self.register("sandstone", BlockFaceColor::uniform(216, 203, 155, 255));
        self.register("wool", BlockFaceColor::uniform(222, 222, 222, 255));
        self.register("white_wool", BlockFaceColor::uniform(233, 236, 236, 255));
        self.register("gold_block", BlockFaceColor::uniform(245, 205, 47, 255));
        self.register("iron_block", BlockFaceColor::uniform(216, 216, 216, 255));
        self.register("brick", BlockFaceColor::uniform(150, 97, 83, 255));
        self.register("bricks", BlockFaceColor::uniform(150, 97, 83, 255));
        self.register("tnt", BlockFaceColor::uniform(218, 59, 44, 255));
        self.register("bookshelf", BlockFaceColor::uniform(162, 130, 78, 255));
        self.register("mossy_cobblestone", BlockFaceColor::uniform(108, 119, 94, 255));
        self.register("obsidian", BlockFaceColor::uniform(20, 18, 29, 255));
        self.register("torch", BlockFaceColor::uniform(255, 210, 100, 255));
        self.register("diamond_ore", BlockFaceColor::uniform(129, 140, 143, 255));
        self.register("deepslate_diamond_ore", BlockFaceColor::uniform(90, 110, 115, 255));
        self.register("diamond_block", BlockFaceColor::uniform(94, 215, 210, 255));
        self.register("crafting_table", BlockFaceColor::uniform(162, 130, 78, 255));
        self.register("farmland", BlockFaceColor::uniform(88, 56, 33, 255));
        self.register("furnace", BlockFaceColor::uniform(115, 115, 115, 255));
        self.register("snow", BlockFaceColor::uniform(240, 245, 245, 255));
        self.register("snow_block", BlockFaceColor::uniform(240, 245, 245, 255));
        self.register("ice", BlockFaceColor::uniform(144, 186, 242, 200));
        self.register("packed_ice", BlockFaceColor::uniform(141, 180, 242, 255));
        self.register("blue_ice", BlockFaceColor::uniform(116, 167, 253, 255));
        self.register("clay", BlockFaceColor::uniform(160, 166, 179, 255));
        self.register("netherrack", BlockFaceColor::uniform(111, 48, 48, 255));
        self.register("soul_sand", BlockFaceColor::uniform(81, 62, 50, 255));
        self.register("soul_soil", BlockFaceColor::uniform(75, 56, 45, 255));
        self.register("glowstone", BlockFaceColor::uniform(239, 193, 126, 255));
        self.register("nether_bricks", BlockFaceColor::uniform(44, 21, 26, 255));
        self.register("end_stone", BlockFaceColor::uniform(221, 223, 165, 255));
        self.register("emerald_ore", BlockFaceColor::uniform(120, 137, 125, 255));
        self.register("emerald_block", BlockFaceColor::uniform(42, 203, 87, 255));
        self.register("deepslate", BlockFaceColor::uniform(79, 79, 82, 255));
        self.register("cobbled_deepslate", BlockFaceColor::uniform(77, 77, 80, 255));
        self.register("copper_ore", BlockFaceColor::uniform(125, 125, 120, 255));
        self.register("raw_copper_block", BlockFaceColor::uniform(154, 105, 79, 255));
        self.register("copper_block", BlockFaceColor::uniform(192, 107, 79, 255));
        self.register("ancient_debris", BlockFaceColor::uniform(95, 68, 62, 255));
        self.register("netherite_block", BlockFaceColor::uniform(66, 61, 63, 255));
        self.register("sculk", BlockFaceColor::uniform(13, 30, 36, 255));
        self.register("mud", BlockFaceColor::uniform(60, 57, 60, 255));
        self.register("mangrove_roots", BlockFaceColor::uniform(78, 59, 44, 255));
    }

    /// Registers a block with uniform or directional color.
    pub fn register(&mut self, block: &str, color: BlockFaceColor) {
        let key = normalize_block_key(block);
        self.colors.insert(key, color);
    }

    /// Loads and parses a Dynmap colorscheme file (`default.txt` / `standard.txt`).
    pub fn load_from_dynmap_file<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<usize> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 5 {
                let block_id = parts[0];
                if let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
                    parts[1].parse::<u8>(),
                    parts[2].parse::<u8>(),
                    parts[3].parse::<u8>(),
                    parts[4].parse::<u8>(),
                ) {
                    let mut face_color = BlockFaceColor::uniform(r, g, b, a);
                    if parts.len() >= 17 {
                        // All 4 faces provided
                        if let (Ok(nr), Ok(ng), Ok(nb), Ok(na)) = (
                            parts[5].parse::<u8>(),
                            parts[6].parse::<u8>(),
                            parts[7].parse::<u8>(),
                            parts[8].parse::<u8>(),
                        ) {
                            face_color.north = [nr, ng, nb, na];
                        }
                        if let (Ok(er), Ok(eg), Ok(eb), Ok(ea)) = (
                            parts[9].parse::<u8>(),
                            parts[10].parse::<u8>(),
                            parts[11].parse::<u8>(),
                            parts[12].parse::<u8>(),
                        ) {
                            face_color.east = [er, eg, eb, ea];
                        }
                        if let (Ok(sr), Ok(sg), Ok(sb), Ok(sa)) = (
                            parts[13].parse::<u8>(),
                            parts[14].parse::<u8>(),
                            parts[15].parse::<u8>(),
                            parts[16].parse::<u8>(),
                        ) {
                            face_color.south = [sr, sg, sb, sa];
                        }
                    }
                    self.register(block_id, face_color);
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Resolves the color of a block name (handles namespace and block states gracefully).
    pub fn get_color(&self, block_name: &str) -> BlockFaceColor {
        let key = normalize_block_key(block_name);
        if let Some(c) = self.colors.get(&key) {
            return *c;
        }

        // Try stripping state properties: "grass_block[snowy=false]" -> "grass_block"
        if let Some(bracket_idx) = key.find('[') {
            let base_key = &key[..bracket_idx];
            if let Some(c) = self.colors.get(base_key) {
                return *c;
            }
        }

        self.fallback
    }

    /// Quick top-face RGBA query.
    pub fn get_top_rgba(&self, block_name: &str) -> [u8; 4] {
        self.get_color(block_name).top
    }

    pub fn len(&self) -> usize {
        self.colors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }
}

/// Normalizes block names (lowercases and removes "minecraft:" prefix if present for clean lookup).
pub fn normalize_block_key(raw: &str) -> String {
    let lower = raw.trim().to_ascii_lowercase();
    if let Some(stripped) = lower.strip_prefix("minecraft:") {
        stripped.to_string()
    } else {
        lower
    }
}

/// Alpha blending between two RGBA colors.
pub fn blend_rgba(fg: [u8; 4], bg: [u8; 4], alpha: f32) -> [u8; 4] {
    let a = alpha.clamp(0.0, 1.0);
    let inv_a = 1.0 - a;
    [
        ((fg[0] as f32 * a) + (bg[0] as f32 * inv_a)) as u8,
        ((fg[1] as f32 * a) + (bg[1] as f32 * inv_a)) as u8,
        ((fg[2] as f32 * a) + (bg[2] as f32 * inv_a)) as u8,
        255,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vanilla_defaults() {
        let palette = BlockPalette::new_with_defaults();
        assert!(palette.len() > 30);

        let stone = palette.get_top_rgba("minecraft:stone");
        assert_eq!(stone[0], 125);
        assert_eq!(stone[1], 125);
        assert_eq!(stone[2], 125);

        let grass_state = palette.get_top_rgba("grass_block[snowy=false]");
        assert_eq!(grass_state[0], 90);
    }

    #[test]
    fn test_fallback_unregistered() {
        let palette = BlockPalette::new();
        let fallback = palette.get_top_rgba("mysterious_mod:quantum_block");
        assert_eq!(fallback, [128, 128, 128, 255]);
    }
}
