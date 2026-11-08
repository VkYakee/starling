use serde::{Deserialize, Serialize};
use crate::util::pos::{BlockPos, Vec3i};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SelectedCorner {
    #[default]
    None,
    Corner1,
    Corner2,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoxRegion {
    pub name: String,
    pub pos1: BlockPos,
    pub pos2: BlockPos,
    pub selected_corner: SelectedCorner,
}

impl BoxRegion {
    pub fn new(name: impl Into<String>, pos1: BlockPos, pos2: BlockPos) -> Self {
        Self {
            name: name.into(),
            pos1,
            pos2,
            selected_corner: SelectedCorner::None,
        }
    }

    pub fn min_corner(&self) -> BlockPos {
        BlockPos::min(self.pos1, self.pos2)
    }

    pub fn max_corner(&self) -> BlockPos {
        BlockPos::max(self.pos1, self.pos2)
    }

    pub fn size(&self) -> Vec3i {
        let min = self.min_corner();
        let max = self.max_corner();
        Vec3i::new(
            max.x - min.x + 1,
            max.y - min.y + 1,
            max.z - min.z + 1,
        )
    }

    pub fn volume(&self) -> usize {
        let size = self.size();
        (size.x.unsigned_abs() * size.y.unsigned_abs() * size.z.unsigned_abs()) as usize
    }

    pub fn nudge_corner(&mut self, corner: SelectedCorner, dx: i32, dy: i32, dz: i32) {
        match corner {
            SelectedCorner::Corner1 => self.pos1 = self.pos1.offset(dx, dy, dz),
            SelectedCorner::Corner2 => self.pos2 = self.pos2.offset(dx, dy, dz),
            SelectedCorner::None => {
                self.pos1 = self.pos1.offset(dx, dy, dz);
                self.pos2 = self.pos2.offset(dx, dy, dz);
            }
        }
    }
}
