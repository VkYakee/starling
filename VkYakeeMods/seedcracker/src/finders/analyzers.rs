use crate::bedrock::{BedrockBlock, BlockType};
use crate::structures::{StructureInstance, StructureType};
use super::observations::{BlockObservation, PatternDetector, PatternKind, FinderObservations};

#[derive(Debug, Clone, Default)]
pub struct AnalyzerOutput { pub bedrock:Vec<BedrockBlock>, pub structures:Vec<StructureInstance>, pub observations:FinderObservations }
pub trait ChunkAnalyzer: Send + Sync { fn analyze(&self, chunk_x:i32, chunk_z:i32, blocks:&[BedrockBlock])->AnalyzerOutput; }
#[derive(Debug,Default,Clone,Copy)] pub struct BedrockAnalyzer;
impl ChunkAnalyzer for BedrockAnalyzer {
 fn analyze(&self,_cx:i32,_cz:i32,blocks:&[BedrockBlock])->AnalyzerOutput {
  AnalyzerOutput{bedrock:blocks.iter().filter(|b|b.block_type==BlockType::Bedrock).copied().collect(),..Default::default()}
 }
}
#[derive(Debug,Default,Clone)] pub struct StructureAnalyzer { detector:PatternDetector }
impl ChunkAnalyzer for StructureAnalyzer {
 fn analyze(&self,cx:i32,cz:i32,blocks:&[BedrockBlock])->AnalyzerOutput {
  // Bedrock-only payload cannot reveal structure palettes; this path is intentionally
  // empty until canonical block snapshots are supplied through scan_blocks().
  let _=(cx,cz,blocks); AnalyzerOutput::default()
 }
}
impl StructureAnalyzer {
 pub fn scan_blocks(&mut self,blocks:&[BlockObservation])->AnalyzerOutput {
  let observations=self.detector.scan(blocks);let mut structures=Vec::new();
  for hit in &observations.patterns {
   let ty=match hit.kind {
    PatternKind::DesertPyramid=>Some(StructureType::DesertPyramid),
    PatternKind::JungleTemple=>Some(StructureType::JungleTemple),
    PatternKind::SwampHut=>Some(StructureType::SwampHut),
    PatternKind::Igloo=>Some(StructureType::Igloo),
    PatternKind::Village=>Some(StructureType::Village),
    PatternKind::Shipwreck=>Some(StructureType::Shipwreck),
    PatternKind::Monument=>Some(StructureType::OceanMonument),
    PatternKind::Outpost=>Some(StructureType::PillagerOutpost),
    PatternKind::EndCity=>Some(StructureType::EndCity),
    PatternKind::TrialChambers=>Some(StructureType::TrialChambers),
    PatternKind::AncientCity=>Some(StructureType::AncientCity),_=>None};
   if let Some(t)=ty {structures.push(StructureInstance::from_block_coords(t,hit.x,hit.z));}
  }
  AnalyzerOutput{structures,observations,..Default::default()}
 }
 pub fn report(instance:StructureInstance)->AnalyzerOutput{AnalyzerOutput{structures:vec![instance],..Default::default()}}
 pub fn supported_types()-> &'static [StructureType] { &[StructureType::DesertPyramid,StructureType::JungleTemple,StructureType::SwampHut,StructureType::Igloo,StructureType::Village,StructureType::Shipwreck,StructureType::OceanMonument,StructureType::PillagerOutpost,StructureType::EndCity,StructureType::TrialChambers,StructureType::AncientCity] }
}
