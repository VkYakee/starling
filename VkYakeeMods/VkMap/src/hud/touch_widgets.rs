use serde::{Deserialize, Serialize};

/// 3-Tier Minimap Display Size modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MinimapDisplaySize {
    /// Pequeñito: Compact HUD map (96x96 px).
    Small,
    /// Mediano: Starling native minimap size (144x144 px).
    Medium,
    /// Extra Grande: Fullscreen / expanded tactical navigation map (280x280 px).
    Fullscreen,
}

impl Default for MinimapDisplaySize {
    fn default() -> Self {
        MinimapDisplaySize::Medium
    }
}

impl MinimapDisplaySize {
    pub fn pixel_size(&self) -> u32 {
        match self {
            MinimapDisplaySize::Small => 96,
            MinimapDisplaySize::Medium => 144,
            MinimapDisplaySize::Fullscreen => 280,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            MinimapDisplaySize::Small => "Pequeño (96px)",
            MinimapDisplaySize::Medium => "Mediano (144px)",
            MinimapDisplaySize::Fullscreen => "Extra Grande (280px)",
        }
    }

    pub fn expand(&self) -> Self {
        match self {
            MinimapDisplaySize::Small => MinimapDisplaySize::Medium,
            MinimapDisplaySize::Medium => MinimapDisplaySize::Fullscreen,
            MinimapDisplaySize::Fullscreen => MinimapDisplaySize::Fullscreen,
        }
    }

    pub fn shrink(&self) -> Self {
        match self {
            MinimapDisplaySize::Fullscreen => MinimapDisplaySize::Medium,
            MinimapDisplaySize::Medium => MinimapDisplaySize::Small,
            MinimapDisplaySize::Small => MinimapDisplaySize::Small,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            MinimapDisplaySize::Small => MinimapDisplaySize::Medium,
            MinimapDisplaySize::Medium => MinimapDisplaySize::Fullscreen,
            MinimapDisplaySize::Fullscreen => MinimapDisplaySize::Small,
        }
    }
}

/// Floating touch widget actions located beside the minimap for instant touch control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchWidgetAction {
    ExpandSize,   // [ 🔍+ ]
    ShrinkSize,   // [ 🔍- ]
    ZoomIn,       // [ ➕ ]
    ZoomOut,      // [ ➖ ]
    OpenMapMenu,  // [ 🗺️ ]
}

/// Floating touch widgets container positioned adjacent to the minimap screen corner.
#[derive(Debug, Clone)]
pub struct MinimapTouchWidgets {
    pub display_size: MinimapDisplaySize,
    pub zoom_level: f32, // 0.5, 1.0, 2.0, 4.0
    pub map_menu_open: bool,
}

impl Default for MinimapTouchWidgets {
    fn default() -> Self {
        Self {
            display_size: MinimapDisplaySize::Medium,
            zoom_level: 1.0,
            map_menu_open: false,
        }
    }
}

impl MinimapTouchWidgets {
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles floating widget click.
    pub fn handle_action(&mut self, action: TouchWidgetAction) {
        match action {
            TouchWidgetAction::ExpandSize => {
                self.display_size = self.display_size.expand();
            }
            TouchWidgetAction::ShrinkSize => {
                self.display_size = self.display_size.shrink();
            }
            TouchWidgetAction::ZoomIn => {
                if self.zoom_level < 4.0 {
                    self.zoom_level *= 2.0;
                }
            }
            TouchWidgetAction::ZoomOut => {
                if self.zoom_level > 0.5 {
                    self.zoom_level /= 2.0;
                }
            }
            TouchWidgetAction::OpenMapMenu => {
                self.map_menu_open = !self.map_menu_open;
            }
        }
    }

    /// Returns floating touch widget button labels for rendering beside the map.
    pub fn floating_button_labels(&self) -> Vec<String> {
        vec![
            format!("🔍+ Ampliar [{}]", self.display_size.name()),
            format!("🔍- Reducir"),
            format!("🔎 Zoom: {:.1}x", self.zoom_level),
            format!("🗺️ Menú de Opciones Mapa: {}", if self.map_menu_open { "ABIERTO" } else { "CERRADO" }),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_size_transitions() {
        let size = MinimapDisplaySize::Small;
        assert_eq!(size.pixel_size(), 96);

        let size = size.expand();
        assert_eq!(size, MinimapDisplaySize::Medium);
        assert_eq!(size.pixel_size(), 144);

        let size = size.expand();
        assert_eq!(size, MinimapDisplaySize::Fullscreen);
        assert_eq!(size.pixel_size(), 280);

        let size = size.shrink();
        assert_eq!(size, MinimapDisplaySize::Medium);
    }

    #[test]
    fn test_touch_widget_actions() {
        let mut widgets = MinimapTouchWidgets::new();
        widgets.handle_action(TouchWidgetAction::ExpandSize);
        assert_eq!(widgets.display_size, MinimapDisplaySize::Fullscreen);

        widgets.handle_action(TouchWidgetAction::ZoomIn);
        assert_eq!(widgets.zoom_level, 2.0);

        widgets.handle_action(TouchWidgetAction::OpenMapMenu);
        assert!(widgets.map_menu_open);
    }
}
