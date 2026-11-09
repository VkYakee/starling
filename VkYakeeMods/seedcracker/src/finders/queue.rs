use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use super::builder::FinderJob;

/// Thread-safe FIFO replacing SeedCracker's FinderQueue concept.
/// Jobs are immutable snapshots; queue never owns or mutates Starling chunks.
#[derive(Clone, Default)]
pub struct FinderQueue { inner: Arc<Mutex<VecDeque<FinderJob>>> }

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FinderQueueStats { pub queued: usize, pub submitted: u64, pub completed: u64 }

#[derive(Debug, Clone)]
pub struct QueuedFinder { pub sequence: u64, pub job: FinderJob }

impl FinderQueue {
    pub fn push(&self, job: FinderJob) -> u64 {
        let mut q = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let sequence = q.len() as u64;
        q.push_back(job);
        sequence
    }
    pub fn pop(&self) -> Option<FinderJob> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner()).pop_front()
    }
    pub fn drain(&self) -> Vec<FinderJob> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner()).drain(..).collect()
    }
    pub fn len(&self) -> usize { self.inner.lock().unwrap_or_else(|e| e.into_inner()).len() }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
    pub fn clear(&self) { self.inner.lock().unwrap_or_else(|e| e.into_inner()).clear(); }
}
