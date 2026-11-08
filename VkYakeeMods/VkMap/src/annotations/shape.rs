use serde::{Deserialize, Serialize};

/// Geometric vector shapes for map annotations (Conflux-Map & XaeroPlus drawing standard).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnnotationShape {
    /// Freehand vector path / trail.
    Freehand {
        points: Vec<(i32, i32)>,
        thickness: u32,
    },
    /// Direct line connecting two points (highways, tunnels, rail paths).
    Line {
        start: (i32, i32),
        end: (i32, i32),
        thickness: u32,
    },
    /// Circular zone (radar safety radius, spawn protection, effect zones).
    Circle {
        center: (i32, i32),
        radius: i32,
        filled: bool,
    },
    /// Rectangular region (town borders, chunk claims, farm boundaries).
    Rectangle {
        min_x: i32,
        min_z: i32,
        max_x: i32,
        max_z: i32,
        filled: bool,
    },
    /// Text point marker.
    Label {
        pos: (i32, i32),
        text: String,
    },
}

/// A complete vector annotation object on the map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub title: String,
    pub color: [u8; 4],
    pub shape: AnnotationShape,
    pub visible: bool,
}

impl Annotation {
    pub fn new(id: impl Into<String>, title: impl Into<String>, color: [u8; 4], shape: AnnotationShape) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            color,
            shape,
            visible: true,
        }
    }

    pub fn line(id: impl Into<String>, p1: (i32, i32), p2: (i32, i32), color: [u8; 4], thickness: u32) -> Self {
        Self::new(
            id,
            "Highway",
            color,
            AnnotationShape::Line {
                start: p1,
                end: p2,
                thickness,
            },
        )
    }

    pub fn chunk_claim(id: impl Into<String>, owner: &str, chunk_x: i32, chunk_z: i32, color: [u8; 4]) -> Self {
        let min_x = chunk_x * 16;
        let min_z = chunk_z * 16;
        Self::new(
            id,
            format!("Claim: {}", owner),
            color,
            AnnotationShape::Rectangle {
                min_x,
                min_z,
                max_x: min_x + 15,
                max_z: min_z + 15,
                filled: false,
            },
        )
    }

    pub fn circle_zone(id: impl Into<String>, name: &str, center: (i32, i32), radius: i32, color: [u8; 4], filled: bool) -> Self {
        Self::new(
            id,
            name,
            color,
            AnnotationShape::Circle {
                center,
                radius,
                filled,
            },
        )
    }
}
