use serde::{Deserialize, Serialize};
use super::pos::{BlockPos, Direction};
use crate::schematic::palette::BlockState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EasyPlaceProtocol {
    #[default]
    Auto,
    V2,
    V3,
    SlabOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EasyPlaceAction {
    pub target_pos: BlockPos,
    pub clicked_side: Direction,
    pub click_offset_x: f32,
    pub click_offset_y: f32,
    pub click_offset_z: f32,
}

pub struct EasyPlaceUtils;

impl EasyPlaceUtils {
    /// Determines whether the player's held item matches the required schematic block
    pub fn is_held_item_matching(held_item_id: &str, schematic_state: &BlockState) -> bool {
        let block_name = schematic_state.name.strip_prefix("minecraft:").unwrap_or(&schematic_state.name);
        let item_name = held_item_id.strip_prefix("minecraft:").unwrap_or(held_item_id);
        block_name == item_name
    }

    /// Calculates click offset to encode orientation for protocol V2 (e.g. facing, axis, half)
    pub fn calculate_click_offset(state: &BlockState, _clicked_side: Direction) -> (f32, f32, f32) {
        let mut x = 0.5f32;
        let mut y = 0.5f32;
        let mut z = 0.5f32;

        if let Some(facing) = state.get_property("facing") {
            match facing {
                "north" => z = 0.1,
                "south" => z = 0.9,
                "west" => x = 0.1,
                "east" => x = 0.9,
                "up" => y = 0.9,
                "down" => y = 0.1,
                _ => {}
            }
        } else if let Some(axis) = state.get_property("axis") {
            match axis {
                "x" => { x = 0.9; y = 0.5; z = 0.5; },
                "y" => { x = 0.5; y = 0.9; z = 0.5; },
                "z" => { x = 0.5; y = 0.5; z = 0.9; },
                _ => {}
            }
        }

        if let Some(half) = state.get_property("half").or_else(|| state.get_property("type")) {
            if half == "top" {
                y = 0.75;
            } else if half == "bottom" {
                y = 0.25;
            }
        }

        (x, y, z)
    }

    /// Evaluates if placement is valid according to placement restrictions
    pub fn is_placement_allowed(
        schematic_state: &BlockState,
        world_state: &BlockState,
        allow_replacement: bool,
    ) -> bool {
        if world_state.is_air() {
            return true;
        }

        if allow_replacement && world_state.name != schematic_state.name {
            return true;
        }

        false
    }
}
