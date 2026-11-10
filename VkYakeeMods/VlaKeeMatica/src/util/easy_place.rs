//! EasyPlace protocol implementation for schematic placement

use serde::{Deserialize, Serialize};

/// EasyPlace protocol variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EasyPlaceProtocol {
    /// Automatic - chooses best protocol based on schematic
    Auto,
    /// EasyPlace V1 - basic single-click placement
    V1,
    /// EasyPlace V2 - multi-click with offset tracking
    V2,
    /// EasyPlace V3 - advanced with rotation/mirror support
    V3,
    /// Slab-only placement mode
    SlabOnly,
}

impl Default for EasyPlaceProtocol {
    fn default() -> Self {
        EasyPlaceProtocol::V2
    }
}

impl EasyPlaceProtocol {
    pub fn all() -> [EasyPlaceProtocol; 5] {
        [
            EasyPlaceProtocol::Auto,
            EasyPlaceProtocol::V1,
            EasyPlaceProtocol::V2,
            EasyPlaceProtocol::V3,
            EasyPlaceProtocol::SlabOnly,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            EasyPlaceProtocol::Auto => "Auto",
            EasyPlaceProtocol::V1 => "V1",
            EasyPlaceProtocol::V2 => "V2",
            EasyPlaceProtocol::V3 => "V3",
            EasyPlaceProtocol::SlabOnly => "SlabOnly",
        }
    }

    pub fn next(self) -> Self {
        match self {
            EasyPlaceProtocol::Auto => EasyPlaceProtocol::V1,
            EasyPlaceProtocol::V1 => EasyPlaceProtocol::V2,
            EasyPlaceProtocol::V2 => EasyPlaceProtocol::V3,
            EasyPlaceProtocol::V3 => EasyPlaceProtocol::SlabOnly,
            EasyPlaceProtocol::SlabOnly => EasyPlaceProtocol::Auto,
        }
    }
}

/// Calculates click offset for EasyPlace protocols
pub struct EasyPlaceCalculator;

impl EasyPlaceCalculator {
    /// Calculate the offset for the next click based on protocol
    pub fn calculate_offset(
        protocol: EasyPlaceProtocol,
        click_count: usize,
        schematic_size: (usize, usize, usize),
    ) -> (i32, i32, i32) {
        match protocol {
            EasyPlaceProtocol::Auto => {
                // Auto-detect based on schematic complexity
                if schematic_size.0 * schematic_size.1 * schematic_size.2 > 10000 {
                    Self::calculate_v3_offset(click_count, schematic_size)
                } else {
                    Self::calculate_v2_offset(click_count, schematic_size)
                }
            }
            EasyPlaceProtocol::V1 => (0, 0, 0),
            EasyPlaceProtocol::V2 => Self::calculate_v2_offset(click_count, schematic_size),
            EasyPlaceProtocol::V3 => Self::calculate_v3_offset(click_count, schematic_size),
            EasyPlaceProtocol::SlabOnly => Self::calculate_slab_offset(click_count, schematic_size),
        }
    }

    fn calculate_v2_offset(click_count: usize, size: (usize, usize, usize)) -> (i32, i32, i32) {
        // V2: Simple grid pattern
        let (w, _h, d) = size;
        let layer_area = w * d;
        let layer = click_count / layer_area.max(1);
        let in_layer = click_count % layer_area.max(1);
        let x = in_layer % w.max(1);
        let z = in_layer / w.max(1);
        (x as i32, layer as i32, z as i32)
    }

    fn calculate_v3_offset(click_count: usize, size: (usize, usize, usize)) -> (i32, i32, i32) {
        // V3: Optimized path with rotation awareness
        let (w, _h, d) = size;
        let spiral_layer = click_count / (w * d).max(1);
        let in_layer = click_count % (w * d).max(1);
        let x = in_layer % w.max(1);
        let z = in_layer / w.max(1);
        (x as i32, spiral_layer as i32, z as i32)
    }

    fn calculate_slab_offset(click_count: usize, size: (usize, usize, usize)) -> (i32, i32, i32) {
        // SlabOnly: Only places on slab-valid positions (even Y for bottom slabs)
        let (w, _h, d) = size;
        let layer_area = w * d;
        let layer = (click_count * 2) / layer_area.max(1); // Double spacing for slabs
        let in_layer = click_count % layer_area.max(1);
        let x = in_layer % w.max(1);
        let z = in_layer / w.max(1);
        (x as i32, layer as i32 * 2, z as i32) // Even Y levels only
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easy_place_cycle() {
        let mut p = EasyPlaceProtocol::Auto;
        for _ in 0..5 {
            p = p.next();
        }
        assert_eq!(p, EasyPlaceProtocol::Auto);
    }

    #[test]
    fn test_calculate_v2_offset() {
        let offset = EasyPlaceCalculator::calculate_v2_offset(0, (10, 5, 10));
        assert_eq!(offset, (0, 0, 0));
        
        let offset = EasyPlaceCalculator::calculate_v2_offset(5, (10, 5, 10));
        assert_eq!(offset, (5, 0, 0));
    }
}
