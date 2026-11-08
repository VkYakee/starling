use serde::{Deserialize, Serialize};
use super::area_selection::AreaSelection;
use super::box_region::BoxRegion;
use crate::util::pos::BlockPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SelectionMode {
    #[default]
    Normal,
    Simple,
}

#[derive(Debug, Clone)]
pub struct SelectionManager {
    pub current_selection: AreaSelection,
    pub mode: SelectionMode,
}

impl Default for SelectionManager {
    fn default() -> Self {
        let mut sel = AreaSelection::new("Default");
        sel.add_box(BoxRegion::new("Box1", BlockPos::ZERO, BlockPos::new(10, 10, 10)));
        Self {
            current_selection: sel,
            mode: SelectionMode::Normal,
        }
    }
}

impl SelectionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_selection(&mut self, selection: AreaSelection) {
        self.current_selection = selection;
    }

    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            SelectionMode::Normal => SelectionMode::Simple,
            SelectionMode::Simple => SelectionMode::Normal,
        };
    }
}
