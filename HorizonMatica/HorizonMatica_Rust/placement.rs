// Copyright (c) 2026 HorizonMatica Authors / Starling Android
//! Placement engine: spatial coordinates, rotation, mirroring, layer slicing,
//! and bounding box computation.

use std::collections::HashMap;
use super::container::SchematicBlockState;
use super::format::LoadedSchematic;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    CounterClockwise90,
}

impl Rotation {
    pub fn cycle(&mut self) {
        *self = match self {
            Self::None => Self::Clockwise90,
            Self::Clockwise90 => Self::Clockwise180,
            Self::Clockwise180 => Self::CounterClockwise90,
            Self::CounterClockwise90 => Self::None,
        };
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::None => "0°",
            Self::Clockwise90 => "90°",
            Self::Clockwise180 => "180°",
            Self::CounterClockwise90 => "270°",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mirror {
    None,
    X, // Front-Back
    Z, // Left-Right
}

impl Mirror {
    pub fn cycle(&mut self) {
        *self = match self {
            Self::None => Self::X,
            Self::X => Self::Z,
            Self::Z => Self::None,
        };
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::X => "X (F/B)",
            Self::Z => "Z (L/R)",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerMode {
    All,
    Single(i32),
    Range(i32, i32),
}

impl LayerMode {
    pub fn cycle(&mut self, current_y: i32, max_height: i32) {
        *self = match self {
            Self::All => Self::Single(current_y.clamp(0, (max_height - 1).max(0))),
            Self::Single(y) => Self::Range(0, (*y).clamp(0, (max_height - 1).max(0))),
            Self::Range(_, _) => Self::All,
        };
    }

    pub fn adjust_layer(&mut self, delta: i32, max_height: i32) {
        let max_idx = (max_height - 1).max(0);
        match self {
            Self::All => {}
            Self::Single(y) => {
                *y = (*y + delta).clamp(0, max_idx);
            }
            Self::Range(min_y, max_y) => {
                *max_y = (*max_y + delta).clamp(*min_y, max_idx);
            }
        }
    }

    pub fn is_layer_visible(&self, local_y: i32) -> bool {
        match self {
            Self::All => true,
            Self::Single(y) => local_y == *y,
            Self::Range(min_y, max_y) => local_y >= *min_y && local_y <= *max_y,
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            Self::All => "ALL".to_string(),
            Self::Single(y) => format!("Y={y}"),
            Self::Range(min_y, max_y) => format!("Y=[{min_y}..{max_y}]"),
        }
    }
}

/// Active placement configuration for an in-game schematic.
#[derive(Clone, Debug)]
pub struct SchematicPlacement {
    pub origin: [i32; 3],
    pub rotation: Rotation,
    pub mirror: Mirror,
    pub layer_mode: LayerMode,
    pub enabled: bool,
    pub subregion_enabled: HashMap<String, bool>,
}

impl Default for SchematicPlacement {
    fn default() -> Self {
        Self {
            origin: [0, 64, 0],
            rotation: Rotation::None,
            mirror: Mirror::None,
            layer_mode: LayerMode::All,
            enabled: true,
            subregion_enabled: HashMap::new(),
        }
    }
}

impl SchematicPlacement {
    pub fn new(origin: [i32; 3]) -> Self {
        Self {
            origin,
            ..Default::default()
        }
    }

    pub fn is_subregion_enabled(&self, name: &str) -> bool {
        self.subregion_enabled.get(name).copied().unwrap_or(true)
    }

    /// Transforms local subregion block pos into world coordinate.
    pub fn transform_local_to_world(&self, local_pos: [i32; 3], sub_offset: [i32; 3]) -> [i32; 3] {
        let x = local_pos[0] + sub_offset[0];
        let y = local_pos[1] + sub_offset[1];
        let z = local_pos[2] + sub_offset[2];

        // 1. Mirror
        let mut mx = x;
        let my = y;
        let mut mz = z;
        match self.mirror {
            Mirror::None => {}
            Mirror::X => mx = -mx,
            Mirror::Z => mz = -mz,
        }

        // 2. Rotate
        let (rx, ry, rz) = match self.rotation {
            Rotation::None => (mx, my, mz),
            Rotation::Clockwise90 => (-mz, my, mx),
            Rotation::Clockwise180 => (-mx, my, -mz),
            Rotation::CounterClockwise90 => (mz, my, -mx),
        };

        // 3. Translate by placement origin
        [
            self.origin[0] + rx,
            self.origin[1] + ry,
            self.origin[2] + rz,
        ]
    }

    /// Transforms world block pos into local subregion block pos.
    pub fn transform_world_to_local(&self, world_pos: [i32; 3], sub_offset: [i32; 3]) -> [i32; 3] {
        // 1. Untranslate
        let tx = world_pos[0] - self.origin[0];
        let ty = world_pos[1] - self.origin[1];
        let tz = world_pos[2] - self.origin[2];

        // 2. Unrotate
        let (ux, uy, uz) = match self.rotation {
            Rotation::None => (tx, ty, tz),
            Rotation::Clockwise90 => (tz, ty, -tx),
            Rotation::Clockwise180 => (-tx, ty, -tz),
            Rotation::CounterClockwise90 => (-tz, ty, tx),
        };

        // 3. Unmirror
        let mut mx = ux;
        let my = uy;
        let mut mz = uz;
        match self.mirror {
            Mirror::None => {}
            Mirror::X => mx = -mx,
            Mirror::Z => mz = -mz,
        }

        // 4. Subtract subregion offset
        [
            mx - sub_offset[0],
            my - sub_offset[1],
            mz - sub_offset[2],
        ]
    }

    /// Computes world axis-aligned bounding box [min_x, min_y, min_z], [max_x, max_y, max_z]
    pub fn compute_world_bounds(&self, schematic: &LoadedSchematic) -> ([i32; 3], [i32; 3]) {
        if schematic.regions.is_empty() {
            return (self.origin, self.origin);
        }

        let mut min = [i32::MAX, i32::MAX, i32::MAX];
        let mut max = [i32::MIN, i32::MIN, i32::MIN];

        for r in &schematic.regions {
            if !self.is_subregion_enabled(&r.name) {
                continue;
            }
            let corners = [
                [0, 0, 0],
                [r.size[0] - 1, 0, 0],
                [0, r.size[1] - 1, 0],
                [r.size[0] - 1, r.size[1] - 1, 0],
                [0, 0, r.size[2] - 1],
                [r.size[0] - 1, 0, r.size[2] - 1],
                [0, r.size[1] - 1, r.size[2] - 1],
                [r.size[0] - 1, r.size[1] - 1, r.size[2] - 1],
            ];

            for c in corners {
                let wp = self.transform_local_to_world(c, r.pos);
                for i in 0..3 {
                    min[i] = min[i].min(wp[i]);
                    max[i] = max[i].max(wp[i]);
                }
            }
        }

        if min[0] == i32::MAX {
            (self.origin, self.origin)
        } else {
            (min, max)
        }
    }

    /// Rotates a blockstate's properties (like facing, axis) to match the placement rotation.
    pub fn rotate_state_properties(&self, state: &SchematicBlockState) -> SchematicBlockState {
        if self.rotation == Rotation::None && self.mirror == Mirror::None {
            return state.clone();
        }

        let mut new_props = state.properties.clone();

        if let Some(facing) = new_props.get_mut("facing") {
            *facing = rotate_facing(facing, self.rotation, self.mirror);
        }
        if let Some(axis) = new_props.get_mut("axis") {
            *axis = rotate_axis(axis, self.rotation);
        }

        SchematicBlockState {
            name: state.name.clone(),
            properties: new_props,
        }
    }
}

fn rotate_facing(facing: &str, rot: Rotation, mirror: Mirror) -> String {
    let mut f = facing.to_string();
    match mirror {
        Mirror::None => {}
        Mirror::X => {
            if f == "east" { f = "west".into(); }
            else if f == "west" { f = "east".into(); }
        }
        Mirror::Z => {
            if f == "north" { f = "south".into(); }
            else if f == "south" { f = "north".into(); }
        }
    }

    match rot {
        Rotation::None => f,
        Rotation::Clockwise90 => match f.as_str() {
            "north" => "east".into(),
            "east" => "south".into(),
            "south" => "west".into(),
            "west" => "north".into(),
            other => other.to_string(),
        },
        Rotation::Clockwise180 => match f.as_str() {
            "north" => "south".into(),
            "south" => "north".into(),
            "east" => "west".into(),
            "west" => "east".into(),
            other => other.to_string(),
        },
        Rotation::CounterClockwise90 => match f.as_str() {
            "north" => "west".into(),
            "west" => "south".into(),
            "south" => "east".into(),
            "east" => "north".into(),
            other => other.to_string(),
        },
    }
}

fn rotate_axis(axis: &str, rot: Rotation) -> String {
    match rot {
        Rotation::Clockwise90 | Rotation::CounterClockwise90 => match axis {
            "x" => "z".into(),
            "z" => "x".into(),
            other => other.to_string(),
        },
        _ => axis.to_string(),
    }
}
