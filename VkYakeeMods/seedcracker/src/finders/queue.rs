use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use super::builder::FinderJob;

#[derive(Clone, Default)]
pub struct FinderQueue {
    inner: Arc<Mutex<VecDeque<QueuedFinder>>>,
    next: Arc<AtomicU64>,
    submitted: Arc<AtomicU64>,
    completed: Arc<AtomicU64>,
    cancelled: Arc<AtomicU64>,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FinderQueueStats { pub queued: usize, pub submitted: u64, pub completed: u64, pub cancelled: u64 }
#[derive(Debug, Clone)]
pub struct QueuedFinder { pub sequence: u64, pub job: FinderJob }
impl FinderQueue {
    pub fn push(&self, job: FinderJob) -> u64 {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        self.inner.lock().unwrap_or_else(|e|e.into_inner()).push_back(QueuedFinder { sequence:id, job });
        self.submitted.fetch_add(1, Ordering::Relaxed);
        id
    }
    pub fn pop(&self) -> Option<FinderJob> { self.pop_with_id().map(|q|q.job) }
    pub fn pop_with_id(&self) -> Option<QueuedFinder> { self.inner.lock().unwrap_or_else(|e|e.into_inner()).pop_front() }
    pub fn drain(&self) -> Vec<FinderJob> { self.inner.lock().unwrap_or_else(|e|e.into_inner()).drain(..).map(|q|q.job).collect() }
    pub fn len(&self) -> usize { self.inner.lock().unwrap_or_else(|e|e.into_inner()).len() }
    pub fn is_empty(&self) -> bool { self.len()==0 }
    pub fn clear(&self) -> usize {
        let mut q=self.inner.lock().unwrap_or_else(|e|e.into_inner());
        let n=q.len(); q.clear(); self.cancelled.fetch_add(n as u64,Ordering::Relaxed); n
    }
    pub fn cancel_stale(&self, generation:u64) -> usize {
        let mut q=self.inner.lock().unwrap_or_else(|e|e.into_inner());
        let before=q.len(); q.retain(|item|item.job.generation==generation);
        let removed=before-q.len(); self.cancelled.fetch_add(removed as u64,Ordering::Relaxed); removed
    }
    pub fn mark_completed(&self) { self.completed.fetch_add(1,Ordering::Relaxed); }
    pub fn mark_cancelled(&self) { self.cancelled.fetch_add(1,Ordering::Relaxed); }
    pub fn stats(&self)->FinderQueueStats {
        FinderQueueStats { queued:self.len(), submitted:self.submitted.load(Ordering::Relaxed), completed:self.completed.load(Ordering::Relaxed), cancelled:self.cancelled.load(Ordering::Relaxed) }
    }
}
