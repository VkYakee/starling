//! Finder pipeline primitives. Chunk ingestion remains owned by Starling;
//! this module only schedules and analyzes immutable observations.
pub mod builder;
pub mod control;
pub mod queue;
pub mod analyzers;

pub use builder::{FinderBuilder, FinderJob, FinderKind};
pub use control::{FinderControl, FinderStatus};
pub use queue::{FinderQueue, FinderQueueStats, QueuedFinder};
pub use analyzers::{ChunkAnalyzer, AnalyzerOutput, BedrockAnalyzer, StructureAnalyzer};
