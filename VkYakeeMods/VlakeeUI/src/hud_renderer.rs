//! Mobile HUD menu and touch buttons ported from Déesse UI (déesse_hudMenuTouch).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VlakeeMenuPage {
    #[default]
    Page1,      // Telemetry & HUD (SmartF3, InvHUD, Durability, Night Vision)
    Page2,      // World & Tools (Chunk Viewer, Nether Portal, Death Compass, Calculator)
    Page3,      // In-Game Options (Camera F5, Gamemode, FOV Presets, Container Shortcuts)
    Page4More,  // SmartF3 More (F1, F3+G, F3+B, F3+H, F3+N, F3+A, F3+C, Shift+F3 Pie)
    Page5Craft, // Quick Craft (Mass Craft mode, 1-Stack, Quick Recipes)
}

impl VlakeeMenuPage {
    pub fn name(&self) -> &'static str {
        match self {
            VlakeeMenuPage::Page1 => "Página 1 (F3, InvHUD, Visión Nocturna)",
            VlakeeMenuPage::Page2 => "Página 2 (Chunk Viewer, Nether, Muerte)",
            VlakeeMenuPage::Page3 => "Página 3 (Opciones Dentro del Juego)",
            VlakeeMenuPage::Page4More => "Página 4 (➕ SmartF3 More & Speedrun)",
            VlakeeMenuPage::Page5Craft => "Página 5 (⚡ Crafteo Rápido / Mass Craft)",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            VlakeeMenuPage::Page1 => VlakeeMenuPage::Page2,
            VlakeeMenuPage::Page2 => VlakeeMenuPage::Page3,
            VlakeeMenuPage::Page3 => VlakeeMenuPage::Page4More,
            VlakeeMenuPage::Page4More => VlakeeMenuPage::Page5Craft,
            VlakeeMenuPage::Page5Craft => VlakeeMenuPage::Page1,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            VlakeeMenuPage::Page1 => VlakeeMenuPage::Page5Craft,
            VlakeeMenuPage::Page2 => VlakeeMenuPage::Page1,
            VlakeeMenuPage::Page3 => VlakeeMenuPage::Page2,
            VlakeeMenuPage::Page4More => VlakeeMenuPage::Page3,
            VlakeeMenuPage::Page5Craft => VlakeeMenuPage::Page4More,
        }
    }

    pub fn toggle(&self) -> Self {
        self.next()
    }
}

pub struct VlakeeHudMenu {
    pub active_page: VlakeeMenuPage,
    pub is_open: bool,
}

impl Default for VlakeeHudMenu {
    fn default() -> Self {
        Self {
            active_page: VlakeeMenuPage::Page1,
            is_open: false,
        }
    }
}

impl VlakeeHudMenu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle_menu(&mut self) -> bool {
        self.is_open = !self.is_open;
        self.is_open
    }

    pub fn toggle_page(&mut self) {
        self.active_page = self.active_page.next();
    }

    pub fn prev_page(&mut self) {
        self.active_page = self.active_page.prev();
    }

    pub fn set_page(&mut self, page: VlakeeMenuPage) {
        self.active_page = page;
    }
}
