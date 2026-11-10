//! VlaKeeMatica - Litematica 1:1 Native Rust Rewrite for Starling / Horizon
//! 
//! This is a complete rewrite of Litematica (by Masa) in Rust, designed to run natively
//! in Starling (Horizon Android fork) with full feature parity including:
//! - Schematic loading (.litematic, .schem, .schematic, .nbt)
//! - 3D hologram placement and rendering
//! - Material list & inventory tracking
//! - EasyPlace protocol support (Auto, V2, V3, SlabOnly)
//! - Schematic verifier (Missing, Extra, WrongBlock, WrongState)
//! - Area selection (Pos1, Pos2) and schematic creation
//! - Project versioning with snapshots
//! - Layer slicing (Y, X, Z axes)
//! - Rotation (90° CW/CCW) and Mirroring (X/Z)
//! - Mini-renderer (Isometric 3D + 2D Layer views)
//!
//! Architecture follows Horizon's native mod system: mixin-based hooks + VkMod trait.

// Re-export all public modules
pub mod util;
pub mod schematic;
pub mod selection;
pub mod materials;
pub mod tool;
pub mod config;
pub mod assets;
pub mod network;

// Core prelude for convenient imports
pub mod prelude {
    pub use crate::util::pos::{BlockPos, Vec3i, Rotation, Mirror, Axis};
    pub use crate::util::layer::{LayerMode, LayerRange};
    pub use crate::util::easy_place::EasyPlaceProtocol;
    pub use crate::schematic::litematic::{LitematicaSchematic, SubRegion};
    pub use crate::schematic::palette::{BlockState, BlockStatePalette};
    pub use crate::schematic::placement::{SchematicPlacement, SubRegionPlacement, SchematicPlacementManager};
    pub use crate::schematic::verifier::{SchematicVerifier, BlockMismatch, VerificationSummary, MismatchType};
    pub use crate::schematic::projects::{SchematicProject, SchematicProjectsManager, ProjectVersion};
    pub use crate::schematic::mini_render::{SchematicMiniRenderer, PreviewMode};
    pub use crate::materials::{MaterialList, MaterialEntry, MaterialListExporter};
    pub use crate::materials::inventory::{InventoryTracker, InventoryProgress, InventoryProgressEntry};
    pub use crate::selection::{SelectionManager, BoxRegion, AreaSelection};
    pub use crate::tool::{ToolMode};
    pub use crate::config::{VlaKeeMaticaConfig, Hotkeys, Visuals, Colors, InfoOverlays};
    pub use crate::assets::AssetManager;
}

pub use util::*;
pub use schematic::*;
pub use selection::*;
pub use materials::*;
pub use tool::*;
pub use config::*;
pub use assets::*;
pub use network::*;

// Horizon Mod SDK integration - re-export from horizon-mods which publicly re-exports runtime items
#[cfg(feature = "mods")]
pub use horizon_mods::{
    Clock, Player,
    Corner, Line,
    UiScreen, UiWidget,
    FillShape, QuadShape, RenderRect, Shape, SpriteShape, TextShape,
    InputEvent, HeldItem, ItemUse,
};

#[cfg(feature = "mods")]
mod native_hooks;

#[cfg(feature = "mods")]
mod vk_mod_impl;
