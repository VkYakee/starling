/// Configuration for Ray-Casting / Elevation Shading (Squaremap & Pl3xMap algorithm).
#[derive(Debug, Clone, Copy)]
pub struct DepthShadingConfig {
    pub shadow_ratio: f32,    // Default 180/255 = 0.70588
    pub neutral_ratio: f32,   // Default 220/255 = 0.86274
    pub highlight_ratio: f32, // Default 1.0
    pub threshold: f32,       // Default 0.6
    pub water_depth_shading: bool,
    pub max_water_depth: u8,  // Default 16
}

impl Default for DepthShadingConfig {
    fn default() -> Self {
        Self {
            shadow_ratio: 180.0 / 255.0,
            neutral_ratio: 220.0 / 255.0,
            highlight_ratio: 1.0,
            threshold: 0.6,
            water_depth_shading: true,
            max_water_depth: 16,
        }
    }
}

/// Shading engine implementing Squaremap, Pl3xMap, and Dynmap depth & fluid algorithms.
#[derive(Debug, Clone)]
pub struct DepthShadingEngine {
    config: DepthShadingConfig,
}

impl Default for DepthShadingEngine {
    fn default() -> Self {
        Self::new(DepthShadingConfig::default())
    }
}

impl DepthShadingEngine {
    pub fn new(config: DepthShadingConfig) -> Self {
        Self { config }
    }

    /// Computes elevation relief shading based on vertical height difference against the northern neighbor.
    ///
    /// `cur_y`: current block height
    /// `prev_y`: northern neighbor block height
    /// `x`, `z`: block coordinates (used for subtle checkerboard dithering)
    pub fn shade_elevation(&self, base_color: [u8; 4], cur_y: i32, prev_y: i32, x: i32, z: i32) -> [u8; 4] {
        let odd = (x + z) & 1;
        let diff_y = (cur_y - prev_y) as f32 + ((odd as f32) - 0.5) * 0.4;

        let ratio = if diff_y > self.config.threshold {
            self.config.highlight_ratio
        } else if diff_y < -self.config.threshold {
            self.config.shadow_ratio
        } else {
            self.config.neutral_ratio
        };

        apply_shade_ratio(base_color, ratio)
    }

    /// Computes realistic water depth tinting and blending with the underlying sea floor.
    ///
    /// Shallow water allows the ocean floor to be visible; deep water darkens progressively.
    pub fn shade_fluid(&self, water_color: [u8; 4], floor_color: [u8; 4], depth: u8) -> [u8; 4] {
        if !self.config.water_depth_shading || depth == 0 {
            return water_color;
        }

        let clamped_depth = depth.min(self.config.max_water_depth) as f32;
        // Water darkens as it gets deeper
        let depth_ratio = (0.85 - clamped_depth * 0.015).max(0.4);
        let shaded_water = apply_shade_ratio(water_color, depth_ratio);

        // Blend fraction with floor block
        // At depth 1, floor block is visibly showing through (alpha ~ 0.35)
        // At depth >= 10, floor block fades out
        let floor_mix = (0.30 / (clamped_depth / 2.0).max(1.0)).clamp(0.0, 0.40);

        super::palette::blend_rgba(floor_color, shaded_water, floor_mix)
    }
}

/// Applies a multiplicative shading ratio to RGB components while preserving alpha.
pub fn apply_shade_ratio(rgba: [u8; 4], ratio: f32) -> [u8; 4] {
    let r = ((rgba[0] as f32 * ratio).clamp(0.0, 255.0)) as u8;
    let g = ((rgba[1] as f32 * ratio).clamp(0.0, 255.0)) as u8;
    let b = ((rgba[2] as f32 * ratio).clamp(0.0, 255.0)) as u8;
    [r, g, b, rgba[3]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elevation_shading() {
        let engine = DepthShadingEngine::default();
        let white = [200, 200, 200, 255];

        // Flat terrain (cur_y == prev_y) -> neutral ratio ~ 0.862
        let flat = engine.shade_elevation(white, 64, 64, 0, 0);
        assert!(flat[0] < white[0]);

        // Hill slope up (cur_y > prev_y) -> highlight ratio 1.0
        let uphill = engine.shade_elevation(white, 66, 64, 0, 0);
        assert_eq!(uphill[0], white[0]);

        // Hill slope down (cur_y < prev_y) -> shadow ratio ~ 0.705
        let downhill = engine.shade_elevation(white, 62, 64, 0, 0);
        assert!(downhill[0] < flat[0]);
    }

    #[test]
    fn test_fluid_depth() {
        let engine = DepthShadingEngine::default();
        let water = [60, 100, 220, 255];
        let sand = [220, 200, 150, 255];

        let shallow = engine.shade_fluid(water, sand, 1);
        let deep = engine.shade_fluid(water, sand, 12);

        // Shallow water mixes some sand, so it has higher red component than deep water
        assert!(shallow[0] > deep[0]);
    }
}
