use crate::math::chunk_rand::ChunkRand;
use crate::math::lcg::{JavaRandom, MASK48, MULTIPLIER};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeType {
    Oak,
    Birch,
}

#[derive(Debug, Clone)]
pub struct BlobTreeData {
    pub tree_type: TreeType,
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub trunk_height: i32,
    pub leaves: Vec<i8>, // 0 = absent, 1 = present, -1 = unknown
}

impl BlobTreeData {
    pub fn new_oak(chunk_x: i32, chunk_z: i32, trunk_height: i32, leaves: Vec<i8>) -> Self {
        Self {
            tree_type: TreeType::Oak,
            chunk_x,
            chunk_z,
            trunk_height,
            leaves,
        }
    }

    pub fn new_birch(chunk_x: i32, chunk_z: i32, trunk_height: i32, leaves: Vec<i8>) -> Self {
        Self {
            tree_type: TreeType::Birch,
            chunk_x,
            chunk_z,
            trunk_height,
            leaves,
        }
    }
}

pub struct TreeCracker;

impl TreeCracker {
    /// Validates if a seed generates the tree parameters.
    pub fn test_tree_seed(mut rand: JavaRandom, tree: &BlobTreeData) -> bool {
        let (base_height, first_random_height) = match tree.tree_type {
            TreeType::Oak => (4, 2),
            TreeType::Birch => (5, 2),
        };

        if tree.trunk_height < base_height || tree.trunk_height > (base_height + first_random_height) {
            return false;
        }

        // Tree height check
        if rand.next_int_bounded(first_random_height + 1) != (tree.trunk_height - base_height) {
            return false;
        }

        // Trunk placer extra call
        rand.next_int();

        // 1.16+ top 4 leaves skip
        for _ in 0..4 {
            rand.next_int();
        }

        // Mutable leaves check
        for &leaf in &tree.leaves {
            if leaf == -1 {
                rand.next_int();
            } else if rand.next_int_bounded(2) != (leaf as i32) {
                return false;
            }
        }

        true
    }

    /// Solves for structure seed from tree observations.
    pub fn crack_from_tree(
        tree: &BlobTreeData,
        candidate_feature_seeds: &[u64],
        salt: u64,
    ) -> Vec<u64> {
        let mut matching_seeds = Vec::new();
        for &seed in candidate_feature_seeds {
            let rand = JavaRandom::with_raw_seed(seed);
            if Self::test_tree_seed(rand, tree) {
                let pop_seed = ((seed ^ MULTIPLIER).wrapping_sub(salt)) & MASK48;
                let structure_seed =
                    ChunkRand::reverse_population_seed(pop_seed, tree.chunk_x, tree.chunk_z);
                matching_seeds.push(structure_seed);
            }
        }
        matching_seeds
    }
}
