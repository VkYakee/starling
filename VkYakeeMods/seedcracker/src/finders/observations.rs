use std::collections::{HashMap, HashSet};
use crate::structures::{StructureInstance, StructureType};
use crate::decorators::{DesertWellData, EmeraldOreData};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockObservation {
    pub x: i32, pub y: i32, pub z: i32, pub block: String,
}
impl BlockObservation {
    pub fn new(x:i32,y:i32,z:i32,block:impl Into<String>)->Self { Self{x,y,z,block:block.into()} }
    pub fn chunk(&self)->(i32,i32) { (self.x.div_euclid(16),self.z.div_euclid(16)) }
    pub fn local(&self)->(i32,i32) { (self.x.rem_euclid(16),self.z.rem_euclid(16)) }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PatternKind { BuriedTreasure, DesertPyramid, JungleTemple, SwampHut, Igloo, Village, Shipwreck, Monument, Outpost, EndCity, TrialChambers, AncientCity, Dungeon, DesertWell, EmeraldOre, EndGateway, WarpedFungus, EndPillar }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternHit { pub kind:PatternKind, pub x:i32,pub y:i32,pub z:i32 }
#[derive(Debug, Clone, Default)]
pub struct FinderObservations {
    pub blocks: Vec<BlockObservation>,
    pub biomes: Vec<(i32,i32,i32)>,
    pub structures: Vec<StructureInstance>,
    pub patterns: Vec<PatternHit>,
}
#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub enum FinderCategory { Structures, Decorators, Biomes, Blocks }
#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub enum FinderType { BuriedTreasure,DesertPyramid,EndCity,JungleTemple,Monument,SwampHut,Shipwreck,Outpost,Igloo,Village,TrialChambers,AncientCity,EndPillars,EndGateway,Dungeon,EmeraldOre,DesertWell,WarpedFungus,Biome }
impl FinderType {
 pub fn category(self)->FinderCategory {match self {
  Self::BuriedTreasure|Self::DesertPyramid|Self::EndCity|Self::JungleTemple|Self::Monument|Self::SwampHut|Self::Shipwreck|Self::Outpost|Self::Igloo|Self::Village|Self::TrialChambers|Self::AncientCity=>FinderCategory::Structures,
  Self::EndPillars|Self::EndGateway|Self::Dungeon|Self::EmeraldOre|Self::DesertWell|Self::WarpedFungus=>FinderCategory::Decorators,Self::Biome=>FinderCategory::Biomes}}
}
#[derive(Debug,Clone)]
pub struct BlockFinder { targets:HashSet<String>, min_y:i32,max_y:i32 }
impl BlockFinder {
 pub fn new<I,S>(targets:I,min_y:i32,max_y:i32)->Self where I:IntoIterator<Item=S>,S:Into<String>{Self{targets:targets.into_iter().map(|s|s.into().to_ascii_lowercase()).collect(),min_y,max_y}}
 pub fn find<'a>(&self,blocks:&'a [BlockObservation])->Vec<&'a BlockObservation>{blocks.iter().filter(|b|b.y>=self.min_y&&b.y<=self.max_y&&self.targets.iter().any(|t|b.block.to_ascii_lowercase().ends_with(t))).collect()}
}
#[derive(Debug,Clone)]
pub struct FinderRegistry { enabled:HashSet<FinderType>, generation:u64 }
impl Default for FinderRegistry {
 fn default()->Self{Self{enabled:HashSet::from([FinderType::BuriedTreasure,FinderType::DesertPyramid,FinderType::EndCity,FinderType::JungleTemple,FinderType::Monument,FinderType::SwampHut,FinderType::Shipwreck,FinderType::Outpost,FinderType::Igloo,FinderType::Village,FinderType::TrialChambers,FinderType::AncientCity,FinderType::EndPillars,FinderType::EndGateway,FinderType::Dungeon,FinderType::EmeraldOre,FinderType::DesertWell,FinderType::WarpedFungus,FinderType::Biome]),generation:1}}
}
impl FinderRegistry {
 pub fn enabled(&self,kind:FinderType)->bool{self.enabled.contains(&kind)}
 pub fn set_enabled(&mut self,kind:FinderType,value:bool){if value{self.enabled.insert(kind);}else{self.enabled.remove(&kind);}self.generation=self.generation.wrapping_add(1);}
 pub fn generation(&self)->u64{self.generation}
 pub fn reload(&mut self){self.generation=self.generation.wrapping_add(1);}
 pub fn enabled_types(&self)->impl Iterator<Item=FinderType>+'_ {self.enabled.iter().copied()}
}
#[derive(Debug, Clone, Default)]
pub struct BlockUpdateBatch { pub updates:Vec<BlockObservation>, pub removed:Vec<(i32,i32,i32)> }

/// Snapshot-based detector engine. It is deterministic and independent of Starling's
/// chunk decoder; callers feed canonical block names and biome IDs.
#[derive(Debug, Default, Clone)]
pub struct PatternDetector { known:HashSet<(i32,i32,i32,String)> }
impl PatternDetector {
 pub fn scan(&mut self, blocks:&[BlockObservation])->FinderObservations {
  let mut out=FinderObservations::default(); let map:HashMap<(i32,i32,i32),&str>=blocks.iter().map(|b|((b.x,b.y,b.z),b.block.as_str())).collect();
  let has=|x:i32,y:i32,z:i32,name:&str|map.get(&(x,y,z)).is_some_and(|v|v.eq_ignore_ascii_case(name)||v.rsplit(':').next().is_some_and(|tail|tail.eq_ignore_ascii_case(name)));
  for b in blocks {
   let key=(b.x,b.y,b.z,b.block.clone()); self.known.insert(key);
   let n=b.block.rsplit(':').next().unwrap_or(&b.block).to_ascii_lowercase();
   if n.contains("emerald_ore"){out.patterns.push(PatternHit{kind:PatternKind::EmeraldOre,x:b.x,y:b.y,z:b.z});}
   if n=="spawner" || n.ends_with(":spawner") {
    let mut floor=0; for dx in -4..=4 {for dz in -4..=4 {if (-4..=4).contains(&dx)&&(-4..=4).contains(&dz) {let f=map.get(&(b.x+dx,b.y-1,b.z+dz)).copied().unwrap_or("");if f.contains("cobblestone")||f.contains("mossy_cobblestone"){floor+=1}}}}
    if floor>=20 {out.patterns.push(PatternHit{kind:PatternKind::Dungeon,x:b.x,y:b.y,z:b.z});}
   }
   if n=="chest"||n.ends_with(":chest") {
    let (lx,lz)=b.local();let below=map.get(&(b.x,b.y-1,b.z)).copied().unwrap_or("");
    if lx==9&&lz==9&&["sandstone","stone","andesite","granite","diorite","coal_ore","iron_ore","gold_ore","gravel"].iter().any(|s|below.contains(s)){out.patterns.push(PatternHit{kind:PatternKind::BuriedTreasure,x:b.x,y:b.y,z:b.z});}
   }
   if n.contains("end_gateway") {out.patterns.push(PatternHit{kind:PatternKind::EndGateway,x:b.x,y:b.y,z:b.z});}
   if n.contains("warped_stem") {
    let soil=map.get(&(b.x,b.y-1,b.z)).copied().unwrap_or("");
    if soil.contains("netherrack")||soil.contains("warped_nylium"){out.patterns.push(PatternHit{kind:PatternKind::WarpedFungus,x:b.x,y:b.y,z:b.z});}
   }
   if n=="bedrock" && b.y>=76 && b.y<=106 {out.patterns.push(PatternHit{kind:PatternKind::EndPillar,x:b.x,y:b.y,z:b.z});}
   // Desert-well signature: center water and sandstone rim.
   if n=="water" && has(b.x-1,b.y,b.z,"sandstone")&&has(b.x+1,b.y,b.z,"sandstone")&&has(b.x,b.y,b.z-1,"sandstone")&&has(b.x,b.y,b.z+1,"sandstone") {out.patterns.push(PatternHit{kind:PatternKind::DesertWell,x:b.x,y:b.y,z:b.z});}
  }
  // Per-chunk palette signatures for SeedCrackerX structure families.
  let mut palettes:HashMap<(i32,i32),HashMap<String,usize>>=HashMap::new();
  for b in blocks { let key=b.chunk();let name=b.block.rsplit(':').next().unwrap_or(&b.block).to_ascii_lowercase();*palettes.entry(key).or_default().entry(name).or_default()+=1; }
  for ((cx,cz),p) in palettes {
   let count=|needle:&str|p.iter().filter(|(name,_)|name.contains(needle)).map(|(_,n)|*n).sum::<usize>();
   let signature=[(PatternKind::DesertPyramid,"orange_terracotta",20),(PatternKind::JungleTemple,"mossy_cobblestone",20),(PatternKind::SwampHut,"oak_planks",12),(PatternKind::Igloo,"snow_block",12),(PatternKind::Village,"hay_block",8),(PatternKind::Shipwreck,"oak_planks",15),(PatternKind::Monument,"prismarine",40),(PatternKind::Outpost,"dark_oak_planks",12),(PatternKind::EndCity,"purpur",20),(PatternKind::TrialChambers,"tuff",40),(PatternKind::AncientCity,"sculk",20)];
   for (kind,block,min) in signature {if count(block)>=min {out.patterns.push(PatternHit{kind,x:cx*16+8,y:0,z:cz*16+8});}}
  }
  out.blocks=blocks.to_vec();out
 }
 pub fn apply_updates(&mut self,batch:BlockUpdateBatch) {
  for (x,y,z) in batch.removed {self.known.retain(|(bx,by,bz,_)|(*bx,*by,*bz)!=(x,y,z));}
  for b in batch.updates {self.known.retain(|(x,y,z,_)|(*x,*y,*z)!=(b.x,b.y,b.z));}
 }
 pub fn reset(&mut self){self.known.clear();}
}
