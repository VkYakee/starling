// Copyright (c) 2026 VkYakee
// VkYakee VoxelMap x seedcrackerX
//! Bedrock Edition Touch UI layout, touch buttons, and gesture detection for Android.

use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchAction {
    ToggleBar,
    CycleMapMode,
    ToggleCave,
    QuickWaypoint,
    ToggleRadar,
    OpenCrackerModal,
    CloseCrackerModal,
    ForceCrack,
    CycleVersion,
    ResetData,
}

#[derive(Clone, Debug)]
pub struct TouchButton {
    pub id: &'static str,
    pub rect: [f32; 4], // [x, y, w, h]
    pub label: String,
    pub active: bool,
    pub action: TouchAction,
}

impl TouchButton {
    pub fn contains(&self, cursor: [f32; 2]) -> bool {
        let [x, y, w, h] = self.rect;
        cursor[0] >= x && cursor[0] <= x + w && cursor[1] >= y && cursor[1] <= y + h
    }
}

pub struct TouchUiState {
    pub bar_open: bool,
    pub modal_open: bool,
    pub map_mode: u8, // 0: Normal, 1: Big WorldMap, 2: Hidden
    pub buttons: Vec<TouchButton>,
    pub modal_buttons: Vec<TouchButton>,
    pub feedback_message: Option<(String, Instant)>,
}

impl TouchUiState {
    pub fn new() -> Self {
        Self {
            bar_open: false,
            modal_open: false,
            map_mode: 0,
            buttons: Vec::new(),
            modal_buttons: Vec::new(),
            feedback_message: None,
        }
    }

    pub fn set_feedback(&mut self, msg: impl Into<String>) {
        self.feedback_message = Some((msg.into(), Instant::now()));
    }

    pub fn current_feedback(&self) -> Option<&str> {
        if let Some((msg, at)) = &self.feedback_message {
            if at.elapsed() < Duration::from_secs(3) {
                return Some(msg.as_str());
            }
        }
        None
    }
}
