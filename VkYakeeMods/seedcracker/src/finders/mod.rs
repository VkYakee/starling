//! Finder pipeline: immutable chunk jobs, generation-based cancellation and analyzers.
pub mod builder;
pub mod control;
pub mod queue;
pub mod analyzers;

use crate::bedrock::BedrockBlock;
use builder::{FinderJob, FinderKind, FinderBuilder};
use analyzers::{AnalyzerOutput, BedrockAnalyzer, ChunkAnalyzer, StructureAnalyzer};
use control::FinderControl;
use queue::{FinderQueue, FinderQueueStats};

pub use builder::{FinderBuilder as JobBuilder, FinderJob as Job, FinderKind as Kind};
pub use control::{FinderControl as Control, FinderStatus};
pub use queue::{FinderQueue as Queue, FinderQueueStats as QueueStats, QueuedFinder};
pub use analyzers::{ChunkAnalyzer as Analyzer, AnalyzerOutput as Output, BedrockAnalyzer as Bedrock, StructureAnalyzer as Structures};

/// Owns scheduling and execution policy; Starling remains responsible for decoding chunks.
#[derive(Clone, Default)]
pub struct FinderPipeline {
    pub queue: FinderQueue,
    pub control: FinderControl,
}
impl FinderPipeline {
    pub fn enqueue(&self, job: FinderJob) -> Option<u64> {
        if !self.control.is_current(job.generation) || !job.is_valid() { return None; }
        Some(self.queue.push(job))
    }
    pub fn enqueue_chunk(&self, kind: FinderKind, dimension: impl Into<String>, cx:i32, cz:i32, bedrock:Vec<BedrockBlock>) -> Option<u64> {
        let job=FinderBuilder::new(kind).controlled(&self.control).build(dimension,cx,cz,bedrock);
        self.enqueue(job)
    }
    /// Executes one queued job. Returns None for cancelled/stale work.
    pub fn run_one(&self) -> Option<(u64, AnalyzerOutput)> {
        let queued=self.queue.pop_with_id()?;
        if !self.control.is_current(queued.job.generation) { self.queue.mark_cancelled(); return None; }
        let job=queued.job;
        let mut output=AnalyzerOutput::default();
        match job.kind {
            FinderKind::Bedrock => output=BedrockAnalyzer.analyze(job.chunk_x,job.chunk_z,&job.bedrock),
            FinderKind::Structure => output=StructureAnalyzer.analyze(job.chunk_x,job.chunk_z,&job.bedrock),
            FinderKind::Combined => {
                output=BedrockAnalyzer.analyze(job.chunk_x,job.chunk_z,&job.bedrock);
                if self.control.is_current(job.generation) {
                    let structures=StructureAnalyzer.analyze(job.chunk_x,job.chunk_z,&job.bedrock);
                    output.structures.extend(structures.structures);
                }
            }
        }
        if !self.control.is_current(job.generation) { self.queue.mark_cancelled(); return None; }
        self.queue.mark_completed();
        Some((queued.sequence,output))
    }
    pub fn cancel_pending(&self)->usize { self.queue.cancel_stale(self.control.cancel_all()) }
    pub fn pause(&self)->usize { self.control.pause(); self.queue.clear() }
    pub fn resume(&self) { self.control.resume(); }
    pub fn stop(&self)->usize { self.control.stop(); self.queue.clear() }
    pub fn stats(&self)->FinderQueueStats { self.queue.stats() }
}
