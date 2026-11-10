//! Structure seed validation against Horizon's authoritative placement oracle.
use horizon_world::worldgen::structure::start::{StructureRegistry,Rings};
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool,Ordering};

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub struct ObservedStructure{pub set_name:String,pub chunk_x:i32,pub chunk_z:i32}
#[derive(Debug,Clone,Default)]
pub struct StructureCrackerResult{pub candidates:Vec<u64>,pub cancelled:bool,pub exhaustive:bool}
impl StructureCrackerResult{pub fn new()->Self{Self::default()}}

pub struct StructureCracker<'a>{registry:&'a StructureRegistry,observed:Vec<ObservedStructure>,rings:&'a Rings}
impl<'a> StructureCracker<'a>{
 pub fn new(registry:&'a StructureRegistry,observed:Vec<ObservedStructure>,rings:&'a Rings)->Self{Self{registry,observed,rings}}
 pub fn with_rings(registry:&'a StructureRegistry,observed:Vec<ObservedStructure>,rings:&'a Rings)->Self{Self::new(registry,observed,rings)}
 /// Each observation must correspond to a known set and its exact generated start.
 pub fn test_seed(&self,seed:u64)->bool{
  if self.observed.is_empty(){return false}
  self.observed.iter().all(|o|{
   let Some(set)=self.registry.sets.iter().find(|s|s.name==o.set_name) else{return false};
   self.registry.is_structure_chunk(set,seed as i64,o.chunk_x,o.chunk_z,self.rings)
  })
 }
 /// Scans explicit upper-bit slices, retaining the complete 48-bit domain semantics.
 /// No guessed modulo-4 shortcut: every candidate is checked by Horizon.
 pub fn search_range(&self,upper_start:u64,upper_end:u64,cancel:&AtomicBool)->Vec<u64>{
  if self.observed.is_empty()||upper_start>=upper_end||upper_start>=(1<<29){return Vec::new()}
  (upper_start..upper_end.min(1<<29)).into_par_iter().flat_map_iter(|upper|{
   let mut found=Vec::new();
   for lower in 0..(1u64<<19){
    if cancel.load(Ordering::Relaxed){break}
    let seed=(upper<<19)|lower;
    if self.test_seed(seed){found.push(seed)}
   } found
  }).collect()
 }
 pub fn search_range_exhaustive(&self,upper_start:u64,upper_end:u64,cancel:&AtomicBool)->Vec<u64>{self.search_range(upper_start,upper_end,cancel)}
 pub fn search_exhaustive(&self,cancel:&AtomicBool)->Vec<u64>{
  self.search_range(0,1<<29,cancel)
 }
}
pub struct StructureObserver{observations:Vec<ObservedStructure>}
impl StructureObserver{
 pub fn new()->Self{Self{observations:Vec::new()}}
 pub fn add(&mut self,set_name:String,chunk_x:i32,chunk_z:i32){self.observations.push(ObservedStructure{set_name,chunk_x,chunk_z})}
 pub fn add_block_pos(&mut self,set_name:String,block_x:i32,block_z:i32){self.add(set_name,block_x>>4,block_z>>4)}
 pub fn build(self)->Vec<ObservedStructure>{self.observations}
}
