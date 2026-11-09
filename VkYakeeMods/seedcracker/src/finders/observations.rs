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
#[derive(Debug, Clone, Default)]
pub struct BlockUpdateBatch { pub updates:Vec<BlockObservation>, pub removed:Vec<(i32,i32,i32)> }

/// Snapshot-based detector engine. It is deterministic and independent of Starling's
/// chunk decoder; callers feed canonical block names and biome IDs.
#[derive(Debug, Default, Clone)]
pub struct PatternDetector { known:HashSet<(i32,i32,i32,String)> }
impl PatternDetector {
 pub fn scan(&mut self, blocks:&[BlockObservation])->FinderObservations {
  let mut out=FinderObservations::default(); let map:HashMap<(i32,i32,i32),&str>=blocks.iter().map(|b|((b.x,b.y,b.z),b.block.as_str())).collect();
  let has=|x:i32,y:i32,z:i32,name:&str|map.get(&(x,y,z)).is_some_and(|v|v.eq_ignore_ascii_case(name));
  for b in blocks {
   let key=(b.x,b.y,b.z,b.block.clone()); if !self.known.insert(key){continue}
   let n=b.block.to_ascii_lowercase();
   if n.contains("emerald_ore"){out.patterns.push(PatternHit{kind:PatternKind::EmeraldOre,x:b.x,y:b.y,z:b.z});out.patterns.push(PatternHit{kind:PatternKind::EmeraldOre,x:b.x,y:b.y,z:b.z});}
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
  out.blocks=blocks.to_vec();out
 }
 pub fn apply_updates(&mut self,batch:BlockUpdateBatch) {
  for (x,y,z) in batch.removed {self.known.retain(|(bx,by,bz,_)|(*bx,*by,*bz)!=(x,y,z));}
  for b in batch.updates {self.known.retain(|(x,y,z,_)|(*x,*y,*z)!=(b.x,b.y,b.z));self.known.insert((b.x,b.y,b.z,b.block));}
 }
 pub fn reset(&mut self){self.known.clear();}
}
