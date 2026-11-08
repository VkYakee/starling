use rayon::prelude::*;

use crate::biomes::cubiomes::BiomeGenerator;

#[derive(Debug, Clone, Copy)]
pub struct BiomeSample {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub expected_biome_id: i32,
}

impl BiomeSample {
    pub fn new(x: i32, y: i32, z: i32, expected_biome_id: i32) -> Self {
        Self {
            x,
            y,
            z,
            expected_biome_id,
        }
    }
}

pub struct BiomeLifter;

impl BiomeLifter {
    /// Lifts a 48-bit structure seed to 64-bit world seed(s) matching known biome locations.
    pub fn lift_seed(
        structure_seed: u64,
        samples: &[BiomeSample],
        version: i32,
    ) -> Vec<i64> {
        if samples.is_empty() {
            return Vec::new();
        }

        let base_48 = structure_seed & 0xFFFFFFFFFFFF;

        (0..=0xFFFFu64)
            .into_par_iter()
            .map_init(
                || BiomeGenerator::new(version),
                |r#gen, upper| {
                    let candidate = ((upper << 48) | base_48) as i64;
                    r#gen.set_seed(candidate as u64);

                    let mut matches = true;
                    for sample in samples {
                        let actual = r#gen.get_biome_at_block(sample.x, sample.y, sample.z);
                        if actual != sample.expected_biome_id {
                            matches = false;
                            break;
                        }
                    }

                    if matches {
                        Some(candidate)
                    } else {
                        None
                    }
                },
            )
            .flatten()
            .collect()
    }
}
