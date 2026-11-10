//! Home page - lists all mods

use crate::menu::{PageContext, PageResult, MenuAction, VlaKeeState, RenderContext, TouchEvent, TouchPhase};
use vkyakee_mods::api::ModMetadata;

pub struct HomePage;

impl HomePage {
    pub fn new() -> Self {
        Self
    }
}

impl super::Page for HomePage {
    fn render(&mut self, _ctx: &mut PageContext, _render_ctx: &mut RenderContext) -> PageResult {
        PageResult::default()
    }
    
    fn handle_touch(&mut self, ctx: &mut PageContext, touch: TouchEvent, render_ctx: &RenderContext) -> bool {
        if touch.phase != TouchPhase::Ended {
            return false;
        }
        
        let (w, h) = (render_ctx.screen_size[0], render_ctx.screen_size[1]);
        let panel_w = 320.0;
        let side_w = 80.0;
        let btn_x = (ctx.state.button_pos[0] * w).clamp(6.0, (w - 48.0 - 6.0).max(6.0));
        let btn_y = (ctx.state.button_pos[1] * h).clamp(4.0, (h - 42.0 - 4.0).max(4.0));
        let px = (btn_x - panel_w - 16.0).max(8.0);
        let py = (btn_y - 420.0 * 0.5).clamp(8.0, (h - 420.0 - 8.0).max(8.0));
        let content_x = px + side_w + 8.0;
        let content_y = py + 56.0 + 8.0;
        let content_w = panel_w - side_w - 16.0;
        
        // Check mod item clicks
        let item_h = 44.0;
        let spacing = 8.0;
        let start_y = content_y + 28.0; // After header
        
        for (i, _mod_meta) in ctx.mod_list.iter().enumerate() {
            let item_y = start_y + i as f32 * (item_h + spacing);
            let item_rect = [content_x, item_y, content_w, item_h];
            
            if touch.pos[0] >= item_rect[0] 
                && touch.pos[0] <= item_rect[0] + item_rect[2]
                && touch.pos[1] >= item_rect[1]
                && touch.pos[1] <= item_rect[1] + item_rect[3] {
                
                let page = match ctx.mod_list[i].id {
                    "seedcracker" => crate::menu::MenuPage::SeedCracker,
                    "horizonmatica" => crate::menu::MenuPage::HorizonMatica,
                    "vkmap" => crate::menu::MenuPage::VkMap,
                    "vlakeeui" => crate::menu::MenuPage::Settings,
                    _ => crate::menu::MenuPage::Home,
                };
                
                ctx.state.dispatch(MenuAction::OpenPage(page));
                return true;
            }
        }
        
        false
    }
}
