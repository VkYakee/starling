use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::util::layer::LayerRange;
use crate::util::pos::{get_transformed_block_pos, BlockPos, Mirror, Rotation};
use super::litematic::LitematicaSchematic;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubRegionPlacement {
    pub pos: BlockPos,
    pub rotation: Rotation,
    pub mirror: Mirror,
    pub enabled: bool,
    pub ignore_entities: bool,
}

impl Default for SubRegionPlacement {
    fn default() -> Self {
        Self {
            pos: BlockPos::ZERO,
            rotation: Rotation::None,
            mirror: Mirror::None,
            enabled: true,
            ignore_entities: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchematicPlacement {
    pub origin: BlockPos,
    pub name: String,
    pub rotation: Rotation,
    pub mirror: Mirror,
    pub enabled: bool,
    pub enable_render: bool,
    pub render_enclosing_box: bool,
    pub locked: bool,
    pub sub_region_placements: HashMap<String, SubRegionPlacement>,
    pub layer_range: LayerRange,
}

impl SchematicPlacement {
    pub fn new(name: impl Into<String>, origin: BlockPos) -> Self {
        Self {
            origin,
            name: name.into(),
            rotation: Rotation::None,
            mirror: Mirror::None,
            enabled: true,
            enable_render: true,
            render_enclosing_box: true,
            locked: false,
            sub_region_placements: HashMap::new(),
            layer_range: LayerRange::default(),
        }
    }

    pub fn for_schematic(schematic: &LitematicaSchematic, origin: BlockPos) -> Self {
        let mut placement = Self::new(schematic.metadata.name.clone(), origin);
        for (name, region) in &schematic.regions {
            let sub = SubRegionPlacement {
                pos: region.position,
                rotation: Rotation::None,
                mirror: Mirror::None,
                enabled: true,
                ignore_entities: false,
            };
            placement.sub_region_placements.insert(name.clone(), sub);
        }
        placement
    }

    pub fn toggle_locked(&mut self) -> bool {
        self.locked = !self.locked;
        self.locked
    }

    pub fn rotate_clockwise(&mut self) {
        if !self.locked {
            self.rotation = self.rotation.rotate_cw();
        }
    }

    pub fn rotate_counter_clockwise(&mut self) {
        if !self.locked {
            self.rotation = self.rotation.rotate_ccw();
        }
    }

    pub fn flip_mirror(&mut self) {
        if !self.locked {
            self.mirror = match self.mirror {
                Mirror::None => Mirror::FrontBack,
                Mirror::FrontBack => Mirror::LeftRight,
                Mirror::LeftRight => Mirror::None,
            };
        }
    }

    pub fn nudge(&mut self, dx: i32, dy: i32, dz: i32) {
        if !self.locked {
            self.origin = self.origin.offset(dx, dy, dz);
        }
    }

    pub fn get_transformed_position(&self, region_name: &str, relative_pos: BlockPos) -> BlockPos {
        let sub_placement = self.sub_region_placements.get(region_name);
        let sub_pos = sub_placement.map(|s| s.pos).unwrap_or(BlockPos::ZERO);

        let combined_rel = relative_pos.add(sub_pos);
        let transformed = get_transformed_block_pos(combined_rel, self.mirror, self.rotation);
        self.origin.add(transformed)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SchematicPlacementManager {
    placements: Vec<SchematicPlacement>,
    selected_index: Option<usize>,
}

impl SchematicPlacementManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_placement(&mut self, placement: SchematicPlacement) -> usize {
        let idx = self.placements.len();
        self.placements.push(placement);
        self.selected_index = Some(idx);
        idx
    }

    pub fn remove_placement(&mut self, index: usize) -> Option<SchematicPlacement> {
        if index < self.placements.len() {
            let removed = self.placements.remove(index);
            if self.placements.is_empty() {
                self.selected_index = None;
            } else if let Some(sel) = self.selected_index {
                if sel >= self.placements.len() {
                    self.selected_index = Some(self.placements.len() - 1);
                }
            }
            Some(removed)
        } else {
            None
        }
    }

    pub fn get_selected(&self) -> Option<&SchematicPlacement> {
        self.selected_index.and_then(|idx| self.placements.get(idx))
    }

    pub fn get_selected_mut(&mut self) -> Option<&mut SchematicPlacement> {
        self.selected_index.and_then(|idx| self.placements.get_mut(idx))
    }

    pub fn set_selected_index(&mut self, idx: usize) {
        if idx < self.placements.len() {
            self.selected_index = Some(idx);
        }
    }

    pub fn placements(&self) -> &[SchematicPlacement] {
        &self.placements
    }
}
