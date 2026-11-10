//! Settings page for VlaKeeUI - Configuration options

use crate::menu::{PageContext, PageResult, MenuAction, VlaKeeState, RenderContext, TouchEvent, TouchPhase};
use crate::render::{palette, gradient_color};

pub struct SettingsPage;

impl SettingsPage {
    pub fn new() -> Self {
        Self
    }
}

impl super::Page for SettingsPage {
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
        
        // UI Scale slider
        let slider_y = content_y + 50.0 + 28.0;
        let slider_rect = [content_x + 16.0, slider_y, content_w - 60.0, 6.0];
        if touch.pos[0] >= slider_rect[0] 
            && touch.pos[0] <= slider_rect[0] + slider_rect[2]
            && touch.pos[1] >= slider_rect[1] - 10.0
            && touch.pos[1] <= slider_rect[1] + 10.0 {
            let t = (touch.pos[0] - slider_rect[0]) / slider_rect[2];
            let new_scale = (75.0 + t * 75.0).round() as u16;
            ctx.state.set_scale_percent(new_scale);
            return true;
        }
        
        // Opacity slider
        let opacity_y = content_y + 50.0 + 60.0 + 28.0;
        let opacity_rect = [content_x + 16.0, opacity_y, content_w - 60.0, 6.0];
        if touch.pos[0] >= opacity_rect[0] 
            && touch.pos[0] <= opacity_rect[0] + opacity_rect[2]
            && touch.pos[1] >= opacity_rect[1] - 10.0
            && touch.pos[1] <= opacity_rect[1] + 10.0 {
            let t = (touch.pos[0] - opacity_rect[0]) / opacity_rect[2];
            let new_opacity = (96.0 + t * 149.0).round() as u8;
            ctx.state.set_opacity(new_opacity);
            return true;
        }
        
        // Gradient toggle
        let toggle_y = content_y + 50.0 + 120.0;
        let toggle_rect = [content_x + content_w - 48.0 - 16.0, toggle_y - 4.0, 48.0, 26.0];
        if touch.pos[0] >= toggle_rect[0] 
            && touch.pos[0] <= toggle_rect[0] + toggle_rect[2]
            && touch.pos[1] >= toggle_rect[1]
            && touch.pos[1] <= toggle_rect[1] + toggle_rect[3] {
            ctx.state.show_gradient = !ctx.state.show_gradient;
            return true;
        }
        
        // Reset position button
        let reset_y = content_y + 50.0 + 170.0;
        let reset_rect = [content_x, reset_y, content_w, 36.0];
        if touch.pos[0] >= reset_rect[0] 
            && touch.pos[0] <= reset_rect[0] + reset_rect[2]
            && touch.pos[1] >= reset_rect[1]
            && touch.pos[1] <= reset_rect[1] + reset_rect[3] {
            ctx.state.button_pos = [0.44, 0.05];
            return true;
        }
        
        false
    }
}
