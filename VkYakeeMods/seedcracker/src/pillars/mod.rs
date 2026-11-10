//! End Pillars - Pillar seed cracking for TimeMachine
//!
//! Minecraft's End pillars have heights determined by a 16-bit pillar seed.
//! The pillar seed is derived from the structure seed, and observing
//! pillar heights allows us to recover the pillar seed, which then
//! allows TimeMachine lifting from 48-bit to 64-bit seeds.

pub mod end_pillars;
pub use end_pillars::{EndPillars, PillarObservations, PILLAR_COUNT};
