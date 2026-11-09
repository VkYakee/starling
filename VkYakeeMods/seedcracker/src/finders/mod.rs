//! Finder pipeline: immutable chunk jobs, generation-based cancellation and analyzers.
pub mod builder;
pub mod control;
pub mod queue;
pub mod analyzers;
pub mod observations;

use crate::bedrock::BedrockBlock;
use std::sync::{Arc, Mutex};

pub use builder::{FinderBuilder, FinderJob, FinderKind};
pub use control::{FinderControl, FinderStatus};
pub use queue::{FinderQueue, FinderQueueStats, QueuedFinder};
pub use analyzers::{ChunkAnalyzer, AnalyzerOutput, BedrockAnalyzer, StructureAnalyzer};
pub use observations::{BlockObservation, BlockUpdateBatch, FinderObservations, PatternDetector, PatternHit, PatternKind, FinderCategory, FinderType, BlockFinder, FinderRegistry};

#[derive(Clone, Default)]
pub struct FinderPipeline {
    pub queue: FinderQueue,
    pub control: FinderControl,
    pub detector: Arc<Mutex<PatternDetector>>,
    pub registry: Arc<Mutex<FinderRegistry>>,
}
impl FinderPipeline {
    pub fn scan_blocks(&self, blocks: &[BlockObservation]) -> AnalyzerOutput {
        let mut detector=self.detector.lock().unwrap_or_else(|e|e.into_inner());
        let mut observations=detector.scan(blocks);
        let registry=self.registry.lock().unwrap_or_else(|e|e.into_inner());
        observations.patterns.retain(|hit| registry.enabled(match hit.kind {
            PatternKind::BuriedTreasure=>FinderType::BuriedTreasure,
            PatternKind::DesertPyramid=>FinderType::DesertPyramid,
            PatternKind::JungleTemple=>FinderType::JungleTemple,
            PatternKind::SwampHut=>FinderType::SwampHut,
            PatternKind::Igloo=>FinderType::Igloo,
            PatternKind::Village=>FinderType::Village,
            PatternKind::Shipwreck=>FinderType::Shipwreck,
            PatternKind::Monument=>FinderType::Monument,
            PatternKind::Outpost=>FinderType::Outpost,
            PatternKind::EndCity=>FinderType::EndCity,
            PatternKind::TrialChambers=>FinderType::TrialChambers,
            PatternKind::AncientCity=>FinderType::AncientCity,
            PatternKind::Dungeon=>FinderType::Dungeon,
            PatternKind::DesertWell=>FinderType::DesertWell,
            PatternKind::EmeraldOre=>FinderType::EmeraldOre,
            PatternKind::EndGateway=>FinderType::EndGateway,
            PatternKind::WarpedFungus=>FinderType::WarpedFungus,
            PatternKind::EndPillar=>FinderType::EndPillars,
        }));
        let mut structures=Vec::new();
        for hit in &observations.patterns {
            let ty=match hit.kind {
                PatternKind::DesertPyramid=>Some(crate::structures::StructureType::DesertPyramid),
                PatternKind::JungleTemple=>Some(crate::structures::StructureType::JungleTemple),
                PatternKind::SwampHut=>Some(crate::structures::StructureType::SwampHut),
                PatternKind::Igloo=>Some(crate::structures::StructureType::Igloo),
                PatternKind::Village=>Some(crate::structures::StructureType::Village),
                PatternKind::Shipwreck=>Some(crate::structures::StructureType::Shipwreck),
                PatternKind::Monument=>Some(crate::structures::StructureType::OceanMonument),
                PatternKind::Outpost=>Some(crate::structures::StructureType::PillagerOutpost),
                PatternKind::EndCity=>Some(crate::structures::StructureType::EndCity),
                PatternKind::TrialChambers=>Some(crate::structures::StructureType::TrialChambers),
                PatternKind::AncientCity=>Some(crate::structures::StructureType::AncientCity),
                _=>None
            };
            if let Some(t)=ty {structures.push(crate::structures::StructureInstance::from_block_coords(t,hit.x,hit.z));}
        }
        AnalyzerOutput{structures,observations,..Default::default()}
    }
    pub fn finder_enabled(&self,kind:FinderType)->bool{self.registry.lock().unwrap_or_else(|e|e.into_inner()).enabled(kind)}
    pub fn apply_block_updates(&self,batch:BlockUpdateBatch){self.detector.lock().unwrap_or_else(|e|e.into_inner()).apply_updates(batch);}
    pub fn set_finder_enabled(&self,kind:FinderType,enabled:bool){self.registry.lock().unwrap_or_else(|e|e.into_inner()).set_enabled(kind,enabled);self.reset_finders();}
    pub fn reload_finders(&self){self.registry.lock().unwrap_or_else(|e|e.into_inner()).reload();self.reset_finders();}
    pub fn reset_finders(&self){self.control.cancel_all();self.queue.clear();self.detector.lock().unwrap_or_else(|e|e.into_inner()).reset();}
    pub fn enqueue(&self,job:FinderJob)->Option<u64>{if !self.control.is_current(job.generation)||!job.is_valid(){return None;}Some(self.queue.push(job))}
    pub fn enqueue_chunk(&self,kind:FinderKind,dimension:impl Into<String>,cx:i32,cz:i32,bedrock:Vec<BedrockBlock>)->Option<u64>{
        let job=FinderBuilder::new(kind).controlled(&self.control).build(dimension,cx,cz,bedrock);self.enqueue(job)
    }
    pub fn run_one(&self)->Option<(u64,AnalyzerOutput)>{
        let queued=self.queue.pop_with_id()?;
        if !self.control.is_current(queued.job.generation){self.queue.mark_cancelled();return None;}
        let job=queued.job;let mut output=AnalyzerOutput::default();
        match job.kind {
            FinderKind::Bedrock=>output=BedrockAnalyzer.analyze(job.chunk_x,job.chunk_z,&job.bedrock),
            FinderKind::Structure=>output=StructureAnalyzer::default().analyze(job.chunk_x,job.chunk_z,&job.bedrock),
            FinderKind::Combined=>{output=BedrockAnalyzer.analyze(job.chunk_x,job.chunk_z,&job.bedrock);if self.control.is_current(job.generation){output.structures.extend(StructureAnalyzer::default().analyze(job.chunk_x,job.chunk_z,&job.bedrock).structures);}}
        }
        if !self.control.is_current(job.generation){self.queue.mark_cancelled();return None;}
        self.queue.mark_completed();Some((queued.sequence,output))
    }
    pub fn cancel_pending(&self)->usize{self.queue.cancel_stale(self.control.cancel_all())}
    pub fn pause(&self)->usize{self.control.pause();self.queue.clear()}
    pub fn resume(&self){self.control.resume();}
    pub fn stop(&self)->usize{self.control.stop();self.queue.clear()}
    pub fn stats(&self)->FinderQueueStats{self.queue.stats()}
}
