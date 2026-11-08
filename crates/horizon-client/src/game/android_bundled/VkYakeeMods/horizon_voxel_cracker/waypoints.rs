// Copyright (c) 2026 VkYakee
// VkYakee VoxelMap x seedcrackerX
//! Waypoint management with per-world isolated storage in /sdcard/HorizonVoxelCracker/.

use super::storage;
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, PoisonError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaypointKind {
    User,
    Death,
    Structure,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Waypoint {
    pub id: String,
    pub name: String,
    pub pos: [i32; 3],
    pub dim: String,
    pub color: u32,
    pub enabled: bool,
    pub kind: WaypointKind,
}

impl Waypoint {
    pub fn new(name: impl Into<String>, pos: [i32; 3], dim: impl Into<String>, color: u32) -> Self {
        let name = name.into();
        let id = format!("{}_{}_{}_{}", name, pos[0], pos[1], pos[2]);
        Self {
            id,
            name,
            pos,
            dim: dim.into(),
            color,
            enabled: true,
            kind: WaypointKind::User,
        }
    }

    pub fn death(pos: [i32; 3], dim: impl Into<String>) -> Self {
        Self {
            id: "auto_death".into(),
            name: "Death Point".into(),
            pos,
            dim: dim.into(),
            color: 0xFFFF_3333,
            enabled: true,
            kind: WaypointKind::Death,
        }
    }

    pub fn structure(name: impl Into<String>, pos: [i32; 3], dim: impl Into<String>) -> Self {
        let name = name.into();
        let id = format!("struct_{}_{}_{}", name, pos[0], pos[2]);
        Self {
            id,
            name,
            pos,
            dim: dim.into(),
            color: 0xFF55_FFFF,
            enabled: true,
            kind: WaypointKind::Structure,
        }
    }

    pub fn distance_from(&self, x: f64, y: f64, z: f64) -> f64 {
        let dx = self.pos[0] as f64 - x;
        let dy = self.pos[1] as f64 - y;
        let dz = self.pos[2] as f64 - z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

pub struct WaypointManager {
    current_world_id: Mutex<String>,
    list: Mutex<Vec<Waypoint>>,
}

impl WaypointManager {
    pub fn new() -> Self {
        let default_id = "default_world";
        let loaded = storage::load_world_waypoints(default_id);
        Self {
            current_world_id: Mutex::new(default_id.into()),
            list: Mutex::new(loaded),
        }
    }

    pub fn switch_world(&self, world_id: &str) {
        let mut cur = self.current_world_id.lock().unwrap();
        if *cur == world_id {
            return;
        }
        *cur = world_id.to_string();
        let loaded = storage::load_world_waypoints(world_id);
        *self.list.lock().unwrap_or_else(PoisonError::into_inner) = loaded;
    }

    pub fn all(&self) -> Vec<Waypoint> {
        self.list.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }

    pub fn add(&self, wp: Waypoint) {
        let mut list = self.list.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(existing) = list.iter_mut().find(|w| w.id == wp.id) {
            *existing = wp;
        } else {
            list.push(wp);
        }
        let wid = self.current_world_id.lock().unwrap().clone();
        storage::save_world_waypoints(&wid, &*list);
    }

    pub fn remove(&self, name_or_id: &str) -> bool {
        let mut list = self.list.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(pos) = list.iter().position(|w| w.id == name_or_id || w.name.eq_ignore_ascii_case(name_or_id)) {
            list.remove(pos);
            let wid = self.current_world_id.lock().unwrap().clone();
            storage::save_world_waypoints(&wid, &*list);
            true
        } else {
            false
        }
    }

    pub fn toggle(&self, name_or_id: &str) -> Option<bool> {
        let mut list = self.list.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(wp) = list.iter_mut().find(|w| w.id == name_or_id || w.name.eq_ignore_ascii_case(name_or_id)) {
            wp.enabled = !wp.enabled;
            let status = wp.enabled;
            let wid = self.current_world_id.lock().unwrap().clone();
            storage::save_world_waypoints(&wid, &*list);
            Some(status)
        } else {
            None
        }
    }

    pub fn clear_user(&self) {
        let mut list = self.list.lock().unwrap_or_else(PoisonError::into_inner);
        list.retain(|w| w.kind != WaypointKind::User);
        let wid = self.current_world_id.lock().unwrap().clone();
        storage::save_world_waypoints(&wid, &*list);
    }

    pub fn on_death(&self, pos: [i32; 3], dim: &str) {
        let death_wp = Waypoint::death(pos, dim);
        self.add(death_wp);
    }

    /// Reset transient memory cache on world exit without modifying storage
    pub fn reset_memory(&self) {
        self.list.lock().unwrap_or_else(PoisonError::into_inner).clear();
        *self.current_world_id.lock().unwrap() = String::new();
    }
}
