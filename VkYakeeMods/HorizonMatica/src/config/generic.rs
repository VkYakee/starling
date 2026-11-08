use serde::{Deserialize, Serialize};
use crate::util::easy_place::EasyPlaceProtocol;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericConfigs {
    pub easy_place_mode: bool,
    pub easy_place_protocol: EasyPlaceProtocol,
    pub easy_place_first: bool,
    pub easy_place_click_adjacent: bool,
    pub easy_place_vanilla_reach: bool,
    pub easy_place_swap_interval_ms: u32,
    pub placement_restriction: bool,
    pub command_fill_max_volume: usize,
    pub command_limit_per_tick: usize,
    pub command_use_worldedit: bool,
    pub area_selections_per_world: bool,
    pub debug_logging: bool,
}

impl Default for GenericConfigs {
    fn default() -> Self {
        Self {
            easy_place_mode: false,
            easy_place_protocol: EasyPlaceProtocol::Auto,
            easy_place_first: true,
            easy_place_click_adjacent: false,
            easy_place_vanilla_reach: false,
            easy_place_swap_interval_ms: 0,
            placement_restriction: false,
            command_fill_max_volume: 32768,
            command_limit_per_tick: 8,
            command_use_worldedit: false,
            area_selections_per_world: true,
            debug_logging: false,
        }
    }
}
