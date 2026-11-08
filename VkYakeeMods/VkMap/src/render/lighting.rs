use serde::{Deserialize, Serialize};

/// Lighting modes for the map renderer (Dynmap & Minecraft-Overviewer style).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightingMode {
    /// Pure block color without lighting.
    None,
    /// Standard full daytime sunlight.
    Daylight,
    /// Atmospheric nighttime illumination with moonlight ambient and glowing block lights.
    Night,
    /// Smooth ambient occlusion simulating soft contact shadows.
    AmbientOcclusion,
}

impl Default for LightingMode {
    fn default() -> Self {
        LightingMode::Daylight
    }
}

/// Lighting engine calculating light curves and block luminescence.
#[derive(Debug, Clone)]
pub struct LightingEngine {
    pub mode: LightingMode,
    pub night_ambient: f32, // Default 0.22 (dim moonlight)
}

impl Default for LightingEngine {
    fn default() -> Self {
        Self {
            mode: LightingMode::Daylight,
            night_ambient: 0.22,
        }
    }
}

impl LightingEngine {
    pub fn new(mode: LightingMode) -> Self {
        Self {
            mode,
            night_ambient: 0.22,
        }
    }

    /// Computes lit color given raw base RGBA and light levels (0..15).
    pub fn apply_lighting(&self, base_color: [u8; 4], sky_light: u8, block_light: u8) -> [u8; 4] {
        match self.mode {
            LightingMode::None => base_color,
            LightingMode::Daylight => {
                // In daylight, light level = max(sky_light, block_light)
                let max_light = sky_light.max(block_light).min(15) as f32;
                // Vanilla light formula: 0.8 ^ (15 - L) scaled between 0.15 and 1.0
                let factor = (0.2 + 0.8 * (max_light / 15.0)).clamp(0.2, 1.0);
                multiply_light(base_color, factor)
            }
            LightingMode::Night => {
                let sky_contrib = (sky_light.min(15) as f32 / 15.0) * self.night_ambient;
                let block_contrib = block_light.min(15) as f32 / 15.0;
                let total_light = (sky_contrib + block_contrib).clamp(self.night_ambient, 1.0);

                let lit = multiply_light(base_color, total_light);

                // If block light is high, add warm torchlight tint
                if block_light >= 8 {
                    warm_light_tint(lit, block_light)
                } else {
                    lit
                }
            }
            LightingMode::AmbientOcclusion => {
                // Subtle AO corner shading
                let factor = (sky_light.max(block_light).min(15) as f32 / 15.0).clamp(0.5, 1.0);
                multiply_light(base_color, factor)
            }
        }
    }

    /// Detects if a block is an active light source in Minecraft.
    pub fn get_block_luminance(block_name: &str) -> u8 {
        if block_name.contains("lava")
            || block_name.contains("glowstone")
            || block_name.contains("sea_lantern")
            || block_name.contains("campfire")
            || block_name.contains("fire")
            || block_name.contains("shroomlight")
        {
            15
        } else if block_name.contains("torch") || block_name.contains("lantern") {
            14
        } else if block_name.contains("redstone_lamp") && !block_name.contains("lit=false") {
            15
        } else if block_name.contains("crying_obsidian") || block_name.contains("magma_block") {
            10
        } else if block_name.contains("redstone_torch") {
            7
        } else {
            0
        }
    }
}

fn multiply_light(rgba: [u8; 4], factor: f32) -> [u8; 4] {
    [
        ((rgba[0] as f32 * factor).clamp(0.0, 255.0)) as u8,
        ((rgba[1] as f32 * factor).clamp(0.0, 255.0)) as u8,
        ((rgba[2] as f32 * factor).clamp(0.0, 255.0)) as u8,
        rgba[3],
    ]
}

fn warm_light_tint(rgba: [u8; 4], block_light: u8) -> [u8; 4] {
    let warmth = (block_light as f32 / 15.0) * 0.25;
    let r = ((rgba[0] as f32 * (1.0 + warmth)).min(255.0)) as u8;
    let g = ((rgba[1] as f32 * (1.0 + warmth * 0.7)).min(255.0)) as u8;
    let b = rgba[2];
    [r, g, b, rgba[3]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lighting_day_and_night() {
        let day = LightingEngine::new(LightingMode::Daylight);
        let night = LightingEngine::new(LightingMode::Night);

        let color = [200, 200, 200, 255];
        let lit_day = day.apply_lighting(color, 15, 0);
        let lit_night = night.apply_lighting(color, 15, 0);

        assert_eq!(lit_day[0], 200);
        // At night with only sky light, it should be significantly darker
        assert!(lit_night[0] < lit_day[0]);

        // At night near a torch (block light 14), it should be bright again
        let lit_torch = night.apply_lighting(color, 15, 14);
        assert!(lit_torch[0] > lit_night[0]);
    }
}
