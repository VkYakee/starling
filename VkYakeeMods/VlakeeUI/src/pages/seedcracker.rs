//! SeedCracker page for VlaKeeUI - Quick access to SeedCracker commands

use crate::menu::{PageContext, PageResult, MenuAction, VlaKeeState, RenderContext, TouchEvent, TouchPhase};
use crate::render::{palette, gradient_color};

/// SeedCracker page with quick action buttons
pub struct SeedCrackerPage;

impl SeedCrackerPage {
    pub fn new() -> Self {
        Self
    }
}

/// Quick action buttons for SeedCracker
const SC_ACTIONS: &[(&str, &str, [u8; 4])] = &[
    ("📊 Toggle HUD", "sc gui", palette::GREEN),
    ("🔄 Collect", "sc collect", palette::GREEN_DARK),
    ("🎯 Solve Seed", "sc solve", palette::RED),
    ("🔍 Finders", "sc finder reload", palette::ACCENT),
    ("👁️ ESP", "sc esp", palette::GREEN),
    ("📁 Export CSV", "sc export", palette::BORDER),
    ("⚠️ Hard Reset", "sc reset", palette::RED_DARK),
];

impl super::Page for SeedCrackerPage {
    fn render(&mut self, _ctx: &mut PageContext, _render_ctx: &mut RenderContext) -> PageResult {
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
        
        for (i, (_label, cmd, _color)) in SC_ACTIONS.iter().enumerate() {
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
