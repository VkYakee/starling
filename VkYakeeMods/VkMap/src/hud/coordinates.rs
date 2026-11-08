/// Formatter for player position, facing direction, chunk coords, and biome overlay (VoxelMap & Xaero standard).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FacingDirection {
    North,     // -Z
    NorthEast,
    East,      // +X
    SouthEast,
    South,     // +Z
    SouthWest,
    West,      // -X
    NorthWest,
}

impl FacingDirection {
    pub fn from_yaw(yaw: f32) -> Self {
        // Minecraft yaw: 0 = South, 90 = West, 180/-180 = North, -90/270 = East
        let norm_yaw = (yaw % 360.0 + 360.0) % 360.0;
        if (22.5..67.5).contains(&norm_yaw) {
            FacingDirection::SouthWest
        } else if (67.5..112.5).contains(&norm_yaw) {
            FacingDirection::West
        } else if (112.5..157.5).contains(&norm_yaw) {
            FacingDirection::NorthWest
        } else if (157.5..202.5).contains(&norm_yaw) {
            FacingDirection::North
        } else if (202.5..247.5).contains(&norm_yaw) {
            FacingDirection::NorthEast
        } else if (247.5..292.5).contains(&norm_yaw) {
            FacingDirection::East
        } else if (292.5..337.5).contains(&norm_yaw) {
            FacingDirection::SouthEast
        } else {
            FacingDirection::South
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            FacingDirection::North => "Norte (-Z)",
            FacingDirection::NorthEast => "Noreste (+X, -Z)",
            FacingDirection::East => "Este (+X)",
            FacingDirection::SouthEast => "Sureste (+X, +Z)",
            FacingDirection::South => "Sur (+Z)",
            FacingDirection::SouthWest => "Suroeste (-X, +Z)",
            FacingDirection::West => "Oeste (-X)",
            FacingDirection::NorthWest => "Noroeste (-X, -Z)",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            FacingDirection::North => "N",
            FacingDirection::NorthEast => "NE",
            FacingDirection::East => "E",
            FacingDirection::SouthEast => "SE",
            FacingDirection::South => "S",
            FacingDirection::SouthWest => "SW",
            FacingDirection::West => "W",
            FacingDirection::NorthWest => "NW",
        }
    }
}

/// HUD Coordinates and Telemetry Formatter.
#[derive(Debug, Clone, Default)]
pub struct CoordinatesFormatter;

impl CoordinatesFormatter {
    /// Formats clean one-line telemetry: `XYZ: 124, 64, -305 [Plains] S`
    pub fn format_compact(x: f64, y: f64, z: f64, yaw: f32, biome: &str) -> String {
        let facing = FacingDirection::from_yaw(yaw);
        format!(
            "XYZ: {:.0}, {:.0}, {:.0} [{}] {}",
            x,
            y,
            z,
            clean_biome_name(biome),
            facing.short_name()
        )
    }

    /// Formats detailed multi-line HUD lines.
    pub fn format_multiline(x: f64, y: f64, z: f64, yaw: f32, biome: &str) -> Vec<String> {
        let facing = FacingDirection::from_yaw(yaw);
        let chunk_x = (x.floor() as i32) >> 4;
        let chunk_z = (z.floor() as i32) >> 4;
        let sub_x = (x.floor() as i32) & 15;
        let sub_z = (z.floor() as i32) & 15;

        vec![
            format!("XYZ: {:.1} / {:.1} / {:.1}", x, y, z),
            format!("Bloque en Chunk: [{}, {}] Chunk: [{}, {}]", sub_x, sub_z, chunk_x, chunk_z),
            format!("Bioma: {}", clean_biome_name(biome)),
            format!("Mirando: {} ({:.1}°)", facing.display_name(), yaw),
        ]
    }
}

fn clean_biome_name(raw: &str) -> String {
    let stripped = raw.strip_prefix("minecraft:").unwrap_or(raw);
    stripped
        .split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_facing_directions() {
        assert_eq!(FacingDirection::from_yaw(0.0), FacingDirection::South);
        assert_eq!(FacingDirection::from_yaw(90.0), FacingDirection::West);
        assert_eq!(FacingDirection::from_yaw(180.0), FacingDirection::North);
        assert_eq!(FacingDirection::from_yaw(270.0), FacingDirection::East);
        assert_eq!(FacingDirection::from_yaw(-90.0), FacingDirection::East);
    }

    #[test]
    fn test_telemetry_formatter() {
        let compact = CoordinatesFormatter::format_compact(120.4, 64.0, -450.2, 180.0, "minecraft:dark_forest");
        assert_eq!(compact, "XYZ: 120, 64, -450 [Dark Forest] N");
    }
}
