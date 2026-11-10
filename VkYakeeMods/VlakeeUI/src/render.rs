//! VlaKeeUI Rendering: Drawing the floating button and menu panel
//!
//! This integrates with Starling's HudBatch rendering system,
//! following the same patterns as TouchUi::draw_floating_menu

use crate::menu::{VlaKeeState, MenuPage, palette, gradient_color};
use crate::pages::{HomePage, SeedCrackerPage, VKscPage, HorizonMaticaPage, VkMapPage, SettingsPage, Page};
use std::sync::Mutex;

/// HudBatch and HudAtlas types (forward declared from Starling)
pub mod starling {
    use std::ffi::c_void;
    
    pub struct HudBatch {
        pub width: f32,
        pub height: f32,
        pub scale: f32,
        pub density: f32,
        _private: [u8; 0],
    }
    
    pub struct HudAtlas {
        _private: [u8; 0],
    }
    
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
        ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32)
    }
    
    impl HudBatch {
        pub fn fill(&mut self, _atlas: &HudAtlas, x: f32, y: f32, w: f32, h: f32, color: u32) {}
        pub fn text(&mut self, _atlas: &HudAtlas, text: &str, x: f32, y: f32, color: u32, bold: bool) {}
        pub fn rounded(&mut self, _atlas: &HudAtlas, x: f32, y: f32, w: f32, h: f32, radius: f32, color: u32) {}
        pub fn stroke_rounded(&mut self, _atlas: &HudAtlas, x: f32, y: f32, w: f32, h: f32, radius: f32, stroke: f32, color: u32) {}
        pub fn icon(&mut self, _atlas: &HudAtlas, _icon: u32, x: f32, y: f32, size: f32, color: u32) {}
        pub fn border(&mut self, _atlas: &HudAtlas, x: f32, y: f32, w: f32, h: f32, radius: f32, color: u32) {}
        pub fn centered_fit(&mut self, _face: u32, _size: f32, rect: [f32; 4], color: u32, text: &str) {}
    }
    
    pub mod ui_theme {
        pub struct Ui {
            pub w: f32,
            pub h: f32,
            pub batch: HudBatch,
            pub density: f32,
        }
        
        impl Ui {
            pub fn mobile(_atlas: &HudAtlas, _batch: &mut HudBatch, _pos: (f32, f32)) -> Self {
                Self { w: 0.0, h: 0.0, batch: HudBatch { width: 0.0, height: 0.0, scale: 1.0, density: 1.0, _private: [] }, density: 1.0 }
            }
        }
        
        pub enum Face { Regular, SemiBold, Bold }
        pub enum Icon { MenuEquals, HorizonLogo, Settings, Home, Search, Map, Cog }
        
        pub fn active_theme() -> Theme {
            Theme {
                bg_solid: 0x0C1012DA,
                sidebar_bg: 0x161C20E6,
                surface: 0x161C20E6,
                surface_selected: 0x202830F0,
                border: 0x425C4ED2,
                accent: 0x34D670FF,
                accent_text: 0xFFFFFFFF,
                text_primary: 0xF2F6F4FF,
                text_muted: 0x9AA6A2FF,
                danger: 0xF44252FF,
            }
        }
        
        pub struct Theme {
            pub bg_solid: u32,
            pub sidebar_bg: u32,
            pub surface: u32,
            pub surface_selected: u32,
            pub border: u32,
            pub accent: u32,
            pub accent_text: u32,
            pub text_primary: u32,
            pub text_muted: u32,
            pub danger: u32,
        }
    }
}

use starling::{HudBatch, HudAtlas, rgba, ui_theme};

/// Global VlaKeeMenu state for rendering from Starling's mod_render
static VLAKEE_STATE: Mutex<Option<VlaKeeState>> = Mutex::new(None);

/// Gets or creates the global VlaKeeState
pub fn vlakee_state() -> &'static Mutex<Option<VlaKeeState>> {
    &VLAKEE_STATE
}

/// Initializes the VlaKeeUI state
pub fn init_vlakee_state() {
    if let Ok(mut guard) = VLAKEE_STATE.lock() {
        if guard.is_none() {
            *guard = Some(VlaKeeState::new());
        }
    }
}

/// Draws the VlaKeeUI floating button and menu
/// This is called from Starling's mod_render.rs draw_mod_shapes
pub fn draw_vlakee_menu(batch: &mut HudBatch, atlas: &HudAtlas) {
    let state = {
        let mut guard = VLAKEE_STATE.lock().unwrap_or_else(|e| e.into_inner());
        if guard.is_none() {
            *guard = Some(VlaKeeState::new());
        }
        guard.as_mut().unwrap().clone()
    };
    
    if !state.visible {
        return;
    }
    
    let (w, h) = (batch.width, batch.height);
    
    // Button dimensions
    let btn_w = 48.0;
    let btn_h = 42.0;
    let bx = (state.button_pos[0] * w).clamp(6.0, (w - btn_w - 6.0).max(6.0));
    let by = (state.button_pos[1] * h).clamp(4.0, (h - btn_h - 4.0).max(4.0));
    
    // Animation easing
    let open_t = state.open_anim;
    let open_ease = 1.0 - (1.0 - open_t).powi(3);  // Cubic ease out
    
    // Draw floating button
    draw_floating_button(batch, atlas, bx, by, btn_w, btn_h, &state, open_ease);
    
    // Draw expanded menu if open
    if state.open || open_t > 0.001 {
        draw_menu_panel(batch, atlas, w, h, &state, open_ease);
    }
    
    // Update state back (for animation persistence)
    if let Ok(mut guard) = VLAKEE_STATE.lock() {
        if let Some(s) = guard.as_mut() {
            s.open_anim = state.open_anim;
        }
    }
}

/// Draws the floating button with gradient accent
fn draw_floating_button(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    bx: f32,
    by: f32,
    bw: f32,
    bh: f32,
    state: &VlaKeeState,
    open_ease: f32,
) {
    // Shadow
    batch.fill(atlas, bx - 2.0, by - 2.0, bw, bh, rgba(0, 0, 0, 105));
    
    // Background panel
    let bg_alpha = (205.0 * (1.0 - open_ease * 0.3) + 170.0 * open_ease * 0.3) as u8;
    batch.rounded(atlas, bx, by, bw, bh, 14.0, rgba(12, 16, 18, bg_alpha));
    
    // Gradient accent bar at top (green to red)
    if state.show_gradient {
        for i in 0..12 {
            let t = i as f32 / 11.0;
            let color = gradient_color(t);
            let slice_w = bw / 12.0;
            batch.fill(atlas, bx + i as f32 * slice_w, by, slice_w + 0.5, 3.0, rgba(color[0], color[1], color[2], color[3]));
        }
    } else {
        // Single accent color based on current page
        let accent_t = state.current_page.gradient_pos();
        let color = gradient_color(accent_t);
        batch.fill(atlas, bx, by, bw, 3.0, rgba(color[0], color[1], color[2], color[3]));
    }
    
    // Border
    let border_color = if state.open { palette::ACCENT } else { palette::BORDER };
    batch.stroke_rounded(atlas, bx, by, bw, bh, 14.0, 1.6, rgba(
        (border_color[0] as u32) << 24 | (border_color[1] as u32) << 16 | (border_color[2] as u32) << 8 | (border_color[3] as u32)
    ));
    
    // VK or Menu icon
    let icon_color = if state.open { 
        palette::ACCENT_TEXT 
    } else { 
        palette::ACCENT 
    };
    
    // Draw "VK" text or menu icon
    if state.open {
        batch.text(atlas, "✕", bx + (bw - 16.0) * 0.5, by + (bh - 18.0) * 0.5, 
            rgba(icon_color[0], icon_color[1], icon_color[2], icon_color[3]), true);
    } else {
        batch.text(atlas, "VK", bx + (bw - 20.0) * 0.5, by + (bh - 18.0) * 0.5, 
            rgba(icon_color[0], icon_color[1], icon_color[2], icon_color[3]), true);
    }
    
    // Page indicator dots when menu is closed
    if !state.open && state.current_page != MenuPage::Home {
        draw_page_indicator(batch, atlas, bx, by, bw, bh, &state);
    }
}

/// Draws small page indicator dots on the button
fn draw_page_indicator(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    bx: f32,
    by: f32,
    bw: f32,
    bh: f32,
    state: &VlaKeeState,
) {
    let pages = MenuPage::all();
    let current_idx = pages.iter().position(|p| *p == state.current_page).unwrap_or(0);
    
    let dot_count = pages.len();
    let dot_size = 4.0;
    let spacing = 6.0;
    let total_w = dot_count as f32 * dot_size + (dot_count - 1) as f32 * spacing;
    let start_x = bx + (bw - total_w) * 0.5;
    let dot_y = by + bh + 4.0;
    
    for (i, page) in pages.iter().enumerate() {
        let x = start_x + i as f32 * (dot_size + spacing);
        let color = if i == current_idx {
            gradient_color(page.gradient_pos())
        } else {
            palette::TEXT_MUTED
        };
        batch.fill(atlas, x, dot_y, dot_size, dot_size, rgba(color[0], color[1], color[2], 180));
    }
}

/// Draws the expanded menu panel
fn draw_menu_panel(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    w: f32,
    h: f32,
    state: &VlaKeeState,
    open_ease: f32,
) {
    if open_ease < 0.01 {
        return;
    }
    
    // Scrim backdrop
    let scrim_alpha = (185.0 * open_ease.clamp(0.35, 1.0)).round() as u8;
    batch.fill(atlas, 0.0, 0.0, w, h, rgba(6, 5, 14, scrim_alpha));
    
    // Panel metrics
    let panel_w = 320.0;
    let panel_h = 420.0;
    let side_w = 80.0;
    
    // Position panel to the left of the button
    let btn_x = (state.button_pos[0] * w).clamp(6.0, (w - 48.0 - 6.0).max(6.0));
    let btn_y = (state.button_pos[1] * h).clamp(4.0, (h - 42.0 - 4.0).max(4.0));
    
    let px = (btn_x - panel_w - 16.0).max(8.0);
    let py = (btn_y - panel_h * 0.5).clamp(8.0, (h - panel_h - 8.0).max(8.0));
    
    // Slide animation
    let slide_x = (1.0 - open_ease) * 20.0;
    let final_px = px + slide_x;
    
    // Shadow
    batch.rounded(atlas, final_px + 3.0, py + 6.0, panel_w, panel_h, 20.0, rgba(0, 0, 0, 170));
    
    // Panel background
    batch.rounded(atlas, final_px, py, panel_w, panel_h, 20.0, rgba(
        palette::PANEL_BG[0], palette::PANEL_BG[1], palette::PANEL_BG[2], 
        (palette::PANEL_BG[3] as f32 * open_ease) as u8
    ));
    
    // Sidebar background
    batch.rounded(atlas, final_px + 2.0, py + 56.0, side_w, panel_h - 58.0, 16.0, rgba(
        22, 28, 32, (230.0 * open_ease) as u8
    ));
    
    // Dividers
    let border_c = palette::BORDER;
    batch.fill(atlas, final_px + side_w, py + 56.0, 1.0, panel_h - 56.0, rgba(border_c[0], border_c[1], border_c[2], border_c[3]));
    batch.fill(atlas, final_px, py + 55.0, panel_w, 1.0, rgba(border_c[0], border_c[1], border_c[2], border_c[3]));
    batch.stroke_rounded(atlas, final_px, py, panel_w, panel_h, 20.0, 1.4, rgba(border_c[0], border_c[1], border_c[2], border_c[3]));
    
    // Header
    draw_menu_header(batch, atlas, final_px, py, panel_w, side_w, state, open_ease);
    
    // Sidebar (page navigation)
    draw_sidebar(batch, atlas, final_px, py, panel_w, side_w, panel_h, state, open_ease);
    
    // Content area
    let content_x = final_px + side_w + 8.0;
    let content_y = py + 56.0 + 8.0;
    let content_w = panel_w - side_w - 16.0;
    let content_h = panel_h - 56.0 - 16.0;
    
    // Clip/content background
    batch.rounded(atlas, content_x - 4.0, content_y - 4.0, content_w + 8.0, content_h + 8.0, 12.0, rgba(
        palette::SURFACE[0], palette::SURFACE[1], palette::SURFACE[2], 
        (palette::SURFACE[3] as f32 * open_ease) as u8
    ));
    
    // Render current page content
    draw_page_content(batch, atlas, content_x, content_y, content_w, content_h, state, open_ease);
}

/// Draws the menu header
fn draw_menu_header(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    px: f32,
    py: f32,
    panel_w: f32,
    side_w: f32,
    state: &VlaKeeState,
    open_ease: f32,
) {
    let alpha = (255.0 * open_ease) as u8;
    
    // Logo/icon
    batch.rounded(atlas, px + 14.0, py + 9.0, 30.0, 30.0, 9.0, rgba(
        32, 40, 48, alpha
    ));
    batch.border(atlas, px + 14.0, py + 9.0, 30.0, 30.0, 9.0, rgba(
        palette::ACCENT[0], palette::ACCENT[1], palette::ACCENT[2], alpha
    ));
    batch.text(atlas, "VK", px + 21.0, py + 15.0, rgba(
        palette::ACCENT_TEXT[0], palette::ACCENT_TEXT[1], palette::ACCENT_TEXT[2], alpha
    ), true);
    
    // Title
    batch.text(atlas, "VlaKeeUI", px + 52.0, py + 8.0, rgba(
        palette::TEXT_PRIMARY[0], palette::TEXT_PRIMARY[1], palette::TEXT_PRIMARY[2], alpha
    ), true);
    batch.text(atlas, "Mod Manager", px + 52.0, py + 22.0, rgba(
        palette::TEXT_MUTED[0], palette::TEXT_MUTED[1], palette::TEXT_MUTED[2], alpha
    ), false);
    
    // Current page badge
    let page = state.current_page;
    let badge_text = page.label();
    let badge_color = gradient_color(page.gradient_pos());
    let badge_w = 120.0;
    let badge_x = px + panel_w - badge_w - 16.0;
    batch.rounded(atlas, badge_x, py + 11.0, badge_w, 26.0, 8.0, rgba(
        32, 40, 48, alpha
    ));
    batch.border(atlas, badge_x, py + 11.0, badge_w, 26.0, 8.0, rgba(
        badge_color[0], badge_color[1], badge_color[2], alpha
    ));
    batch.text(atlas, badge_text, badge_x + badge_w * 0.5, py + 18.0, rgba(
        badge_color[0], badge_color[1], badge_color[2], alpha
    ), true);
    
    // Close button
    let close_r = [px + panel_w - 76.0, py + 10.0, 62.0, 28.0];
    batch.rounded(atlas, close_r[0], close_r[1], close_r[2], close_r[3], 9.0, rgba(
        palette::SURFACE[0], palette::SURFACE[1], palette::SURFACE[2], alpha
    ));
    batch.text(atlas, "Close", close_r[0] + 8.0, close_r[1] + 8.0, rgba(
        palette::TEXT_PRIMARY[0], palette::TEXT_PRIMARY[1], palette::TEXT_PRIMARY[2], alpha
    ), true);
}

/// Draws the sidebar with page navigation
fn draw_sidebar(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    px: f32,
    py: f32,
    panel_w: f32,
    side_w: f32,
    panel_h: f32,
    state: &VlaKeeState,
    open_ease: f32,
) {
    let pages = MenuPage::all();
    let item_h = 48.0;
    let start_y = py + 56.0 + 8.0;
    let alpha = (255.0 * open_ease) as u8;
    
    for (i, page) in pages.iter().enumerate() {
        let y = start_y + i as f32 * (item_h + 4.0);
        let is_active = *page == state.current_page;
        let is_prev = state.previous_page == Some(*page);
        
        let item_rect = [px + 4.0, y, side_w - 8.0, item_h];
        
        // Background
        let bg_color = if is_active {
            palette::SURFACE_SELECTED
        } else if is_prev {
            [palette::SURFACE[0], palette::SURFACE[1], palette::SURFACE[2], 200]
        } else {
            palette::SURFACE
        };
        batch.rounded(atlas, item_rect[0], item_rect[1], item_rect[2], item_rect[3], 12.0, rgba(
            bg_color[0], bg_color[1], bg_color[2], 
            (bg_color[3] as f32 * open_ease) as u8
        ));
        
        // Accent bar on left
        let accent_color = gradient_color(page.gradient_pos());
        let bar_w = if is_active { 4.0 } else { 2.0 };
        batch.fill(atlas, item_rect[0], item_rect[1], bar_w, item_rect[3], rgba(
            accent_color[0], accent_color[1], accent_color[2], 
            if is_active { 255 } else { 180 }
        ));
        
        // Icon
        batch.text(atlas, page.icon(), item_rect[0] + 16.0, y + 10.0, rgba(
            palette::TEXT_PRIMARY[0], palette::TEXT_PRIMARY[1], palette::TEXT_PRIMARY[2], alpha
        ), true);
        
        // Label
        let label_color = if is_active {
            accent_color
        } else {
            palette::TEXT_MUTED
        };
        batch.text(atlas, page.label(), item_rect[0] + 44.0, y + 8.0, rgba(
            label_color[0], label_color[1], label_color[2], alpha
        ), is_active);
        
        // Subtitle for home page
        if *page == MenuPage::Home {
            batch.text(atlas, "Main Menu", item_rect[0] + 44.0, y + 22.0, rgba(
                palette::TEXT_MUTED[0], palette::TEXT_MUTED[1], palette::TEXT_MUTED[2], alpha
            ), false);
        }
    }
    
    // Version info at bottom
    let version_y = py + panel_h - 36.0;
    batch.text(atlas, "VlaKeeUI v0.1.0", px + 8.0, version_y, rgba(
        palette::TEXT_MUTED[0], palette::TEXT_MUTED[1], palette::TEXT_MUTED[2], alpha
    ), false);
    batch.text(atlas, "by VkYakee", px + 8.0, version_y + 14.0, rgba(
        palette::TEXT_MUTED[0], palette::TEXT_MUTED[1], palette::TEXT_MUTED[2], alpha
    ), false);
}

/// Draws the current page content
fn draw_page_content(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    state: &VlaKeeState,
    open_ease: f32,
) {
    let alpha = (255.0 * open_ease) as u8;
    
    match state.current_page {
        MenuPage::Home => draw_home_content(batch, atlas, x, y, w, h, state, alpha),
        MenuPage::SeedCracker => draw_seedcracker_content(batch, atlas, x, y, w, h, state, alpha),
        MenuPage::VKsc => draw_vksc_content(batch, atlas, x, y, w, h, state, alpha),
        MenuPage::HorizonMatica => draw_horizonmatica_content(batch, atlas, x, y, w, h, state, alpha),
        MenuPage::VkMap => draw_vkmap_content(batch, atlas, x, y, w, h, state, alpha),
        MenuPage::Settings => draw_settings_content(batch, atlas, x, y, w, h, state, alpha),
    }
}

/// Home page content - list of mods
fn draw_home_content(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    state: &VlaKeeState,
    alpha: u8,
) {
    let mut cy = y;
    let item_h = 44.0;
    let spacing = 8.0;
    
    // Header
    batch.text(atlas, "Installed Mods", x, cy, rgba(
        palette::TEXT_PRIMARY[0], palette::TEXT_PRIMARY[1], palette::TEXT_PRIMARY[2], alpha
    ), true);
    cy += 28.0;
    
    if state.mod_list.is_empty() {
        batch.text(atlas, "No mods loaded", x, cy, rgba(
            palette::TEXT_MUTED[0], palette::TEXT_MUTED[1], palette::TEXT_MUTED[2], alpha
        ), false);
        batch.text(atlas, "Restart to load VkYakeeMods", x, cy + 16.0, rgba(
            palette::TEXT_MUTED[0], palette::TEXT_MUTED[1], palette::TEXT_MUTED[2], alpha
        ), false);
        return;
    }
    
    for (i, mod_meta) in state.mod_list.iter().enumerate() {
        let item_y = cy + i as f32 * (item_h + spacing);
        let gradient_pos = (i as f32 / state.mod_list.len().max(1) as f32).clamp(0.0, 1.0);
        let accent = gradient_color(gradient_pos);
        
        // Item background
        batch.rounded(atlas, x, item_y, w, item_h, 10.0, rgba(
            palette::PANEL_SOFT[0], palette::PANEL_SOFT[1], palette::PANEL_SOFT[2], 
            (palette::PANEL_SOFT[3] as f32 * alpha as f32 / 255.0) as u8
        ));
        
        // Accent bar
        batch.fill(atlas, x, item_y, 3.0, item_h, rgba(accent[0], accent[1], accent[2], alpha));
        
        // Mod name
        batch.text(atlas, mod_meta.name, x + 16.0, item_y + 4.0, rgba(
            palette::TEXT_PRIMARY[0], palette::TEXT_PRIMARY[1], palette::TEXT_PRIMARY[2], alpha
        ), true);
        
        // Mod description
        batch.text(atlas, mod_meta.description, x + 16.0, item_y + 22.0, rgba(
            palette::TEXT_MUTED[0], palette::TEXT_MUTED[1], palette::TEXT_MUTED[2], alpha
        ), false);
        
        // Version
        batch.text(atlas, &format!("v{}", mod_meta.version), x + w - 60.0, item_y + 4.0, rgba(
            palette::TEXT_MUTED[0], palette::TEXT_MUTED[1], palette::TEXT_MUTED[2], alpha
        ), false);
        
        // Arrow indicator
        batch.text(atlas, ">", x + w - 20.0, item_y + 12.0, rgba(
            accent[0], accent[1], accent[2], alpha
        ), true);
    }
}

/// SeedCracker page content with action buttons
fn draw_seedcracker_content(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    state: &VlaKeeState,
    alpha: u8,
) {
    let mut cy = y;
    
    // Back button
    draw_back_button(batch, atlas, x, cy, alpha);
    cy += 36.0;
    
    batch.text(atlas, "SeedCracker", x, cy, rgba(
        palette::TEXT_PRIMARY[0], palette::TEXT_PRIMARY[1], palette::TEXT_PRIMARY[2], alpha
    ), true);
    batch.text(atlas, "Quick Actions", x, cy + 20.0, rgba(
        palette::TEXT_MUTED[0], palette::TEXT_MUTED[1], palette::TEXT_MUTED[2], alpha
    ), false);
    cy += 50.0;
    
    // Action buttons grid
    let actions = [
        ("📊 Toggle HUD", palette::GREEN),
        ("🔄 Collect", palette::GREEN_DARK),
        ("🎯 Solve Seed", palette::RED),
        ("🔍 Finders", palette::ACCENT),
        ("👁️ ESP", palette::GREEN),
        ("📁 Export CSV", palette::BORDER),
        ("⚠️ Hard Reset", palette::RED_DARK),
    ];
    
    draw_action_grid(batch, atlas, x, cy, w, &actions, alpha);
}

/// VKsc page content - SeedCracker Quick Shortcuts
fn draw_vksc_content(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    state: &VlaKeeState,
    alpha: u8,
) {
    let mut cy = y;
    
    // Back button
    draw_back_button(batch, atlas, x, cy, alpha);
    cy += 36.0;
    
    batch.text(atlas, "VKsc - SeedCracker", x, cy, rgba(
        palette::TEXT_PRIMARY[0], palette::TEXT_PRIMARY[1], palette::TEXT_PRIMARY[2], alpha
    ), true);
    batch.text(atlas, "Quick Shortcuts", x, cy + 20.0, rgba(
        palette::TEXT_MUTED[0], palette::TEXT_MUTED[1], palette::TEXT_MUTED[2], alpha
    ), false);
    cy += 50.0;
    
    // Action buttons grid - using VKsc specific colors
    let actions = [
        ("📊 Toggle HUD", palette::GREEN),
        ("🔄 Collect", palette::GREEN_DARK),
        ("🎯 Solve Seed", palette::RED),
        ("🔍 Reload Finders", palette::ACCENT),
        ("👁️ ESP", palette::GREEN),
        ("📁 Export CSV", palette::BORDER),
        ("⚠️ Hard Reset", palette::RED_DARK),
    ];
    
    draw_action_grid(batch, atlas, x, cy, w, &actions, alpha);
}

/// HorizonMatica page content
fn draw_horizonmatica_content(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    state: &VlaKeeState,
    alpha: u8,
) {
    let mut cy = y;
    
    draw_back_button(batch, atlas, x, cy, alpha);
    cy += 36.0;
    
    batch.text(atlas, "HorizonMatica", x, cy, rgba(
        palette::TEXT_PRIMARY[0], palette::TEXT_PRIMARY[1], palette::TEXT_PRIMARY[2], alpha
    ), true);
    batch.text(atlas, "Quick Actions", x, cy + 20.0, rgba(
        palette::TEXT_MUTED[0], palette::TEXT_MUTED[1], palette::TEXT_MUTED[2], alpha
    ), false);
    cy += 50.0;
    
    let actions = [
        ("📐 Printer", palette::GREEN),
        ("👻 Ghost Blocks", palette::GREEN_DARK),
        ("🤖 Auto-Build", palette::ACCENT),
        ("📋 Material List", palette::BORDER),
        ("🔧 Settings", palette::GREEN),
        ("📁 Load Schematic", palette::RED_DARK),
    ];
    
    draw_action_grid(batch, atlas, x, cy, w, &actions, alpha);
}

/// VkMap page content
fn draw_vkmap_content(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    state: &VlaKeeState,
    alpha: u8,
) {
    let mut cy = y;
    
    draw_back_button(batch, atlas, x, cy, alpha);
    cy += 36.0;
    
    batch.text(atlas, "VkMap", x, cy, rgba(
        palette::TEXT_PRIMARY[0], palette::TEXT_PRIMARY[1], palette::TEXT_PRIMARY[2], alpha
    ), true);
    batch.text(atlas, "Quick Actions", x, cy + 20.0, rgba(
        palette::TEXT_MUTED[0], palette::TEXT_MUTED[1], palette::TEXT_MUTED[2], alpha
    ), false);
    cy += 50.0;
    
    let actions = [
        ("🗺️ Minimap", palette::GREEN),
        ("👁️ Radar", palette::GREEN_DARK),
        ("📍 Waypoints", palette::ACCENT),
        ("🕳️ Cave Mode", palette::BORDER),
        ("🏗️ Structures", palette::GREEN),
        ("🔄 Sync Seed", palette::RED_DARK),
        ("📐 Perspective", palette::GREEN),
        ("💾 Save Data", palette::RED),
    ];
    
    draw_action_grid(batch, atlas, x, cy, w, &actions, alpha);
}

/// Settings page content
fn draw_settings_content(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    state: &VlaKeeState,
    alpha: u8,
) {
    let mut cy = y;
    
    draw_back_button(batch, atlas, x, cy, alpha);
    cy += 36.0
    
    batch.text(atlas, "Settings", x, cy, rgba(
        palette::TEXT_PRIMARY[0], palette::TEXT_PRIMARY[1], palette::TEXT_PRIMARY[2], alpha
    ), true);
    batch.text(atlas, "VlaKeeUI Configuration", x, cy + 20.0, rgba(
        palette::TEXT_MUTED[0], palette::TEXT_MUTED[1], palette::TEXT_MUTED[2], alpha
    ), false);
    cy += 50.0;
    
    // UI Scale
    draw_slider_setting(batch, atlas, x, cy, w, "UI Scale", state.scale_percent as f32, 75.0, 150.0, alpha);
    cy += 60.0;
    
    // Panel Opacity
    draw_slider_setting(batch, atlas, x, cy, w, "Panel Opacity", state.panel_opacity as f32, 96.0, 245.0, alpha);
    cy += 60.0;
    
    // Show Gradient Toggle
    draw_toggle_setting(batch, atlas, x, cy, w, "Gradient Accent", state.show_gradient, alpha);
    cy += 50.0;
    
    // Button Position (info)
    batch.rounded(atlas, x, cy, w, 40.0, 8.0, rgba(
        palette::PANEL_SOFT[0], palette::PANEL_SOFT[1], palette::PANEL_SOFT[2], 
        (palette::PANEL_SOFT[3] as f32 * alpha as f32 / 255.0) as u8
    ));
    batch.text(atlas, "Button Position", x + 16.0, cy + 6.0, rgba(
        palette::TEXT_PRIMARY[0], palette::TEXT_PRIMARY[1], palette::TEXT_PRIMARY[2], alpha
    ), true);
    batch.text(atlas, "Drag the VK button to reposition", x + 16.0, cy + 22.0, rgba(
        palette::TEXT_MUTED[0], palette::TEXT_MUTED[1], palette::TEXT_MUTED[2], alpha
    ), false);
    cy += 50.0;
    
    // Reset position button
    draw_action_button(batch, atlas, x, cy, w, "Reset Button Position", palette::RED_DARK, alpha);
    cy += 50.0;
    
    // Version info
    batch.text(atlas, "VlaKeeUI v0.1.0", x, cy, rgba(
        palette::TEXT_MUTED[0], palette::TEXT_MUTED[1], palette::TEXT_MUTED[2], alpha
    ), false);
    batch.text(atlas, "Starling Integration", x, cy + 16.0, rgba(
        palette::TEXT_MUTED[0], palette::TEXT_MUTED[1], palette::TEXT_MUTED[2], alpha
    ), false);
}

/// Helper: Draw action button grid (2 columns)
fn draw_action_grid(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    x: f32,
    y: f32,
    w: f32,
    actions: &[(&str, [u8; 4])],
    alpha: u8,
) {
    let btn_w = (w - 12.0) * 0.5;
    let btn_h = 40.0;
    let spacing_x = 12.0;
    let spacing_y = 48.0;
    
    for (i, (label, color)) in actions.iter().enumerate() {
        let row = i / 2;
        let col = i % 2;
        let bx = x + col as f32 * (btn_w + spacing_x);
        let by = y + row as f32 * spacing_y;
        
        let btn_alpha = (color[3] as f32 * alpha as f32 / 255.0) as u8;
        batch.fill(atlas, bx, by, btn_w, btn_h, rgba(color[0], color[1], color[2], btn_alpha));
        batch.text(atlas, label, bx + btn_w * 0.5, by + 12.0, rgba(255, 255, 255, alpha), true);
    }
}

/// Helper: Draw back button
fn draw_back_button(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    x: f32,
    y: f32,
    alpha: u8,
) {
    batch.rounded(atlas, x, y, 70.0, 28.0, 8.0, rgba(
        palette::SURFACE[0], palette::SURFACE[1], palette::SURFACE[2], alpha
    ));
    batch.text(atlas, "< Back", x + 8.0, y + 8.0, rgba(
        palette::ACCENT[0], palette::ACCENT[1], palette::ACCENT[2], alpha
    ), true);
}

/// Helper: Draw action button
fn draw_action_button(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    x: f32,
    y: f32,
    w: f32,
    label: &str,
    color: [u8; 4],
    alpha: u8,
) {
    let btn_alpha = (color[3] as f32 * alpha as f32 / 255.0) as u8;
    batch.fill(atlas, x, y, w, 36.0, rgba(color[0], color[1], color[2], btn_alpha));
    batch.text(atlas, label, x + w * 0.5, y + 12.0, rgba(255, 255, 255, alpha), true);
}

/// Helper: Draw slider setting
fn draw_slider_setting(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    x: f32,
    y: f32,
    w: f32,
    label: &str,
    value: f32,
    min: f32,
    max: f32,
    alpha: u8,
) {
    batch.text(atlas, label, x, y, rgba(
        palette::TEXT_PRIMARY[0], palette::TEXT_PRIMARY[1], palette::TEXT_PRIMARY[2], alpha
    ), true);
    
    let slider_w = w - 60.0;
    let slider_x = x + 16.0;
    let slider_y = y + 28.0;
    
    // Track
    batch.fill(atlas, slider_x, slider_y, slider_w, 6.0, rgba(
        palette::BORDER[0], palette::BORDER[1], palette::BORDER[2], alpha
    ));
    
    // Fill
    let t = ((value - min) / (max - min)).clamp(0.0, 1.0);
    let fill_w = slider_w * t;
    let fill_color = gradient_color(t);
    batch.fill(atlas, slider_x, slider_y, fill_w, 6.0, rgba(
        fill_color[0], fill_color[1], fill_color[2], alpha
    ));
    
    // Handle
    let handle_x = slider_x + fill_w - 6.0;
    batch.fill(atlas, handle_x, slider_y - 4.0, 14.0, 14.0, rgba(
        palette::ACCENT_TEXT[0], palette::ACCENT_TEXT[1], palette::ACCENT_TEXT[2], alpha
    ));
    
    // Value text
    batch.text(atlas, &format!("{:.0}", value), x + w - 44.0, y, rgba(
        palette::TEXT_MUTED[0], palette::TEXT_MUTED[1], palette::TEXT_MUTED[2], alpha
    ), true);
}

/// Helper: Draw toggle setting
fn draw_toggle_setting(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    x: f32,
    y: f32,
    w: f32,
    label: &str,
    enabled: bool,
    alpha: u8,
) {
    batch.text(atlas, label, x, y, rgba(
        palette::TEXT_PRIMARY[0], palette::TEXT_PRIMARY[1], palette::TEXT_PRIMARY[2], alpha
    ), true);
    
    let toggle_w = 48.0;
    let toggle_h = 26.0;
    let toggle_x = x + w - toggle_w - 16.0;
    let toggle_y = y - 4.0;
    
    let (bg_color, knob_x) = if enabled {
        (palette::GREEN, toggle_x + toggle_w - 20.0)
    } else {
        (palette::RED_DARK, toggle_x)
    };
    
    batch.fill(atlas, toggle_x, toggle_y, toggle_w, toggle_h, rgba(
        bg_color[0], bg_color[1], bg_color[2], alpha
    ));
    batch.fill(atlas, knob_x, toggle_y + 2.0, 22.0, 22.0, rgba(
        palette::ACCENT_TEXT[0], palette::ACCENT_TEXT[1], palette::ACCENT_TEXT[2], alpha
    ));
}
