//! VKsc page - SeedCracker quick shortcuts for VlaKeeUI
//!
//! Provides quick access buttons to execute SeedCracker commands
//! from the floating menu (accessed via VKsc tab)

use crate::menu::{PageContext, PageResult, MenuAction, VlaKeeState, RenderContext, TouchEvent, TouchPhase};
use crate::render::{palette, gradient_color};

/// VKsc page with quick action buttons for SeedCracker
pub struct VKscPage;

impl VKscPage {
    pub fn new() -> Self {
        Self
    }
}

/// Quick action buttons for SeedCracker - each has label, command, and color
const VKSC_ACTIONS: &[(&str, &str, [u8; 4])] = &[
    ("📊 Toggle HUD", "/sc gui", palette::GREEN),
    ("🔄 Collect", "/sc collect", palette::GREEN_DARK),
    ("🎯 Solve Seed", "/sc solve", palette::RED),
    ("🔍 Reload Finders", "/sc finder reload", palette::ACCENT),
    ("👁️ ESP", "/sc esp", palette::GREEN),
    ("📁 Export CSV", "/sc export", palette::BORDER),
    ("⚠️ Hard Reset", "/sc reset", palette::RED_DARK),
];

impl super::Page for VKscPage {
    fn render(&mut self, ctx: &mut PageContext, render_ctx: &mut RenderContext) -> PageResult {
        let (w, h) = (render_ctx.screen_size[0], render_ctx.screen_size[1]);
        let panel_w = 320.0;
        let panel_h = 420.0;
        let side_w = 80.0;
        let btn_x = (ctx.state.button_pos[0] * w).clamp(6.0, (w - 48.0 - 6.0).max(6.0));
        let btn_y = (ctx.state.button_pos[1] * h).clamp(4.0, (h - 42.0 - 4.0).max(4.0));
        let px = (btn_x - panel_w - 16.0).max(8.0);
        let py = (btn_y - panel_h * 0.5).clamp(8.0, (h - panel_h - 8.0).max(8.0));
        
        // Animation easing
        let open_t = ctx.state.open_anim;
        let open_ease = 1.0 - (1.0 - open_t).powi(3);
        let alpha = (255.0 * open_ease) as u8;
        
        if alpha == 0 {
            return PageResult::default();
        }
        
        // Get render primitives
        let batch = unsafe { &mut *render_ctx.batch };
        let atlas = unsafe { &*render_ctx.atlas };
        
        let content_x = px + side_w + 8.0;
        let content_y = py + 56.0 + 8.0;
        let content_w = panel_w - side_w - 16.0;
        
        // Draw sidebar with gradient accent
        let sidebar_color = gradient_color(MenuPage::VKsc.gradient_pos());
        let sidebar_alpha = (sidebar_color[3] as f32 * alpha as f32 / 255.0) as u8;
        batch.fill(atlas, px, py + 48.0, side_w, panel_h - 56.0, 
            crate::render::rgba(sidebar_color[0], sidebar_color[1], sidebar_color[2], sidebar_alpha));
        
        // VKsc label in sidebar
        batch.text(atlas, "VK", px + 8.0, py + 64.0, 
            crate::render::rgba(palette::ACCENT_TEXT[0], palette::ACCENT_TEXT[1], palette::ACCENT_TEXT[2], alpha), true);
        batch.text(atlas, "sc", px + 8.0, py + 82.0, 
            crate::render::rgba(palette::ACCENT_TEXT[0], palette::ACCENT_TEXT[1], palette::ACCENT_TEXT[2], alpha), true);
        
        // Back button
        draw_back_button(batch, atlas, content_x, content_y - 56.0, alpha);
        
        // Page title
        batch.text(atlas, "VKsc - SeedCracker Quick Actions", content_x, content_y - 16.0, 
            crate::render::rgba(palette::TEXT_PRIMARY[0], palette::TEXT_PRIMARY[1], palette::TEXT_PRIMARY[2], alpha), true);
        
        // Action buttons - 2 per row
        let btn_w = (content_w - 12.0) * 0.5;
        let btn_h = 40.0;
        let start_y = content_y + 50.0;
        let spacing_y = 48.0;
        let spacing_x = 12.0;
        
        for (i, (label, _cmd, color)) in VKSC_ACTIONS.iter().enumerate() {
            let row = i / 2;
            let col = i % 2;
            let x = content_x + col as f32 * (btn_w + spacing_x);
            let y = start_y + row as f32 * spacing_y;
            
            let btn_alpha = (color[3] as f32 * alpha as f32 / 255.0) as u8;
            batch.rounded(atlas, x, y, btn_w, btn_h, 8.0, 
                crate::render::rgba(color[0], color[1], color[2], btn_alpha));
            batch.text(atlas, label, x + btn_w * 0.5, y + 12.0, 
                crate::render::rgba(255, 255, 255, alpha), true);
        }
        
        PageResult::default()
    }
    
    fn handle_touch(&mut self, ctx: &mut PageContext, touch: TouchEvent, render_ctx: &RenderContext) -> bool {
        if touch.phase != TouchPhase::Ended {
            return false;
        }
        
        let (w, h) = (render_ctx.screen_size[0], render_ctx.screen_size[1]);
        let panel_w = 320.0;
        let panel_h = 420.0;
        let side_w = 80.0;
        let btn_x = (ctx.state.button_pos[0] * w).clamp(6.0, (w - 48.0 - 6.0).max(6.0));
        let btn_y = (ctx.state.button_pos[1] * h).clamp(4.0, (h - 42.0 - 4.0).max(4.0));
        let px = (btn_x - panel_w - 16.0).max(8.0);
        let py = (btn_y - panel_h * 0.5).clamp(8.0, (h - panel_h - 8.0).max(8.0));
        let content_x = px + side_w + 8.0;
        let content_y = py + 56.0 + 8.0;
        let content_w = panel_w - side_w - 16.0;
        
        // Back button
        let back_rect = [content_x, content_y - 56.0, 70.0, 28.0];
        if touch.pos[0] >= back_rect[0] 
            && touch.pos[0] <= back_rect[0] + back_rect[2]
            && touch.pos[1] >= back_rect[1]
            && touch.pos[1] <= back_rect[1] + back_rect[3] {
            ctx.state.dispatch(MenuAction::GoBack);
            return true;
        }
        
        // Action buttons - 2 per row
        let btn_w = (content_w - 12.0) * 0.5;
        let btn_h = 40.0;
        let start_y = content_y + 50.0;
        let spacing_y = 48.0;
        let spacing_x = 12.0;
        
        for (i, (_label, cmd, _color)) in VKSC_ACTIONS.iter().enumerate() {
            let row = i / 2;
            let col = i % 2;
            let x = content_x + col as f32 * (btn_w + spacing_x);
            let y = start_y + row as f32 * spacing_y;
            
            if touch.pos[0] >= x 
                && touch.pos[0] <= x + btn_w
                && touch.pos[1] >= y
                && touch.pos[1] <= y + btn_h {
                ctx.state.dispatch(MenuAction::RunCommand(cmd.to_string()));
                return true;
            }
        }
        
        false
    }
}

use crate::menu::MenuPage;

/// Helper: Draw back button
fn draw_back_button(
    batch: &mut crate::render::HudBatch,
    atlas: &crate::render::HudAtlas,
    x: f32,
    y: f32,
    alpha: u8,
) {
    batch.rounded(atlas, x, y, 70.0, 28.0, 8.0, crate::render::rgba(
        palette::SURFACE[0], palette::SURFACE[1], palette::SURFACE[2], alpha
    ));
    batch.text(atlas, "< Back", x + 8.0, y + 8.0, crate::render::rgba(
        palette::ACCENT[0], palette::ACCENT[1], palette::ACCENT[2], alpha
    ), true);
}
