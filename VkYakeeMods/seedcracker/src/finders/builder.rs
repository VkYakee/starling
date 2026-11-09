use crate::bedrock::BedrockBlock;
use super::control::FinderControl;

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
impl FinderJob {
    pub fn is_valid(&self) -> bool { !self.dimension.trim().is_empty() }
}

#[derive(Debug, Clone, Copy)]
pub struct FinderBuilder { kind: FinderKind, generation: u64 }
impl Default for FinderBuilder {
    fn default() -> Self { Self { kind: FinderKind::Combined, generation: 0 } }
}
impl FinderBuilder {
    pub fn new(kind: FinderKind) -> Self { Self { kind, generation: 0 } }
    pub fn generation(mut self, generation: u64) -> Self { self.generation = generation; self }
    pub fn controlled(mut self, control: &FinderControl) -> Self { self.generation = control.generation(); self }
    pub fn build(self, dimension: impl Into<String>, chunk_x: i32, chunk_z: i32, bedrock: Vec<BedrockBlock>) -> FinderJob {
        FinderJob { dimension: dimension.into(), chunk_x, chunk_z, kind: self.kind, bedrock, generation: self.generation }
    }
    pub fn build_if_enabled(self, control: &FinderControl, dimension: impl Into<String>, chunk_x: i32, chunk_z: i32, bedrock: Vec<BedrockBlock>) -> Option<FinderJob> {
        if !control.enabled() { return None; }
        let job = self.generation(control.generation()).build(dimension, chunk_x, chunk_z, bedrock);
        job.is_valid().then_some(job)
    }
}
