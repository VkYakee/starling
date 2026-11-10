//! End pillar cracking - re-exported from existing working implementation.
//! The existing EndPillars implementation is correct and matches vanilla.

use crate::pillars::{EndPillars, PILLAR_COUNT};

/// Observed end pillar heights
#[derive(Debug, Clone, Default)]
pub struct PillarObservations {
    pub heights: [Option<i32>; PILLAR_COUNT],
}

impl PillarObservations {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_height(&mut self, index: usize, height: i32) {
        if index < PILLAR_COUNT {
            self.heights[index] = Some(height);
        }
    }

    pub fn observed_count(&self) -> usize {
        self.heights.iter().filter(|h| h.is_some()).count()
    }

    pub fn to_array(&self) -> [Option<i32>; PILLAR_COUNT] {
        self.heights
    }
}

/// Crack end pillar seed from observations
pub fn crack_pillar_seed(observations: &PillarObservations) -> Vec<u16> {
    EndPillars::find_pillar_seeds(&observations.heights)
}
