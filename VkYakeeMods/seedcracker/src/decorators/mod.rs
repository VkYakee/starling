use crate::math::chunk_rand;
use crate::math::lcg::{JavaRandom, MASK48, MULTIPLIER};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecoratorType { Dungeon, DesertWell, EmeraldOre, WarpedFungus }

#[derive(Debug, Clone)]
pub struct EmeraldOreData { pub chunk_x:i32,pub chunk_z:i32,pub block_x:i32,pub block_y:i32,pub block_z:i32,pub step:u64,pub index:u64 }
impl EmeraldOreData {
 pub fn new(chunk_x:i32,chunk_z:i32,block_x:i32,block_y:i32,block_z:i32)->Self{Self{chunk_x,chunk_z,block_x,block_y,block_z,step:6,index:14}}
 pub fn salt(&self)->u64{self.index+10000*self.step}
 pub fn test_seed(&self,structure_seed:u64)->bool{
  let pop=chunk_rand::get_population_seed(structure_seed,self.chunk_x,self.chunk_z);
  let mut r=JavaRandom::from_state(((pop.wrapping_add(self.salt()))^MULTIPLIER)&MASK48);
  let count=r.next_int_bounded(6).unwrap_or(0)+3; let ox=self.block_x&15;let oz=self.block_z&15;
  for _ in 0..count {let x=r.next_int_bounded(16).unwrap_or(-1);let z=r.next_int_bounded(16).unwrap_or(-1);let y=r.next_int_bounded(28).unwrap_or(-1)+4;if x==ox&&z==oz&&y==self.block_y{return true}}
  false
 }
}
#[derive(Debug,Clone)]
pub struct DesertWellData{pub chunk_x:i32,pub chunk_z:i32,pub block_x:i32,pub block_z:i32,pub salt:u64}
impl DesertWellData{
 pub fn new(chunk_x:i32,chunk_z:i32,block_x:i32,block_z:i32)->Self{Self{chunk_x,chunk_z,block_x,block_z,salt:80003}}
 pub fn test_seed(&self,structure_seed:u64)->bool{
  let pop=chunk_rand::get_population_seed(structure_seed,self.chunk_x,self.chunk_z);
  let mut r=JavaRandom::from_state(((pop.wrapping_add(self.salt))^MULTIPLIER)&MASK48);
  if r.next_float()>=0.001{return false}
  r.next_int_bounded(16)==Some(self.block_x&15)&&r.next_int_bounded(16)==Some(self.block_z&15)
 }
}
#[cfg(test)]]