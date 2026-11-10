//! VlaKeeUI Adapter: Implements VkMod trait for integration with VkYakeeMods hub

use crate::menu::{VlaKeeState, MenuAction, MenuPage};
use crate::render::{draw_vlakee_menu, vlakee_state, init_vlakee_state};
use vkyakee_mods::api::{ModMetadata, VkMod};
use std::sync::Mutex;

/// Global VlaKeeUI mod instance state
static VLAKEE_MOD_STATE: Mutex<Option<VlaKeeModState>> = Mutex::new(None);

/// Internal state for the VlaKeeUI mod
struct VlaKeeModState {
    state: VlaKeeState,
    last_update: std::time::Instant,
}

impl Default for VlaKeeModState {
    fn default() -> Self {
        Self {
            state: VlaKeeState::new(),
            last_update: std::time::Instant::now(),
        }
    }
}

/// VlaKeeUI Mod - implements VkMod for integration with VkYakeeMods hub
pub struct VlaKeeUiMod {
    metadata: ModMetadata,
}

impl VlaKeeUiMod {
    pub fn new() -> Self {
        // Initialize the global render state
        init_vlakee_state();
        
        Self {
            metadata: ModMetadata {
                id: "vlakeeui",
                name: "VlaKeeUI",
                version: "0.1.0",
                description: "Floating button menu with green-red gradient for VkYakeeMods administration",
                author: "VkYakee",
            },
        }
    }
    
    /// Gets mutable access to the internal state
    fn with_state<F, R>(&self, f: F) -> R 
    where 
        F: FnOnce(&mut VlaKeeModState) -> R 
    {
        let mut guard = VLAKEE_MOD_STATE.lock().unwrap_or_else(|e| e.into_inner());
        if guard.is_none() {
            *guard = Some(VlaKeeModState::default());
        }
        f(guard.as_mut().unwrap())
    }
    
    /// Gets the shared VlaKeeState for rendering
    pub fn get_render_state() -> VlaKeeState {
        if let Ok(guard) = VLAKEE_MOD_STATE.lock() {
            if let Some(state) = guard.as_ref() {
                return state.state.clone();
            }
        }
        VlaKeeState::new()
    }
    
    /// Updates the shared state from render state
    pub fn sync_from_render(render_state: VlaKeeState) {
        if let Ok(mut guard) = VLAKEE_MOD_STATE.lock() {
            if let Some(state) = guard.as_mut() {
                state.state = render_state;
            }
        }
    }
    
    /// Executes a command via the VkModHub
    fn execute_command(&self, command: &str) {
        // Use the hub's on_command dispatcher
        // This needs to be done from outside the hub lock
        std::thread::spawn({
            let cmd = command.to_string();
            move || {
                // Small delay to avoid reentrancy issues
                std::thread::sleep(std::time::Duration::from_millis(10));
                let mut ran = false;
                vkyakee_mods::on_command(&cmd, &mut ran);
            }
        });
    }
}

impl VkMod for VlaKeeUiMod {
    fn metadata(&self) -> ModMetadata {
        self.metadata
    }
    
    fn on_tick(&mut self) {
        self.with_state(|state| {
            let now = std::time::Instant::now();
            let dt = now.duration_since(state.last_update).as_secs_f32();
            state.last_update = now;
            
            // Update animations
            state.state.update_animations(dt);
            
            // Sync with global render state
            if let Ok(mut render_guard) = vlakee_state().lock() {
                if let Some(render_state) = render_guard.as_mut() {
                    // Preserve render-specific state but update position/animations
                    render_state.open_anim = state.state.open_anim;
                    render_state.page_anim = state.state.page_anim;
                    render_state.button_pos = state.state.button_pos;
                    render_state.open = state.state.open;
                    render_state.current_page = state.state.current_page;
                    render_state.previous_page = state.state.previous_page;
                    render_state.dragging = state.state.dragging;
                    render_state.drag_moved = state.state.drag_moved;
                }
            }
        });
    }
    
    fn on_command(&mut self, command: &str) -> bool {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return false;
        }
        
        // Handle vui/vlakee commands
        if parts[0].eq_ignore_ascii_case("vui") || parts[0].eq_ignore_ascii_case("vlakee") {
            if parts.len() < 2 {
                return false;
            }
            
            match parts[1].to_ascii_lowercase().as_str() {
                "toggle" | "open" | "menu" => {
                    self.with_state(|state| {
                        state.state.dispatch(MenuAction::ToggleMenu);
                    });
                    true
                }
                "close" => {
                    self.with_state(|state| {
                        state.state.dispatch(MenuAction::CloseMenu);
                    });
                    true
                }
                "home" => {
                    self.with_state(|state| {
                        state.state.dispatch(MenuAction::OpenPage(MenuPage::Home));
                    });
                    true
                }
                "seedcracker" | "sc" => {
                    self.with_state(|state| {
                        state.state.dispatch(MenuAction::OpenPage(MenuPage::SeedCracker));
                    });
                    true
                }
                "vksc" => {
                    self.with_state(|state| {
                        state.state.dispatch(MenuAction::OpenPage(MenuPage::VKsc));
                    });
                    true
                }
                "horizonmatica" | "hm" => {
                    self.with_state(|state| {
                        state.state.dispatch(MenuAction::OpenPage(MenuPage::HorizonMatica));
                    });
                    true
                }
                "vkmap" | "map" => {
                    self.with_state(|state| {
                        state.state.dispatch(MenuAction::OpenPage(MenuPage::VkMap));
                    });
                    true
                }
                "settings" | "config" => {
                    self.with_state(|state| {
                        state.state.dispatch(MenuAction::OpenPage(MenuPage::Settings));
                    });
                    true
                }
                "show" => {
                    self.with_state(|state| {
                        state.state.visible = true;
                    });
                    true
                }
                "hide" => {
                    self.with_state(|state| {
                        state.state.visible = false;
                    });
                    true
                }
                "reset" => {
                    self.with_state(|state| {
                        state.state.button_pos = [0.44, 0.05];
                    });
                    true
                }
                "gradient" => {
                    if parts.len() >= 3 {
                        let enabled = parts[2].parse::<bool>().unwrap_or(true);
                        self.with_state(|state| {
                            state.state.show_gradient = enabled;
                        });
                    }
                    true
                }
                "scale" => {
                    if parts.len() >= 3 {
                        if let Ok(scale) = parts[2].parse::<u16>() {
                            self.with_state(|state| {
                                state.state.set_scale_percent(scale);
                            });
                        }
                    }
                    true
                }
                "opacity" => {
                    if parts.len() >= 3 {
                        if let Ok(opacity) = parts[2].parse::<u8>() {
                            self.with_state(|state| {
                                state.state.set_opacity(opacity);
                            });
                        }
                    }
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }
    
    fn hud_lines(&self) -> Vec<String> {
        self.with_state(|state| {
            let mut lines = Vec::new();
            if state.state.visible {
                lines.push(format!("VlaKeeUI: {}", if state.state.open { "OPEN" } else { "CLOSED" }));
                lines.push(format!("Page: {}", state.state.current_page.label()));
            }
            lines
        })
    }
    
    fn settings_buttons(&self) -> Vec<String> {
        vec![
            "Toggle Menu".to_string(),
            "Open Settings".to_string(),
            "Reset Position".to_string(),
            "Toggle Gradient".to_string(),
        ]
    }
    
    fn on_settings_click(&mut self, button_idx: usize) {
        self.with_state(|state| {
            match button_idx {
                0 => state.state.dispatch(MenuAction::ToggleMenu),
                1 => state.state.dispatch(MenuAction::OpenPage(MenuPage::Settings)),
                2 => state.state.button_pos = [0.44, 0.05],
                3 => state.state.show_gradient = !state.state.show_gradient,
                _ => {}
            }
        });
    }
}

impl Default for VlaKeeUiMod {
    fn default() -> Self {
        Self::new()
    }
}

/// Called from Starling's mod_render to draw the VlaKeeUI menu
/// This integrates with the existing draw_vkyakee_menu pattern
pub fn draw_vlakee_ui(batch: &mut crate::render::HudBatch, atlas: &crate::render::HudAtlas) {
    draw_vlakee_menu(batch, atlas)
}

/// Toggles the VlaKeeUI menu (can be called from keybinds)
pub fn toggle_vlakee_menu() {
    if let Ok(mut guard) = VLAKEE_MOD_STATE.lock() {
        if let Some(state) = guard.as_mut() {
            state.state.dispatch(MenuAction::ToggleMenu);
        }
    }
}

/// Checks if VlaKeeUI menu is open
pub fn is_vlakee_menu_open() -> bool {
    if let Ok(guard) = VLAKEE_MOD_STATE.lock() {
        if let Some(state) = guard.as_ref() {
            return state.state.open;
        }
    }
    false
}
