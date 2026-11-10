//! VkYakeeMods floating HUD / ModMenu presentation model.
 //! Rendering is intentionally separated from module execution and solver state.

 use crate::api::ModMetadata;

 /// UI palette. Alpha is explicit so renderers can preserve translucency.
 pub mod palette {
     pub const PANEL: [u8; 4] = [12, 16, 18, 218];       // ~85% opaque charcoal
     pub const PANEL_SOFT: [u8; 4] = [18, 22, 24, 188];  // ~74% opaque
     pub const GREEN: [u8; 4] = [52, 214, 112, 255];
     pub const GREEN_DARK: [u8; 4] = [20, 126, 72, 255];
     pub const RED: [u8; 4] = [244, 66, 82, 255];
     pub const RED_DARK: [u8; 4] = [150, 30, 48, 255];
     pub const TEXT: [u8; 4] = [242, 246, 244, 255];
     pub const MUTED: [u8; 4] = [154, 166, 162, 255];
     pub const BORDER: [u8; 4] = [66, 92, 78, 210];
 }

 #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
 pub enum MenuPage {
     #[default]
     Home,
     SeedCracker,
     SeedFinders,
 }

 #[derive(Debug, Clone, Copy, PartialEq, Eq)]
 pub enum MenuAction {
     OpenSeedCracker,
     Close,
     Back,
     ToggleHud,
     OpenFinders,
     NextFinderPage,
 }

 #[derive(Debug, Clone, Copy, PartialEq, Eq)]
 pub enum ModuleStatus {
     Ready,
     Collecting,
     Solving,
     Cracked,
     Paused,
 }

 impl ModuleStatus {
     pub fn label(self) -> &'static str {
         match self {
             Self::Ready => "READY",
             Self::Collecting => "COLLECTING",
             Self::Solving => "SOLVING",
             Self::Cracked => "CRACKED",
             Self::Paused => "PAUSED",
         }
     }

     pub fn color(self) -> [u8; 4] {
         match self {
             Self::Cracked => palette::GREEN,
             Self::Solving | Self::Collecting => palette::GREEN_DARK,
             Self::Paused => palette::RED,
             Self::Ready => palette::MUTED,
         }
     }
 }

 /// Only finished/usable modules are exposed in the first menu iteration.
 pub fn visible_modules() -> Vec<ModMetadata> {
     vec![ModMetadata {
         id: "seedcracker",
         name: "SeedCracker",
         version: "0.1.0",
         description: "Análisis progresivo de chunks y resolución de semillas",
         author: "VkYakee",
     }]
 }

 /// Persistent-friendly menu state; renderer owns pointer/touch handling.
 #[derive(Debug, Clone)]
 pub struct ModMenuState {
     pub open: bool,
     pub visible: bool,
     pub page: MenuPage,
     pub opacity: u8,
     pub scale_percent: u16,
     pub finder_page: u8,
 }

 impl Default for ModMenuState {
     fn default() -> Self {
         Self {
             open: false,
             visible: true,
             page: MenuPage::Home,
             opacity: 218,
             scale_percent: 100,
             finder_page: 0,
         }
     }
 }

 impl ModMenuState {
     pub fn toggle(&mut self) {
         self.open = !self.open;
         if self.open {
             self.page = MenuPage::Home;
         }
     }

     pub fn dispatch(&mut self, action: MenuAction) {
         match action {
             MenuAction::OpenSeedCracker => {
                 self.open = true;
                 self.page = MenuPage::SeedCracker;
             }
             MenuAction::Close => self.open = false,
             MenuAction::Back => self.page = MenuPage::Home,
             MenuAction::ToggleHud => self.visible = !self.visible,
             MenuAction::OpenFinders => { self.open = true; self.page = MenuPage::SeedFinders; self.finder_page = 0; },
            MenuAction::NextFinderPage => self.finder_page = (self.finder_page + 1) % 2,
         }
     }

     pub fn set_opacity(&mut self, opacity: u8) {
         // Keep the panel readable and avoid fully opaque/fully invisible states.
         self.opacity = opacity.clamp(96, 245);
     }

     pub fn set_scale_percent(&mut self, scale: u16) {
         self.scale_percent = scale.clamp(75, 150);
     }
 }

 #[cfg(test)]
 mod tests {
     use super::*;

     #[test]
     fn first_menu_exposes_seedcracker_only() {
         let modules = visible_modules();
         assert_eq!(modules.len(), 1);
         assert_eq!(modules[0].id, "seedcracker");
     }

     #[test]
     fn actions_switch_pages_and_visibility() {
         let mut state = ModMenuState::default();
         state.dispatch(MenuAction::OpenSeedCracker);
         assert!(state.open);
         assert_eq!(state.page, MenuPage::SeedCracker);
         state.dispatch(MenuAction::Back);
         assert_eq!(state.page, MenuPage::Home);
         state.dispatch(MenuAction::ToggleHud);
         assert!(!state.visible);
     }
 }

/// Theme contracts used by Starling's floating-button and menu renderer.
#[derive(Debug, Clone, Copy)]
pub struct FloatingButtonTheme { pub background: [u8;4], pub foreground: [u8;4], pub border: [u8;4] }
impl Default for FloatingButtonTheme { fn default()->Self { Self { background: palette::PANEL, foreground: palette::GREEN, border: palette::BORDER } } }
#[derive(Debug, Clone, Copy)]
pub struct MenuTheme { pub panel: [u8;4], pub text: [u8;4], pub muted: [u8;4], pub accent: [u8;4], pub border: [u8;4] }
impl Default for MenuTheme { fn default()->Self { Self { panel: palette::PANEL, text: palette::TEXT, muted: palette::MUTED, accent: palette::GREEN, border: palette::BORDER } } }
