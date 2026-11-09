use crate::bedrock::BedrockBlock;

/// Explicitly describes independent finder work without copying Starling's
/// chunk decoder/streaming implementation into SeedCracker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinderKind { Bedrock, Structure, Combined }

#[derive(Debug, Clone)]
pub struct FinderJob {
    pub dimension: String,
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub kind: FinderKind,
    pub bedrock: Vec<BedrockBlock>,
    pub generation: u64,
}

#[derive(Default)]
pub struct FinderBuilder { kind: FinderKind, generation: u64 }
impl Default for FinderKind { fn default() -> Self { FinderKind::Combined } }
impl FinderBuilder {
    pub fn new(kind: FinderKind) -> Self { Self { kind, generation: 0 } }
    pub fn generation(mut self, generation: u64) -> Self { self.generation = generation; self }
    pub fn build(self, dimension: impl Into<String>, chunk_x: i32, chunk_z: i32, bedrock: Vec<BedrockBlock>) -> FinderJob {
        FinderJob { dimension: dimension.into(), chunk_x, chunk_z, kind: self.kind, bedrock, generation: self.generation }
    }
}
