//! VlaKeeUI Pages: Mod-specific menu pages

use crate::menu::{PageContext, PageResult, MenuAction, VlaKeeState, RenderContext, TouchEvent, TouchPhase};
use vkyakee_mods::api::ModMetadata;

/// Trait for menu pages
pub trait Page: Send + Sync {
    fn render(&mut self, ctx: &mut PageContext, render_ctx: &mut RenderContext) -> PageResult;
    fn handle_touch(&mut self, ctx: &mut PageContext, touch: TouchEvent, render_ctx: &RenderContext) -> bool;
    fn on_enter(&mut self, _ctx: &mut PageContext) {}
    fn on_exit(&mut self, _ctx: &mut PageContext) {}
}

/// Page context passed to pages
pub struct PageContext {
    pub state: VlaKeeState,
    pub mod_list: Vec<ModMetadata>,
}

impl PageContext {
    pub fn new(state: VlaKeeState, mod_list: Vec<ModMetadata>) -> Self {
        Self { state, mod_list }
    }
    
    pub fn dispatch(&mut self, action: MenuAction) -> PageResult {
        PageResult::with_action(action)
    }
}

/// Home page - lists all mods
pub mod home;
pub use home::HomePage;

/// SeedCracker page with quick actions
pub mod seedcracker;
pub use seedcracker::SeedCrackerPage;

/// VKsc - SeedCracker quick shortcuts
pub mod vksc;
pub use vksc::VKscPage;

/// VlaKeeMatica page
pub mod vlakeematica;
pub use vlakeematica::VlaKeeMaticaPage;

/// VkMap page
pub mod vkmap;
pub use vkmap::VkMapPage;

/// Settings page
pub mod settings;
pub use settings::SettingsPage;
