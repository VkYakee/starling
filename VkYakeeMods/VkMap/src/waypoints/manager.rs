use super::waypoint::{Dimension, Waypoint};
use serde::{Deserialize, Serialize};

/// High-level Waypoint Manager supporting VoxelMap, XaeroPlus, and JSON storage formats.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WaypointManager {
    waypoints: Vec<Waypoint>,
}

impl WaypointManager {
    pub fn new() -> Self {
        Self {
            waypoints: Vec::new(),
        }
    }

    /// Adds a waypoint (replaces existing if same ID).
    pub fn add(&mut self, wp: Waypoint) {
        if let Some(pos) = self.waypoints.iter().position(|w| w.id == wp.id) {
            self.waypoints[pos] = wp;
        } else {
            self.waypoints.push(wp);
        }
    }

    /// Removes a waypoint by ID.
    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(pos) = self.waypoints.iter().position(|w| w.id == id) {
            self.waypoints.remove(pos);
            true
        } else {
            false
        }
    }

    /// Toggles the enabled state of a waypoint.
    pub fn toggle(&mut self, id: &str) -> bool {
        if let Some(wp) = self.waypoints.iter_mut().find(|w| w.id == id) {
            wp.enabled = !wp.enabled;
            true
        } else {
            false
        }
    }

    pub fn get(&self, id: &str) -> Option<&Waypoint> {
        self.waypoints.iter().find(|w| w.id == id)
    }

    pub fn list_all(&self) -> &[Waypoint] {
        &self.waypoints
    }

    /// Filters enabled waypoints in a specific dimension.
    pub fn get_active_in_dimension(&self, dim: &Dimension) -> Vec<&Waypoint> {
        self.waypoints
            .iter()
            .filter(|w| w.enabled && &w.dimension == dim)
            .collect()
    }

    /// Searches waypoints by substring match on name.
    pub fn search(&self, query: &str) -> Vec<&Waypoint> {
        let q = query.trim().to_ascii_lowercase();
        self.waypoints
            .iter()
            .filter(|w| w.name.to_ascii_lowercase().contains(&q))
            .collect()
    }

    /// Automatically records a death point, disabling previous death markers.
    pub fn create_death_point(&mut self, x: i32, y: i32, z: i32, dim: Dimension) -> &Waypoint {
        // Demote existing death points
        for wp in &mut self.waypoints {
            if wp.is_death_point {
                wp.enabled = false;
            }
        }

        let mut death_wp = Waypoint::new("Death Point", x, y, z, dim, [255, 60, 60]);
        death_wp.icon = "skull".to_string();
        death_wp.is_death_point = true;
        death_wp.enabled = true;

        self.waypoints.push(death_wp);
        self.waypoints.last().unwrap()
    }

    /// Returns the currently active death point if recorded.
    pub fn get_active_death_point(&self) -> Option<&Waypoint> {
        self.waypoints.iter().find(|w| w.is_death_point && w.enabled)
    }

    /// Clears all waypoints.
    pub fn clear(&mut self) {
        self.waypoints.clear();
    }

    /// Exports all waypoints to VoxelMap formatted text lines.
    pub fn export_voxelmap_format(&self) -> String {
        let mut out = String::new();
        for wp in &self.waypoints {
            let r = wp.color[0] as f32 / 255.0;
            let g = wp.color[1] as f32 / 255.0;
            let b = wp.color[2] as f32 / 255.0;
            let dim_str = wp.dimension.as_str();

            out.push_str(&format!(
                "name:{},x:{},z:{},y:{},enabled:{},red:{:.2},green:{:.2},blue:{:.2},suffix:{},world:,dimensions:{}#\n",
                wp.name, wp.x, wp.z, wp.y, wp.enabled, r, g, b, wp.icon, dim_str
            ));
        }
        out
    }

    /// Imports waypoints from VoxelMap format string.
    pub fn import_voxelmap_format(&mut self, content: &str) -> usize {
        let mut count = 0;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let mut name = String::new();
            let mut x = 0;
            let mut y = 64;
            let mut z = 0;
            let mut enabled = true;
            let mut r = 1.0f32;
            let mut g = 1.0f32;
            let mut b = 1.0f32;
            let mut icon = "diamond".to_string();
            let mut dim = Dimension::Overworld;

            for token in trimmed.split(',') {
                if let Some((k, v)) = token.split_once(':') {
                    match k.trim() {
                        "name" => name = v.trim().to_string(),
                        "x" => x = v.trim().parse().unwrap_or(0),
                        "y" => y = v.trim().parse().unwrap_or(64),
                        "z" => z = v.trim().parse().unwrap_or(0),
                        "enabled" => enabled = v.trim().parse().unwrap_or(true),
                        "red" => r = v.trim().parse().unwrap_or(1.0),
                        "green" => g = v.trim().parse().unwrap_or(1.0),
                        "blue" => b = v.trim().parse().unwrap_or(1.0),
                        "suffix" => {
                            let s = v.trim();
                            if !s.is_empty() {
                                icon = s.to_string();
                            }
                        }
                        "dimensions" => {
                            let d = v.trim().trim_end_matches('#');
                            dim = Dimension::from_id(d);
                        }
                        _ => {}
                    }
                }
            }

            if !name.is_empty() {
                let color = [
                    (r.clamp(0.0, 1.0) * 255.0) as u8,
                    (g.clamp(0.0, 1.0) * 255.0) as u8,
                    (b.clamp(0.0, 1.0) * 255.0) as u8,
                ];
                let mut wp = Waypoint::new(name, x, y, z, dim, color);
                wp.enabled = enabled;
                wp.icon = icon;
                self.add(wp);
                count += 1;
            }
        }
        count
    }

    /// Serializes to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.waypoints)
    }

    /// Deserializes from JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let waypoints = serde_json::from_str(json)?;
        Ok(Self { waypoints })
    }

    pub fn len(&self) -> usize {
        self.waypoints.len()
    }

    pub fn is_empty(&self) -> bool {
        self.waypoints.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waypoint_manager_lifecycle() {
        let mut mgr = WaypointManager::new();
        let wp1 = Waypoint::new("Base", 0, 64, 0, Dimension::Overworld, [0, 255, 0]);
        let wp2 = Waypoint::new("Nether Fortress", 100, 60, 200, Dimension::Nether, [255, 0, 0]);

        mgr.add(wp1);
        mgr.add(wp2);

        assert_eq!(mgr.len(), 2);
        assert_eq!(mgr.get_active_in_dimension(&Dimension::Overworld).len(), 1);
        assert_eq!(mgr.get_active_in_dimension(&Dimension::Nether).len(), 1);

        // Auto death point
        mgr.create_death_point(50, 40, -80, Dimension::Overworld);
        assert_eq!(mgr.len(), 3);
        assert_eq!(mgr.search("Death").len(), 1);
    }

    #[test]
    fn test_voxelmap_import_export() {
        let mut mgr = WaypointManager::new();
        let wp = Waypoint::new("Portal", 250, 70, -120, Dimension::Overworld, [128, 0, 255]);
        mgr.add(wp);

        let exported = mgr.export_voxelmap_format();
        assert!(exported.contains("name:Portal"));
        assert!(exported.contains("x:250"));

        let mut mgr2 = WaypointManager::new();
        let imported = mgr2.import_voxelmap_format(&exported);
        assert_eq!(imported, 1);
        assert_eq!(mgr2.len(), 1);
        assert_eq!(mgr2.list_all()[0].name, "Portal");
    }
}
