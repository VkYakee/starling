//! VlaKeeMenu: State machine for the floating button and expandable menu
//!
//! This follows Starling's native TouchUi pattern:
//! - Draggable floating button with persisted position
//! - Tap to toggle menu open/close
//! - Animated open/close transitions
//! - Multiple pages for mod management

use crate::pages::{Page, PageContext, PageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use vkyakee_mods::api::ModMetadata;

/// Color palette for VlaKeeUI - Green to Red gradient theme
pub mod palette {
    // Core gradient colors
    pub const GREEN: [u8; 4] = [52, 214, 112, 255];      // #34D670
    pub const GREEN_DARK: [u8; 4] = [20, 126, 72, 255];   // #147E48
    pub const RED: [u8; 4] = [244, 66, 82, 255];          // #F44252
    pub const RED_DARK: [u8; 4] = [150, 30, 48, 255];     // #961E30
    
    // UI Colors
    pub const PANEL_BG: [u8; 4] = [12, 16, 18, 218];      // Charcoal ~85%
    pub const PANEL_SOFT: [u8; 4] = [18, 22, 24, 188];    // ~74%
    pub const SURFACE: [u8; 4] = [22, 28, 32, 230];       // Slightly lighter panel
    pub const SURFACE_SELECTED: [u8; 4] = [32, 40, 48, 240];
    pub const BORDER: [u8; 4] = [66, 92, 78, 210];        // Green-tinted border
    pub const TEXT_PRIMARY: [u8; 4] = [242, 246, 244, 255];
    pub const TEXT_MUTED: [u8; 4] = [154, 166, 162, 255];
    pub const ACCENT: [u8; 4] = [52, 214, 112, 255];
    pub const ACCENT_TEXT: [u8; 4] = [255, 255, 255, 255];
    pub const DANGER: [u8; 4] = [244, 66, 82, 255];
    
    // Shadow
    pub const SHADOW: [u8; 4] = [0, 0, 0, 115];
    pub const SHADOW_SOFT: [u8; 4] = [0, 0, 0, 105];
}

/// Interpolates between green and red gradient
pub fn gradient_color(t: f32) -> [u8; 4] {
    let t = t.clamp(0.0, 1.0);
    let r = (palette::GREEN[0] as f32 + (palette::RED[0] as f32 - palette::GREEN[0] as f32) * t) as u8;
    let g = (palette::GREEN[1] as f32 + (palette::RED[1] as f32 - palette::GREEN[1] as f32) * t) as u8;
    let b = (palette::GREEN[2] as f32 + (palette::RED[2] as f32 - palette::GREEN[2] as f32) * t) as u8;
    [r, g, b, 255]
}

/// Menu page identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MenuPage {
    #[default]
    Home,
    SeedCracker,
    VKsc,
    VlaKeeMatica,
    VkMap,
    Settings,
}

impl MenuPage {
    pub fn label(&self) -> &'static str {
        match self {
            MenuPage::Home => "Home",
            MenuPage::SeedCracker => "SeedCracker",
            MenuPage::VKsc => "VKsc",
            MenuPage::VlaKeeMatica => "VlaKeeMatica",
            MenuPage::VkMap => "VkMap",
            MenuPage::Settings => "Settings",
        }
    }
    
    pub fn icon(&self) -> &'static str {
        match self {
            MenuPage::Home => "🏠",
            MenuPage::SeedCracker => "🔍",
            MenuPage::VKsc => "⚡",
            MenuPage::VlaKeeMatica => "📐",
            MenuPage::VkMap => "🗺️",
            MenuPage::Settings => "⚙️",
        }
    }
    
    pub fn gradient_pos(&self) -> f32 {
        match self {
            MenuPage::Home => 0.0,
            MenuPage::SeedCracker => 0.25,
            MenuPage::VKsc => 0.375,
            MenuPage::VlaKeeMatica => 0.5,
            MenuPage::VkMap => 0.75,
            MenuPage::Settings => 1.0,
        }
    }
    
    pub fn all() -> [MenuPage; 6] {
        [MenuPage::Home, MenuPage::SeedCracker, MenuPage::VKsc, MenuPage::VlaKeeMatica, MenuPage::VkMap, MenuPage::Settings]
    }
}

/// Actions that can be dispatched to the menu
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    ToggleMenu,
    OpenPage(MenuPage),
    CloseMenu,
    GoBack,
    ToggleModule(String),  // mod_id
    RunCommand(String),    // command string
    RefreshMods,
}

/// Persistent-friendly menu state
/// Renderer owns pointer/touch handling; this is pure state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VlaKeeState {
    /// Whether the floating button is visible on screen
    pub visible: bool,
    /// Whether the menu panel is currently open/expanded
    pub open: bool,
    /// Current page being displayed
    pub current_page: MenuPage,
    /// Previous page (for back navigation)
    pub previous_page: Option<MenuPage>,
    /// Animation progress 0.0..1.0 for open/close transition
    pub open_anim: f32,
    /// Animation progress for tab/page transitions
    pub page_anim: f32,
    /// Floating button position (normalized 0.0..1.0)
    /// Persisted via Starling's touch settings
    pub button_pos: [f32; 2],
    /// Whether button is being dragged
    pub dragging: bool,
    /// Drag offset from button center
    pub drag_offset: [f32; 2],
    /// Whether the drag has moved enough to count as drag vs tap
    pub drag_moved: bool,
    /// Panel opacity (0-255)
    pub panel_opacity: u8,
    /// UI scale percent (75-150)
    pub scale_percent: u16,
    /// Cached mod metadata for the home page
    pub mod_list: Vec<ModMetadata>,
    /// Per-page scroll offsets
    pub scroll_offsets: HashMap<MenuPage, f32>,
    /// Whether to show the gradient accent bar on the button
    pub show_gradient: bool,
}

impl VlaKeeState {
    pub fn new() -> Self {
        Self {
            visible: true,
            open: false,
            current_page: MenuPage::Home,
            previous_page: None,
            open_anim: 0.0,
            page_anim: 1.0,
            button_pos: [0.44, 0.05],  // Top-right-ish default
            dragging: false,
            drag_offset: [0.0, 0.0],
            drag_moved: false,
            panel_opacity: 230,
            scale_percent: 100,
            mod_list: Vec::new(),
            scroll_offsets: HashMap::new(),
            show_gradient: true,
        }
    }
    
    pub fn update_animations(&mut self, dt: f32) {
        let speed = 8.0;
        
        // Open/close animation
        let target_open = if self.open { 1.0 } else { 0.0 };
        if (self.open_anim - target_open).abs() > 0.001 {
            self.open_anim += (target_open - self.open_anim) * speed * dt;
            self.open_anim = self.open_anim.clamp(0.0, 1.0);
        }
        
        // Page transition animation
        let target_page = if self.open { 1.0 } else { 0.0 };
        if (self.page_anim - target_page).abs() > 0.001 {
            self.page_anim += (target_page - self.page_anim) * speed * dt;
            self.page_anim = self.page_anim.clamp(0.0, 1.0);
        }
    }
    
    pub fn toggle_menu(&mut self) {
        self.open = !self.open;
        if self.open {
            self.previous_page = Some(self.current_page);
        }
    }
    
    pub fn open_page(&mut self, page: MenuPage) {
        if page != self.current_page {
            self.previous_page = Some(self.current_page);
            self.current_page = page;
            self.page_anim = 0.0;
        }
    }
    
    pub fn close_menu(&mut self) {
        self.open = false;
    }
    
    pub fn go_back(&mut self) {
        if let Some(prev) = self.previous_page {
            self.current_page = prev;
            self.previous_page = None;
            self.page_anim = 0.0;
        } else if self.current_page != MenuPage::Home {
            self.current_page = MenuPage::Home;
            self.page_anim = 0.0;
        } else {
            self.close_menu();
        }
    }
    
    pub fn set_opacity(&mut self, opacity: u8) {
        self.panel_opacity = opacity;
    }
    
    pub fn set_scale_percent(&mut self, scale: u16) {
        self.scale_percent = scale.clamp(75, 150);
    }
    
    pub fn start_drag(&mut self, touch_pos: [f32; 2], screen_size: [f32; 2]) {
        let (w, h) = (screen_size[0], screen_size[1]);
        let btn_x = (self.button_pos[0] * w).clamp(6.0, (w - 48.0 - 6.0).max(6.0));
        let btn_y = (self.button_pos[1] * h).clamp(4.0, (h - 42.0 - 4.0).max(4.0));
        
        self.dragging = true;
        self.drag_offset = [
            touch_pos[0] - btn_x - 24.0,
            touch_pos[1] - btn_y - 21.0,
        ];
        self.drag_moved = false;
    }
    
    pub fn update_drag(&mut self, touch_pos: [f32; 2], screen_size: [f32; 2]) {
        if !self.dragging {
            return;
        }
        
        let (w, h) = (screen_size[0], screen_size[1]);
        let new_x = touch_pos[0] - self.drag_offset[0] - 24.0;
        let new_y = touch_pos[1] - self.drag_offset[1] - 21.0;
        
        let moved_x = new_x - (self.button_pos[0] * w).clamp(6.0, (w - 48.0 - 6.0).max(6.0));
        let moved_y = new_y - (self.button_pos[1] * h).clamp(4.0, (h - 42.0 - 4.0).max(4.0));
        if moved_x.hypot(moved_y) > 5.0 {
            self.drag_moved = true;
        }
        
        self.button_pos = [
            (new_x / w.max(1.0)).clamp(0.03, 0.93),
            (new_y / h.max(1.0)).clamp(0.02, 0.90),
        ];
    }
    
    /// Ends dragging, returns whether it was a tap (not a drag)
    pub fn end_drag(&mut self) -> bool {
        let was_tap = self.dragging && !self.drag_moved;
        self.dragging = false;
        self.drag_moved = false;
        was_tap
    }
    
    /// Dispatches an action to the menu
    pub fn dispatch(&mut self, action: MenuAction) {
        match action {
            MenuAction::ToggleMenu => self.toggle_menu(),
            MenuAction::OpenPage(page) => self.open_page(page),
            MenuAction::CloseMenu => self.close_menu(),
            MenuAction::GoBack => self.go_back(),
            MenuAction::ToggleModule(mod_id) => {
                // Handled by the specific page
                log::debug!("Toggle module: {}", mod_id);
            }
            MenuAction::RunCommand(cmd) => {
                log::debug!("Run command: {}", cmd);
            }
            MenuAction::RefreshMods => {
                // Will be populated from VkModHub
            }
        }
    }
    
    /// Sets the mod list (called from VkModHub)
    pub fn set_mod_list(&mut self, mods: Vec<ModMetadata>) {
        self.mod_list = mods;
    }
    
    /// Gets the scroll offset for current page
    pub fn get_scroll(&self) -> f32 {
        self.scroll_offsets.get(&self.current_page).copied().unwrap_or(0.0)
    }
    
    /// Sets the scroll offset for current page
    pub fn set_scroll(&mut self, offset: f32) {
        self.scroll_offsets.insert(self.current_page, offset);
    }
}

/// Main VlaKeeMenu struct that holds state and coordinates pages
pub struct VlaKeeMenu {
    pub state: VlaKeeState,
    pages: HashMap<MenuPage, Box<dyn Page>>,
}

impl VlaKeeMenu {
    pub fn new() -> Self {
        let mut menu = Self {
            state: VlaKeeState::new(),
            pages: HashMap::new(),
        };
        
        // Register all pages
        menu.pages.insert(MenuPage::Home, Box::new(crate::pages::HomePage::new()));
        menu.pages.insert(MenuPage::SeedCracker, Box::new(crate::pages::SeedCrackerPage::new()));
        menu.pages.insert(MenuPage::VKsc, Box::new(crate::pages::VKscPage::new()));
        menu.pages.insert(MenuPage::VlaKeeMatica, Box::new(crate::pages::VlaKeeMaticaPage::new()));
        menu.pages.insert(MenuPage::VkMap, Box::new(crate::pages::VkMapPage::new()));
        menu.pages.insert(MenuPage::Settings, Box::new(crate::pages::SettingsPage::new()));
        
        menu
    }
    
    /// Updates the menu (animations, etc.)
    pub fn update(&mut self, dt: f32) {
        self.state.update_animations(dt);
    }
    
    /// Dispatches an action
    pub fn dispatch(&mut self, action: MenuAction) {
        self.state.dispatch(action);
    }
    
    /// Gets the current page
    pub fn current_page(&self) -> &dyn Page {
        self.pages.get(&self.state.current_page)
            .map(|p| p.as_ref())
            .unwrap_or_else(|| self.pages.get(&MenuPage::Home).unwrap().as_ref())
    }
    
    /// Gets a mutable reference to the current page
    pub fn current_page_mut(&mut self) -> &mut dyn Page {
        self.pages.get_mut(&self.state.current_page)
            .map(|p| p.as_mut())
            .unwrap_or_else(|| self.pages.get_mut(&MenuPage::Home).unwrap().as_mut())
    }
    
    /// Renders the current page
    pub fn render_page(&mut self, ctx: &mut PageContext, render_ctx: &mut RenderContext) -> PageResult {
        let page = self.current_page_mut();
        page.render(ctx, render_ctx)
    }
    
    /// Handles touch input for the current page
    pub fn handle_touch(&mut self, ctx: &mut PageContext, touch: TouchEvent) -> bool {
        let page = self.current_page_mut();
        page.handle_touch(ctx, touch)
    }
}

/// Rendering context passed to pages
pub struct RenderContext {
    pub batch: *mut crate::render::HudBatch,
    pub atlas: *mut crate::render::HudAtlas,
    pub screen_size: [f32; 2],
    pub time: f32,
}

/// Touch event for page handling
#[derive(Debug, Clone, Copy)]
pub struct TouchEvent {
    pub id: u64,
    pub phase: TouchPhase,
    pub pos: [f32; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}

/// Result from page rendering/handling
pub struct PageResult {
    pub actions: Vec<MenuAction>,
    pub needs_redraw: bool,
}

impl Default for PageResult {
    fn default() -> Self {
        Self {
            actions: Vec::new(),
            needs_redraw: true,
        }
    }
}

impl PageResult {
    pub fn with_action(action: MenuAction) -> Self {
        Self {
            actions: vec![action],
            needs_redraw: true,
        }
    }
    
    pub fn with_actions(actions: Vec<MenuAction>) -> Self {
        Self {
            actions,
            needs_redraw: true,
        }
    }
}
