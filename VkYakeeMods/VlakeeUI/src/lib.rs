//! VlaKeeUI: Floating button + expandable menu for VkYakeeMods admin
//! 
//! Features:
//! - Draggable floating button (persists position via Starling's touch settings)
//! - Expandable menu with pages for each mod (SeedCracker, HorizonMatica, VkMap, Settings)
//! - Green-to-red gradient theme
//! - Follows Starling's native TouchUi.floating_menu pattern

pub mod menu;
pub mod render;
pub mod adapter;
pub mod pages;

pub use menu::{VlaKeeMenu, MenuPage, MenuAction, VlaKeeState};
pub use render::draw_vlakee_menu;
pub use adapter::VlaKeeUiMod;
pub use pages::*;
