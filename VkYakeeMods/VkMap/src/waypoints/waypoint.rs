use serde::{Deserialize, Serialize};

/// Minecraft Dimensions supported by the mapping system.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Dimension {
    Overworld,
    Nether,
    TheEnd,
    Custom(String),
}

impl Default for Dimension {
    fn default() -> Self {
        Dimension::Overworld
    }
}

impl Dimension {
    pub fn as_str(&self) -> &str {
        match self {
            Dimension::Overworld => "overworld",
            Dimension::Nether => "the_nether",
            Dimension::TheEnd => "the_end",
            Dimension::Custom(s) => s.as_str(),
        }
    }

    /// Minecraft coordinate scale factor (Nether is 8:1 relative to Overworld).
    pub fn coordinate_scale(&self) -> f64 {
        match self {
            Dimension::Nether => 8.0,
            _ => 1.0,
        }
    }

    pub fn from_id(id: &str) -> Self {
        let clean = id.trim().to_ascii_lowercase();
        if clean.contains("nether") {
            Dimension::Nether
        } else if clean.contains("end") {
            Dimension::TheEnd
        } else if clean.contains("overworld") || clean.is_empty() {
            Dimension::Overworld
        } else {
            Dimension::Custom(id.to_string())
        }
    }
}

/// Standard player travel speeds in Minecraft (blocks/meters per second).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TravelSpeed {
    Walking,    // 4.317 m/s
    Sprinting,  // 5.612 m/s
    Horse,      // 9.6 m/s
    Elytra,     // 30.0 m/s
    IceBoat,    // 72.0 m/s
}

impl TravelSpeed {
    pub fn blocks_per_second(&self) -> f64 {
        match self {
            TravelSpeed::Walking => 4.317,
            TravelSpeed::Sprinting => 5.612,
            TravelSpeed::Horse => 9.6,
            TravelSpeed::Elytra => 30.0,
            TravelSpeed::IceBoat => 72.0,
        }
    }
}

/// Comprehensive Minecraft Waypoint (combining VoxelMap, Xaero, and Conflux standards).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Waypoint {
    pub id: String,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub dimension: Dimension,
    pub color: [u8; 3], // RGB (0..255)
    pub icon: String,   // "diamond", "house", "skull", "bed", "portal", "mine", etc.
    pub enabled: bool,
    pub is_death_point: bool,
    pub created_at: u64,
}

impl Waypoint {
    pub fn new(
        name: impl Into<String>,
        x: i32,
        y: i32,
        z: i32,
        dimension: Dimension,
        color: [u8; 3],
    ) -> Self {
        let name_str = name.into();
        let id = format!("{}_{}_{}_{}", name_str, x, y, z);
        Self {
            id,
            name: name_str,
            x,
            y,
            z,
            dimension,
            color,
            icon: "diamond".to_string(),
            enabled: true,
            is_death_point: false,
            created_at: 0,
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = icon.into();
        self
    }

    /// Calculates Euclidean 2D distance on the horizontal X-Z plane.
    pub fn distance_2d(&self, px: f64, pz: f64) -> f64 {
        let dx = (self.x as f64) - px;
        let dz = (self.z as f64) - pz;
        (dx * dx + dz * dz).sqrt()
    }

    /// Calculates Euclidean 3D distance.
    pub fn distance_3d(&self, px: f64, py: f64, pz: f64) -> f64 {
        let dx = (self.x as f64) - px;
        let dy = (self.y as f64) - py;
        let dz = (self.z as f64) - pz;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Computes estimated time of arrival (ETA in seconds) based on distance and travel speed.
    pub fn eta_seconds(&self, px: f64, pz: f64, speed: TravelSpeed) -> f64 {
        let dist = self.distance_2d(px, pz);
        dist / speed.blocks_per_second()
    }

    /// Returns human-readable ETA (e.g. "2m 15s" or "38s").
    pub fn eta_formatted(&self, px: f64, pz: f64, speed: TravelSpeed) -> String {
        let total_sec = self.eta_seconds(px, pz, speed).round() as u64;
        let mins = total_sec / 60;
        let secs = total_sec % 60;
        if mins > 0 {
            format!("{}m {:02}s", mins, secs)
        } else {
            format!("{}s", secs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waypoint_distance_and_eta() {
        let wp = Waypoint::new("Base", 100, 64, 100, Dimension::Overworld, [0, 255, 0]);
        let dist = wp.distance_2d(0.0, 0.0);
        assert!((dist - 141.42).abs() < 0.5);

        let eta = wp.eta_seconds(0.0, 0.0, TravelSpeed::Elytra);
        assert!((eta - 4.71).abs() < 0.1);
        assert_eq!(wp.eta_formatted(0.0, 0.0, TravelSpeed::Elytra), "5s");
    }
}
