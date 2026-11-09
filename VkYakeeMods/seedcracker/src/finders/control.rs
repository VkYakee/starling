use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, Ordering}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinderStatus { Idle, Running, Paused, Stopping }

/// Lifecycle/cancellation control separated from queueing and analysis.
#[derive(Clone)]
pub struct FinderControl {
    enabled: Arc<AtomicBool>,
    generation: Arc<AtomicU64>,
}
impl Default for FinderControl {
    fn default() -> Self { Self { enabled: Arc::new(AtomicBool::new(true)), generation: Arc::new(AtomicU64::new(0)) } }
}
impl FinderControl {
    pub fn status(&self) -> FinderStatus { if self.enabled() { FinderStatus::Running } else { FinderStatus::Paused } }
    pub fn enabled(&self) -> bool { self.enabled.load(Ordering::Acquire) }
    pub fn set_enabled(&self, value: bool) { self.enabled.store(value, Ordering::Release); self.generation.fetch_add(1, Ordering::AcqRel); }
    pub fn pause(&self) { self.set_enabled(false); }
    pub fn resume(&self) { self.set_enabled(true); }
    pub fn generation(&self) -> u64 { self.generation.load(Ordering::Acquire) }
    pub fn is_current(&self, generation: u64) -> bool { self.generation() == generation && self.enabled() }
    pub fn cancel_all(&self) { self.generation.fetch_add(1, Ordering::AcqRel); }
}
