use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Biome color information for grass, foliage, and water tinting (VoxelMap & Dynmap standards).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BiomeColorEntry {
    pub grass_rgb: [u8; 3],
    pub foliage_rgb: [u8; 3],
    pub water_rgb: [u8; 3],
}

impl Default for BiomeColorEntry {
    fn default() -> Self {
        Self {
            grass_rgb: [124, 180, 70],
            foliage_rgb: [90, 160, 50],
            water_rgb: [63, 118, 228],
        }
    }
}

/// Comprehensive Biome Color and Environmental Tinting Registry.
#[derive(Debug, Clone)]
pub struct BiomePalette {
    biomes: HashMap<String, BiomeColorEntry>,
    fallback: BiomeColorEntry,
}

impl Default for BiomePalette {
    fn default() -> Self {
        Self::new_with_defaults()
    }
}

impl BiomePalette {
    pub fn new() -> Self {
        Self {
            biomes: HashMap::new(),
            fallback: BiomeColorEntry::default(),
        }
    }

    /// Creates and registers standard vanilla Minecraft biomes.
    pub fn new_with_defaults() -> Self {
        let mut palette = Self::new();
        palette.register_vanilla_defaults();
        palette
    }

    pub fn register_vanilla_defaults(&mut self) {
        self.register("plains", [145, 189, 89], [119, 171, 47], [63, 118, 228]);
        self.register("sunflower_plains", [145, 189, 89], [119, 171, 47], [63, 118, 228]);
        self.register("snowy_plains", [128, 180, 151], [96, 161, 122], [61, 87, 214]);
        self.register("desert", [191, 178, 77], [174, 164, 42], [50, 99, 219]);
        self.register("swamp", [106, 112, 57], [106, 112, 57], [97, 123, 100]);
        self.register("mangrove_swamp", [141, 161, 57], [141, 161, 57], [58, 122, 107]);
        self.register("forest", [121, 192, 90], [89, 176, 50], [63, 118, 228]);
        self.register("flower_forest", [121, 192, 90], [89, 176, 50], [63, 118, 228]);
        self.register("birch_forest", [136, 181, 99], [107, 163, 59], [63, 118, 228]);
        self.register("dark_forest", [80, 122, 50], [40, 96, 25], [63, 118, 228]);
        self.register("taiga", [134, 173, 119], [104, 153, 85], [40, 90, 180]);
        self.register("snowy_taiga", [120, 152, 129], [86, 128, 97], [32, 69, 164]);
        self.register("old_growth_pine_taiga", [134, 173, 119], [104, 153, 85], [40, 90, 180]);
        self.register("jungle", [89, 201, 60], [48, 188, 12], [20, 112, 236]);
        self.register("sparse_jungle", [100, 194, 65], [60, 180, 20], [20, 112, 236]);
        self.register("bamboo_jungle", [89, 201, 60], [48, 188, 12], [20, 112, 236]);
        self.register("badlands", [144, 129, 77], [158, 129, 77], [78, 97, 219]);
        self.register("eroded_badlands", [144, 129, 77], [158, 129, 77], [78, 97, 219]);
        self.register("wooded_badlands", [144, 129, 77], [158, 129, 77], [78, 97, 219]);
        self.register("savanna", [191, 183, 85], [174, 164, 42], [44, 101, 236]);
        self.register("savanna_plateau", [191, 183, 85], [174, 164, 42], [44, 101, 236]);
        self.register("windswept_hills", [138, 181, 112], [107, 163, 73], [63, 118, 228]);
        self.register("cherry_grove", [182, 219, 169], [182, 219, 169], [93, 183, 232]);
        self.register("meadow", [131, 187, 100], [99, 170, 60], [63, 118, 228]);
        self.register("ocean", [145, 189, 89], [119, 171, 47], [23, 63, 210]);
        self.register("deep_ocean", [145, 189, 89], [119, 171, 47], [20, 52, 190]);
        self.register("warm_ocean", [145, 189, 89], [119, 171, 47], [67, 213, 238]);
        self.register("lukewarm_ocean", [145, 189, 89], [119, 171, 47], [34, 148, 235]);
        self.register("cold_ocean", [145, 189, 89], [119, 171, 47], [32, 69, 164]);
        self.register("deep_cold_ocean", [145, 189, 89], [119, 171, 47], [27, 56, 140]);
        self.register("frozen_ocean", [128, 180, 151], [96, 161, 122], [20, 48, 130]);
        self.register("nether_wastes", [180, 60, 60], [180, 60, 60], [144, 16, 16]);
        self.register("crimson_forest", [220, 30, 30], [220, 30, 30], [180, 10, 10]);
        self.register("warped_forest", [20, 140, 130], [20, 140, 130], [20, 80, 140]);
        self.register("soul_sand_valley", [90, 80, 75], [90, 80, 75], [60, 50, 70]);
        self.register("basalt_deltas", [100, 100, 105], [100, 100, 105], [70, 70, 80]);
        self.register("the_end", [220, 220, 170], [220, 220, 170], [98, 82, 140]);
    }

    pub fn register(&mut self, biome_name: &str, grass: [u8; 3], foliage: [u8; 3], water: [u8; 3]) {
        let key = normalize_biome_key(biome_name);
        self.biomes.insert(
            key,
            BiomeColorEntry {
                grass_rgb: grass,
                foliage_rgb: foliage,
                water_rgb: water,
            },
        );
    }

    /// Loads VoxelMap `biomecolors.txt` (format `Biome Name=0xRRGGBB`).
    pub fn load_from_voxelmap_file<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<usize> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((name, hex_str)) = trimmed.split_once('=') {
                let clean_hex = hex_str.trim().trim_start_matches("0x");
                if let Ok(val) = u32::from_str_radix(clean_hex, 16) {
                    let r = ((val >> 16) & 0xFF) as u8;
                    let g = ((val >> 8) & 0xFF) as u8;
                    let b = (val & 0xFF) as u8;

                    let entry = BiomeColorEntry {
                        grass_rgb: [r, g, b],
                        foliage_rgb: [
                            (r as f32 * 0.9) as u8,
                            (g as f32 * 0.9) as u8,
                            (b as f32 * 0.9) as u8,
                        ],
                        water_rgb: [63, 118, 228],
                    };
                    self.biomes.insert(normalize_biome_key(name), entry);
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    /// Gets the biome color entry by name.
    pub fn get(&self, biome_name: &str) -> BiomeColorEntry {
        let key = normalize_biome_key(biome_name);
        self.biomes.get(&key).copied().unwrap_or(self.fallback)
    }

    /// Multiplies the base block color by the biome's grass color tint.
    pub fn tint_grass(&self, biome_name: &str, base_rgba: [u8; 4]) -> [u8; 4] {
        let entry = self.get(biome_name);
        multiply_rgb(base_rgba, entry.grass_rgb)
    }

    /// Multiplies the base block color by the biome's foliage color tint.
    pub fn tint_foliage(&self, biome_name: &str, base_rgba: [u8; 4]) -> [u8; 4] {
        let entry = self.get(biome_name);
        multiply_rgb(base_rgba, entry.foliage_rgb)
    }

    /// Applies the biome's water color.
    pub fn tint_water(&self, biome_name: &str, base_rgba: [u8; 4]) -> [u8; 4] {
        let entry = self.get(biome_name);
        [entry.water_rgb[0], entry.water_rgb[1], entry.water_rgb[2], base_rgba[3]]
    }

    pub fn len(&self) -> usize {
        self.biomes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.biomes.is_empty()
    }
}

pub fn normalize_biome_key(raw: &str) -> String {
    let lower = raw.trim().to_ascii_lowercase().replace(' ', "_");
    if let Some(stripped) = lower.strip_prefix("minecraft:") {
        stripped.to_string()
    } else {
        lower
    }
}

fn multiply_rgb(base_rgba: [u8; 4], tint_rgb: [u8; 3]) -> [u8; 4] {
    [
        ((base_rgba[0] as u32 * tint_rgb[0] as u32) / 255) as u8,
        ((base_rgba[1] as u32 * tint_rgb[1] as u32) / 255) as u8,
        ((base_rgba[2] as u32 * tint_rgb[2] as u32) / 255) as u8,
        base_rgba[3],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biome_defaults_and_tints() {
        let palette = BiomePalette::new_with_defaults();
        assert!(palette.len() >= 25);

        let plains = palette.get("minecraft:plains");
        assert_eq!(plains.grass_rgb[0], 145);

        let tinted_grass = palette.tint_grass("desert", [255, 255, 255, 255]);
        assert_eq!(tinted_grass[0], 191);
        assert_eq!(tinted_grass[1], 178);
    }
}
