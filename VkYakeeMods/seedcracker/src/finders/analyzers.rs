use crate::bedrock::{BedrockBlock, BlockType};
use crate::structures::{StructureInstance, StructureType};

#[derive(Debug, Clone, Default)]
pub struct AnalyzerOutput {
    pub bedrock: Vec<BedrockBlock>,
    pub structures: Vec<StructureInstance>,
}
pub trait ChunkAnalyzer: Send + Sync { fn analyze(&self, chunk_x: i32, chunk_z: i32, blocks: &[BedrockBlock]) -> AnalyzerOutput; }

/// Pure bedrock extractor. The original block observations are preserved;
/// no chunk parsing or storage responsibilities are duplicated here.
#[derive(Debug, Default, Clone, Copy)]
pub struct BedrockAnalyzer;
impl ChunkAnalyzer for BedrockAnalyzer {
    fn analyze(&self, _cx: i32, _cz: i32, blocks: &[BedrockBlock]) -> AnalyzerOutput {
        AnalyzerOutput { bedrock: blocks.iter().filter(|b| b.block_type == BlockType::Bedrock).cloned().collect(), structures: Vec::new() }
    }
}

/// Structure analyzer placeholder boundary: consumes decoded observations and
/// returns independent findings. Structure-specific parsers can implement this
/// trait without touching the game receiver or streaming cache.
#[derive(Debug, Default, Clone, Copy)]
pub struct StructureAnalyzer;
impl ChunkAnalyzer for StructureAnalyzer {
    fn analyze(&self, _cx: i32, _cz: i32, _blocks: &[BedrockBlock]) -> AnalyzerOutput {
        AnalyzerOutput::default()
    }
}
impl StructureAnalyzer {
    pub fn report(instance: StructureInstance) -> AnalyzerOutput {
        AnalyzerOutput { bedrock: Vec::new(), structures: vec![instance] }
    }
    pub fn supported_types() -> &'static [StructureType] {
        &[StructureType::DesertPyramid, StructureType::JungleTemple, StructureType::SwampHut,
          StructureType::Igloo, StructureType::Village, StructureType::Shipwreck,
          StructureType::OceanMonument, StructureType::PillagerOutpost, StructureType::EndCity,
          StructureType::TrialChambers, StructureType::AncientCity]
}
