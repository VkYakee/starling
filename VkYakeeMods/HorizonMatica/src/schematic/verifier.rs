use serde::{Deserialize, Serialize};

use crate::util::pos::BlockPos;
use super::litematic::LitematicaSchematic;
use super::palette::BlockState;
use super::placement::SchematicPlacement;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MismatchType {
    Missing,
    Extra,
    WrongBlock,
    WrongState,
    DiffBlock,
    CorrectState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockMismatch {
    pub mismatch_type: MismatchType,
    pub expected: BlockState,
    pub found: BlockState,
    pub pos: BlockPos,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationSummary {
    pub total_schematic_blocks: usize,
    pub total_world_blocks: usize,
    pub correct_count: usize,
    pub missing_count: usize,
    pub extra_count: usize,
    pub wrong_block_count: usize,
    pub wrong_state_count: usize,
    pub diff_block_count: usize,
}

impl VerificationSummary {
    pub fn total_errors(&self) -> usize {
        self.missing_count + self.extra_count + self.wrong_block_count + self.wrong_state_count + self.diff_block_count
    }
}

pub struct SchematicVerifier;

impl SchematicVerifier {
    /// Verifies world blocks against a schematic placement using a world lookup closure
    pub fn verify<F>(
        schematic: &LitematicaSchematic,
        placement: &SchematicPlacement,
        mut world_block_lookup: F,
    ) -> (VerificationSummary, Vec<BlockMismatch>)
    where
        F: FnMut(BlockPos) -> BlockState,
    {
        let mut summary = VerificationSummary::default();
        let mut mismatches = Vec::new();

        for (region_name, sub_region) in &schematic.regions {
            let sub_placement = match placement.sub_region_placements.get(region_name) {
                Some(p) if p.enabled => p,
                _ => continue,
            };
            let _ = sub_placement;

            let container = &sub_region.container;
            for x in 0..container.size_x {
                for y in 0..container.size_y {
                    for z in 0..container.size_z {
                        let rel_pos = BlockPos::new(x as i32, y as i32, z as i32);
                        let world_pos = placement.get_transformed_position(region_name, rel_pos);

                        if !placement.layer_range.is_position_within_layer(world_pos) {
                            continue;
                        }

                        let expected = container.get(x, y, z);
                        let found = world_block_lookup(world_pos);

                        if !expected.is_air() {
                            summary.total_schematic_blocks += 1;
                        }
                        if !found.is_air() {
                            summary.total_world_blocks += 1;
                        }

                        if expected == found {
                            if !expected.is_air() {
                                summary.correct_count += 1;
                            }
                        } else if found.is_air() && !expected.is_air() {
                            summary.missing_count += 1;
                            mismatches.push(BlockMismatch {
                                mismatch_type: MismatchType::Missing,
                                expected,
                                found,
                                pos: world_pos,
                            });
                        } else if expected.is_air() && !found.is_air() {
                            summary.extra_count += 1;
                            mismatches.push(BlockMismatch {
                                mismatch_type: MismatchType::Extra,
                                expected,
                                found,
                                pos: world_pos,
                            });
                        } else if expected.name != found.name {
                            summary.wrong_block_count += 1;
                            mismatches.push(BlockMismatch {
                                mismatch_type: MismatchType::WrongBlock,
                                expected,
                                found,
                                pos: world_pos,
                            });
                        } else {
                            summary.wrong_state_count += 1;
                            mismatches.push(BlockMismatch {
                                mismatch_type: MismatchType::WrongState,
                                expected,
                                found,
                                pos: world_pos,
                            });
                        }
                    }
                }
            }
        }

        (summary, mismatches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schematic::litematic::SubRegion;

    #[test]
    fn test_verifier_accuracy() {
        let mut schem = LitematicaSchematic::new("Test", "Author");
        let mut sub = SubRegion::new(BlockPos::ZERO, BlockPos::new(2, 1, 2));

        let stone = BlockState::new("minecraft:stone");
        let oak_stairs_north = BlockState::new("minecraft:oak_stairs").with_property("facing", "north");
        let oak_stairs_south = BlockState::new("minecraft:oak_stairs").with_property("facing", "south");

        sub.container.set(0, 0, 0, stone.clone());
        sub.container.set(1, 0, 0, oak_stairs_north.clone());
        schem.add_region("main", sub);

        let placement = SchematicPlacement::for_schematic(&schem, BlockPos::ZERO);

        let mut world_mock = std::collections::HashMap::new();
        world_mock.insert(BlockPos::new(0, 0, 0), stone.clone()); // Correct
        world_mock.insert(BlockPos::new(1, 0, 0), oak_stairs_south); // Wrong state!
        world_mock.insert(BlockPos::new(0, 0, 1), BlockState::new("minecraft:dirt")); // Extra!

        let (summary, _mismatches) = SchematicVerifier::verify(&schem, &placement, |pos| {
            world_mock.get(&pos).cloned().unwrap_or_else(BlockState::air)
        });

        assert_eq!(summary.correct_count, 1);
        assert_eq!(summary.wrong_state_count, 1);
        assert_eq!(summary.extra_count, 1);
        assert_eq!(summary.total_errors(), 2);
    }
}
