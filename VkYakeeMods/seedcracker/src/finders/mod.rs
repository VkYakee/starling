//! Finder pipeline: immutable chunk jobs, generation-based cancellation and analyzers.
pub mod builder;
pub mod control;
pub mod queue;
pub mod analyzers;\npub mod observations;

use crate::bedrock::BedrockBlock;
use builder::{FinderJob, FinderKind, FinderBuilder};
use analyzers::{AnalyzerOutput, BedrockAnalyzer, ChunkAnalyzer, StructureAnalyzer};
use control::FinderControl;
use queue::{FinderQueue, FinderQueueStats};

pub use builder::{FinderBuilder as JobBuilder, FinderJob as Job, FinderKind as Kind};
pub use control::{FinderControl as Control, FinderStatus};
pub use queue::{FinderQueue as Queue, FinderQueueStats as QueueStats, QueuedFinder};
pub use analyzers::{ChunkAnalyzer as Analyzer, AnalyzerOutput as Output, BedrockAnalyzer as Bedrock, StructureAnalyzer as Structures};\npub use observations::{BlockObservation, BlockUpdateBatch, FinderObservations, PatternDetector, PatternHit, PatternKind, FinderCategory, FinderType, BlockFinder, FinderRegistry};

/// Owns scheduling and execution policy; Starling remains responsible for decoding chunks.
#[derive(Clone, Default)]
pub struct FinderPipeline {
    pub queue: FinderQueue,
    pub control: FinderControl,
}
impl FinderPipeline {
    pub fn scan_blocks(&self, blocks: &[BlockObservation]) -> AnalyzerOutput {\n        let mut detector=self.detector.lock().unwrap_or_else(|e|e.into_inner());\n        let mut observations=detector.scan(blocks);
        let registry=self.registry.lock().unwrap_or_else(|e|e.into_inner());
        observations.patterns.retain(|hit|registry.enabled(match hit.kind {
            observations::PatternKind::BuriedTreasure=>observations::FinderType::BuriedTreasure,
            observations::PatternKind::DesertPyramid=>observations::FinderType::DesertPyramid,
            observations::PatternKind::JungleTemple=>observations::FinderType::JungleTemple,
            observations::PatternKind::SwampHut=>observations::FinderType::SwampHut,
            observations::PatternKind::Igloo=>observations::FinderType::Igloo,
            observations::PatternKind::Village=>observations::FinderType::Biome,
            observations::PatternKind::Shipwreck=>observations::FinderType::Shipwreck,
            observations::PatternKind::Monument=>observations::FinderType::Monument,
            observations::PatternKind::Outpost=>observations::FinderType::Outpost,
            observations::PatternKind::EndCity=>observations::FinderType::EndCity,
            observations::PatternKind::TrialChambers=>observations::FinderType::TrialChambers,
            observations::PatternKind::AncientCity=>observations::FinderType::Monument,
            observations::PatternKind::Dungeon=>observations::FinderType::Dungeon,
            observations::PatternKind::DesertWell=>observations::FinderType::DesertWell,
            observations::PatternKind::EmeraldOre=>observations::FinderType::EmeraldOre,
            observations::PatternKind::EndGateway=>observations::FinderType::EndGateway,
            observations::PatternKind::WarpedFungus=>observations::FinderType::WarpedFungus,
            observations::PatternKind::EndPillar=>observations::FinderType::EndPillars,
        }));
        let mut structures=Vec::new();\n        for hit in &observations.patterns {\n            let ty=match hit.kind {\n                observations::PatternKind::DesertPyramid=>Some(crate::structures::StructureType::DesertPyramid),\n                observations::PatternKind::JungleTemple=>Some(crate::structures::StructureType::JungleTemple),\n                observations::PatternKind::SwampHut=>Some(crate::structures::StructureType::SwampHut),\n                observations::PatternKind::Igloo=>Some(crate::structures::StructureType::Igloo),\n                observations::PatternKind::Village=>Some(crate::structures::StructureType::Village),\n                observations::PatternKind::Shipwreck=>Some(crate::structures::StructureType::Shipwreck),\n                observations::PatternKind::Monument=>Some(crate::structures::StructureType::OceanMonument),\n                observations::PatternKind::Outpost=>Some(crate::structures::StructureType::PillagerOutpost),\n                observations::PatternKind::EndCity=>Some(crate::structures::StructureType::EndCity),\n                observations::PatternKind::TrialChambers=>Some(crate::structures::StructureType::TrialChambers),\n                observations::PatternKind::AncientCity=>Some(crate::structures::StructureType::AncientCity),_=>None};\n            if let Some(t)=ty {structures.push(crate::structures::StructureInstance::from_block_coords(t,hit.x,hit.z));}\n        }\n        AnalyzerOutput{structures,observations,..Default::default()}\n    }\n    pub fn apply_block_updates(&self, batch: observations::BlockUpdateBatch) { self.detector.lock().unwrap_or_else(|e|e.into_inner()).apply_updates(batch); }\n    pub fn set_finder_enabled(&self,kind:FinderType,enabled:bool){self.registry.lock().unwrap_or_else(|e|e.into_inner()).set_enabled(kind,enabled);self.reset_finders();}\n    pub fn reload_finders(&self) {self.registry.lock().unwrap_or_else(|e|e.into_inner()).reload();self.reset_finders();}\n    pub fn reset_finders(&self) { self.control.cancel_all(); self.queue.clear(); self.detector.lock().unwrap_or_else(|e|e.into_inner()).reset(); }\n    pub fn enqueue(&self, job: FinderJob) -> Option<u64> {
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
