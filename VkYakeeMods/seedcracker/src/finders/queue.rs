use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use super::builder::FinderJob;

/// FIFO with stable monotonically increasing IDs (not queue-length IDs).
#[derive(Clone, Default)]
pub struct FinderQueue { inner: Arc<Mutex<VecDeque<QueuedFinder>>>, next: Arc<AtomicU64> }
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FinderQueueStats { pub queued: usize, pub submitted: u64, pub completed: u64 }
#[derive(Debug, Clone)]
pub struct QueuedFinder { pub sequence: u64, pub job: FinderJob }
impl FinderQueue {
 pub fn push(&self, job: FinderJob)->u64 { let id=self.next.fetch_add(1,Ordering::Relaxed); self.inner.lock().unwrap_or_else(|e|e.into_inner()).push_back(QueuedFinder{sequence:id,job}); id }
 pub fn pop(&self)->Option<FinderJob>{self.inner.lock().unwrap_or_else(|e|e.into_inner()).pop_front().map(|q|q.job)}
 pub fn pop_with_id(&self)->Option<QueuedFinder>{self.inner.lock().unwrap_or_else(|e|e.into_inner()).pop_front()}
 pub fn drain(&self)->Vec<FinderJob>{self.inner.lock().unwrap_or_else(|e|e.into_inner()).drain(..).map(|q|q.job).collect()}
 pub fn len(&self)->usize{self.inner.lock().unwrap_or_else(|e|e.into_inner()).len()}
 pub fn is_empty(&self)->bool{self.len()==0}
 pub fn clear(&self){self.inner.lock().unwrap_or_else(|e|e.into_inner()).clear();}
}