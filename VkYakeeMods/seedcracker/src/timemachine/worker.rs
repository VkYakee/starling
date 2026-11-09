use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};

use super::{CrackerResult, CrackerState, TimeMachine};

/// Generation-gated worker. A stale or cancelled run can never publish results.
#[derive(Clone, Default)]
pub struct SolverWorker {
    generation: Arc<AtomicU64>,
    cancellation: Arc<Mutex<Option<Arc<AtomicBool>>>>,
    published: Arc<Mutex<Option<(u64, CrackerResult)>>>,
}

impl SolverWorker {
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    pub fn cancel(&self) {
        if let Some(token) = self.cancellation.lock().unwrap_or_else(|e| e.into_inner()).take() {
            token.store(true, Ordering::Release);
        }
        self.generation.fetch_add(1, Ordering::AcqRel);
    }

    pub fn start(&self, state: CrackerState) -> JoinHandle<CrackerResult> {
        self.cancel();
        let generation = self.generation.load(Ordering::Acquire);
        let token = Arc::new(AtomicBool::new(false));
        *self.cancellation.lock().unwrap_or_else(|e| e.into_inner()) = Some(token.clone());

        let generation_ref = self.generation.clone();
        let published_ref = self.published.clone();
        thread::spawn(move || {
            let result = TimeMachine::solve_cancellable(&state, token.clone());
            let is_current = generation_ref.load(Ordering::Acquire) == generation;
            if is_current && !token.load(Ordering::Acquire) && !result.cancelled {
                *published_ref.lock().unwrap_or_else(|e| e.into_inner()) =
                    Some((generation, result.clone()));
            }
            result
        })
    }

    pub fn latest(&self) -> Option<CrackerResult> {
        let current = self.generation();
        self.published.lock().unwrap_or_else(|e| e.into_inner())
            .as_ref().filter(|(generation, result)| *generation == current && result.validated)
            .map(|(_, result)| result.clone())
    }

    pub fn clear_published(&self) {
        *self.published.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
}
