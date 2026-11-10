use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::util::pos::BlockPos;
use super::box_region::BoxRegion;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AreaSelection {
    pub name: String,
    pub sub_region_boxes: HashMap<String, BoxRegion>,
    pub current_box: Option<String>,
    pub explicit_origin: Option<BlockPos>,
    pub calculated_origin: BlockPos,
}

impl AreaSelection {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            sub_region_boxes: HashMap::new(),
            current_box: None,
            explicit_origin: None,
            calculated_origin: BlockPos::ZERO,
        }
    }

    pub fn add_box(&mut self, box_region: BoxRegion) {
        let name = box_region.name.clone();
        self.sub_region_boxes.insert(name.clone(), box_region);
        self.current_box = Some(name);
        self.update_calculated_origin();
    }

    pub fn remove_box(&mut self, name: &str) -> Option<BoxRegion> {
        let removed = self.sub_region_boxes.remove(name);
        if self.current_box.as_deref() == Some(name) {
            self.current_box = self.sub_region_boxes.keys().next().cloned();
        }
        self.update_calculated_origin();
        removed
    }

    pub fn get_current_box(&self) -> Option<&BoxRegion> {
        self.current_box.as_ref().and_then(|name| self.sub_region_boxes.get(name))
    }

    pub fn get_current_box_mut(&mut self) -> Option<&mut BoxRegion> {
        let name = self.current_box.clone()?;
        self.sub_region_boxes.get_mut(&name)
    }

    pub fn update_calculated_origin(&mut self) {
        if self.sub_region_boxes.is_empty() {
            self.calculated_origin = BlockPos::ZERO;
            return;
        }

        let mut min = BlockPos::new(i32::MAX, i32::MAX, i32::MAX);
        for b in self.sub_region_boxes.values() {
            min = BlockPos::min(min, b.min_corner());
        }
        self.calculated_origin = min;
    }

    pub fn get_effective_origin(&self) -> BlockPos {
        self.explicit_origin.unwrap_or(self.calculated_origin)
    }
}
