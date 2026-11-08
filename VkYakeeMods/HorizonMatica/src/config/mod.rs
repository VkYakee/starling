pub mod generic;
pub mod visuals;
pub mod colors;
pub mod hotkeys;
pub mod info_overlays;

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub use generic::GenericConfigs;
pub use visuals::VisualConfigs;
pub use colors::{ColorConfigs, ColorRgba};
pub use hotkeys::HotkeyConfigs;
pub use info_overlays::InfoOverlayConfigs;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Configs {
    pub generic: GenericConfigs,
    pub visuals: VisualConfigs,
    pub colors: ColorConfigs,
    pub hotkeys: HotkeyConfigs,
    pub info_overlays: InfoOverlayConfigs,
}

impl Configs {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let configs: Configs = serde_json::from_str(&contents)?;
        Ok(configs)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}
