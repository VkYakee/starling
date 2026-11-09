use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, Ordering}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinderStatus { Idle, Running, Paused, Stopping }

/// Shared lifecycle gate. Every pause/stop invalidates jobs already in flight.
#[derive(Clone)]
pub struct FinderControl {
    enabled: Arc<AtomicBool>,
    stopping: Arc<AtomicBool>,
    generation: Arc<AtomicU64>,
}
impl Default for FinderControl {
    fn default() -> Self {
        Self { enabled: Arc::new(AtomicBool::new(true)), stopping: Arc::new(AtomicBool::new(false)), generation: Arc::new(AtomicU64::new(1)) }
    }
}
impl FinderControl {
    pub fn status(&self) -> FinderStatus {
        if self.stopping.load(Ordering::Acquire) { FinderStatus::Stopping }
        else if self.enabled() { FinderStatus::Running } else { FinderStatus::Paused }
    }
    pub fn enabled(&self) -> bool { self.enabled.load(Ordering::Acquire) && !self.stopping.load(Ordering::Acquire) }
    pub fn generation(&self) -> u64 { self.generation.load(Ordering::Acquire) }
    pub fn is_current(&self, generation: u64) -> bool { self.enabled() && self.generation() == generation }
    pub fn set_enabled(&self, value: bool) {
        self.generation.fetch_add(1, Ordering::AcqRel);
        self.stopping.store(false, Ordering::Release);
        self.enabled.store(value, Ordering::Release);
    }
    pub fn pause(&self) { self.set_enabled(false); }
    pub fn resume(&self) { self.set_enabled(true); }
    /// Invalidates all queued/in-flight jobs without permanently stopping intake.
    pub fn cancel_all(&self) -> u64 { self.generation.fetch_add(1, Ordering::AcqRel) + 1 }
    /// Permanently gates intake until resume() is called.
    pub fn stop(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
        self.enabled.store(false, Ordering::Release);
        self.stopping.store(true, Ordering::Release);
    }
}
