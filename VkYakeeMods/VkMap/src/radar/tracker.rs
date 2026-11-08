use super::entity::{EntityCategory, RadarEntity};
use serde::{Deserialize, Serialize};

/// Elevation indicator showing whether an entity is above, below, or on the same level as the player (VoxelMap style).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElevationIndicator {
    Above, // '^' marker or brightened icon
    Level, // standard circle marker
    Below, // 'v' marker or darkened icon
}

/// Entity radar visibility mode to prevent screen clutter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RadarVisibilityMode {
    /// Only entities within the player's forward line of sight / field of view (forward 120° cone).
    LineOfSight,
    /// Omniscient X-Ray: shows all entities regardless of layer, underground depth, or obstacles.
    AllLayers,
    /// Only entities situated on the same surface level as the player (within ±5 blocks Y).
    SurfaceOnly,
}

impl Default for RadarVisibilityMode {
    fn default() -> Self {
        RadarVisibilityMode::AllLayers
    }
}

impl RadarVisibilityMode {
    pub fn name(&self) -> &'static str {
        match self {
            RadarVisibilityMode::LineOfSight => "Línea de Visión (FOV)",
            RadarVisibilityMode::AllLayers => "Todas las Capas (Rayos X)",
            RadarVisibilityMode::SurfaceOnly => "Misma Superficie (±5 Y)",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            RadarVisibilityMode::LineOfSight => RadarVisibilityMode::AllLayers,
            RadarVisibilityMode::AllLayers => RadarVisibilityMode::SurfaceOnly,
            RadarVisibilityMode::SurfaceOnly => RadarVisibilityMode::LineOfSight,
        }
    }
}

/// Computed radar contact relative to the player camera.
#[derive(Debug, Clone, PartialEq)]
pub struct RadarContact {
    pub entity: RadarEntity,
    pub dx: f64,
    pub dz: f64,
    pub distance: f64,
    pub screen_dx: f32,
    pub screen_dy: f32,
    pub elevation: ElevationIndicator,
    pub color: [u8; 4],
}

/// High-performance entity radar tracker with fair-play competitive rules and visibility modes.
#[derive(Debug, Clone)]
pub struct RadarTracker {
    pub max_range: f64,          // Maximum detection radius (e.g. 64 or 128 blocks)
    pub visibility_mode: RadarVisibilityMode,
    pub fair_play_mode: bool,    // If true, conceals players and hostile mobs on competitive servers
    pub show_hostiles: bool,
    pub show_passives: bool,
    pub show_players: bool,
    pub show_items: bool,
    pub elevation_threshold: f64, // Default 2.5 blocks
}

impl Default for RadarTracker {
    fn default() -> Self {
        Self {
            max_range: 64.0,
            visibility_mode: RadarVisibilityMode::AllLayers,
            fair_play_mode: false,
            show_hostiles: true,
            show_passives: true,
            show_players: true,
            show_items: true,
            elevation_threshold: 2.5,
        }
    }
}

impl RadarTracker {
    pub fn new(max_range: f64) -> Self {
        Self {
            max_range,
            ..Default::default()
        }
    }

    /// Processes world entities and produces filtered, positioned radar contacts relative to the player.
    pub fn update_contacts(
        &self,
        player_x: f64,
        player_y: f64,
        player_z: f64,
        player_yaw: f32,
        rotating_map: bool,
        zoom_scale: f32,
        entities: &[RadarEntity],
    ) -> Vec<RadarContact> {
        let mut contacts = Vec::new();
        let yaw_rad = if rotating_map {
            (player_yaw + 180.0).to_radians()
        } else {
            0.0
        };
        let cos_yaw = yaw_rad.cos() as f64;
        let sin_yaw = yaw_rad.sin() as f64;

        // Player look angle for LineOfSight checking
        // Minecraft yaw: 0 = South (+Z), 90 = West (-X), 180 = North (-Z), 270 = East (+X)
        let player_facing_rad = (player_yaw + 90.0).to_radians() as f64;

        for ent in entities {
            // Apply Fair-play restrictions
            if self.fair_play_mode && matches!(ent.category, EntityCategory::Player | EntityCategory::Hostile) {
                continue;
            }

            // Category visibility filters
            match ent.category {
                EntityCategory::Hostile if !self.show_hostiles => continue,
                EntityCategory::Passive if !self.show_passives => continue,
                EntityCategory::Player if !self.show_players => continue,
                EntityCategory::Item if !self.show_items => continue,
                _ => {}
            }

            let dx = ent.x - player_x;
            let dz = ent.z - player_z;
            let dy = ent.y - player_y;
            let distance = (dx * dx + dz * dz).sqrt();

            if distance > self.max_range {
                continue;
            }

            // Apply Visibility Mode Filters
            match self.visibility_mode {
                RadarVisibilityMode::SurfaceOnly => {
                    if dy.abs() > 5.0 {
                        continue;
                    }
                }
                RadarVisibilityMode::LineOfSight => {
                    // Check if entity is within vertical sight threshold (not buried deep underground)
                    if dy.abs() > 12.0 {
                        continue;
                    }
                    // Check if entity is within forward 120° cone
                    let angle_to_ent = dz.atan2(dx);
                    let mut diff_angle = (angle_to_ent - player_facing_rad).abs();
                    while diff_angle > std::f64::consts::PI {
                        diff_angle = (2.0 * std::f64::consts::PI) - diff_angle;
                    }
                    if diff_angle > (60.0f64.to_radians()) {
                        continue;
                    }
                }
                RadarVisibilityMode::AllLayers => {
                    // Show all without vertical or directional restriction
                }
            }

            // Rotate coordinates if rotating minimap is active
            let (rot_dx, rot_dz) = if rotating_map {
                let rx = dx * cos_yaw - dz * sin_yaw;
                let rz = dx * sin_yaw + dz * cos_yaw;
                (rx, rz)
            } else {
                (dx, dz)
            };

            let screen_dx = (rot_dx * zoom_scale as f64) as f32;
            let screen_dy = (rot_dz * zoom_scale as f64) as f32;

            // Height difference calculation
            let elevation = if dy > self.elevation_threshold {
                ElevationIndicator::Above
            } else if dy < -self.elevation_threshold {
                ElevationIndicator::Below
            } else {
                ElevationIndicator::Level
            };

            // Color tint based on elevation
            let mut color = ent.category.default_color();
            match elevation {
                ElevationIndicator::Above => {
                    // Brighten
                    color[0] = (color[0] as f32 * 1.2).min(255.0) as u8;
                    color[1] = (color[1] as f32 * 1.2).min(255.0) as u8;
                    color[2] = (color[2] as f32 * 1.2).min(255.0) as u8;
                }
                ElevationIndicator::Below => {
                    // Darken
                    color[0] = (color[0] as f32 * 0.7) as u8;
                    color[1] = (color[1] as f32 * 0.7) as u8;
                    color[2] = (color[2] as f32 * 0.7) as u8;
                }
                ElevationIndicator::Level => {}
            }

            contacts.push(RadarContact {
                entity: ent.clone(),
                dx,
                dz,
                distance,
                screen_dx,
                screen_dy,
                elevation,
                color,
            });
        }

        contacts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visibility_modes() {
        let mut tracker = RadarTracker::new(64.0);
        let entities = vec![
            RadarEntity::new(1, "Same Layer", EntityCategory::Passive, 10.0, 64.0, 0.0, 0.0),
            RadarEntity::new(2, "Deep Cave", EntityCategory::Hostile, 10.0, 20.0, 0.0, 0.0),
        ];

        // SurfaceOnly should ignore Deep Cave
        tracker.visibility_mode = RadarVisibilityMode::SurfaceOnly;
        let surface_contacts = tracker.update_contacts(0.0, 64.0, 0.0, 0.0, false, 1.0, &entities);
        assert_eq!(surface_contacts.len(), 1);

        // AllLayers shows both
        tracker.visibility_mode = RadarVisibilityMode::AllLayers;
        let all_contacts = tracker.update_contacts(0.0, 64.0, 0.0, 0.0, false, 1.0, &entities);
        assert_eq!(all_contacts.len(), 2);
    }
}
