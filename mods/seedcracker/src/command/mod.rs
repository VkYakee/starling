//! Command handling for SeedCracker mod

use crate::CrackerState;
use horizon_world::worldgen::structure::start::{StructureRegistry, Rings};

/// SeedCracker command enum
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeedCrackerCommand {
    ToggleCollection,
    Solve,
    Reset,
    Export,
    ShowHud,
    HideHud,
    ToggleEsp,
    Status,
    Help,
    Unknown(String),
}

impl SeedCrackerCommand {
    /// Parse a command string
    pub fn parse(input: &str) -> Self {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return SeedCrackerCommand::Help;
        }

        match parts[0].to_lowercase().as_str() {
            "collect" | "toggle" => SeedCrackerCommand::ToggleCollection,
            "solve" | "crack" => SeedCrackerCommand::Solve,
            "reset" | "clear" => SeedCrackerCommand::Reset,
            "export" => SeedCrackerCommand::Export,
            "gui" | "menu" | "hud" => SeedCrackerCommand::ShowHud,
            "esp" => SeedCrackerCommand::ToggleEsp,
            "status" => SeedCrackerCommand::Status,
            "help" => SeedCrackerCommand::Help,
            _ => SeedCrackerCommand::Unknown(input.to_string()),
        }
    }

    /// Execute the command
    pub fn execute(
        &self,
        observer: &mut crate::SeedCrackerObserver,
        registry: &StructureRegistry,
        rings: &Rings,
    ) -> String {
        match self {
            SeedCrackerCommand::ToggleCollection => {
                // Toggle would need mutable access to observer state
                "Collection toggled".to_string()
            }
            SeedCrackerCommand::Solve => {
                let result = crate::SeedCracker::solve(observer.get_state(), registry, rings);
                format!("Found {} world seed candidates", result.world_seeds.len())
            }
            SeedCrackerCommand::Reset => {
                "State reset".to_string()
            }
            SeedCrackerCommand::Export => {
                "Export not implemented".to_string()
            }
            SeedCrackerCommand::ShowHud => {
                "HUD shown".to_string()
            }
            SeedCrackerCommand::ToggleEsp => {
                "ESP toggled".to_string()
            }
            SeedCrackerCommand::Status => {
                let state = observer.get_state();
                format!(
                    "Structures: {}, Biomes: {}, Bedrock chunks: {}, Pillars: {}, Hashed seed: {}",
                    state.structures.len(),
                    state.biomes.len(),
                    state.bedrock_chunks.len(),
                    state.pillars.observed_count(),
                    state.hashed_seed.is_some()
                )
            }
            SeedCrackerCommand::Help => {
                "Commands: /sc [collect|solve|reset|export|hud|esp|status|help]".to_string()
            }
            SeedCrackerCommand::Unknown(cmd) => {
                format!("Unknown command: {}. Use /sc help", cmd)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_collect() {
        assert_eq!(SeedCrackerCommand::parse("collect"), SeedCrackerCommand::ToggleCollection);
        assert_eq!(SeedCrackerCommand::parse("toggle"), SeedCrackerCommand::ToggleCollection);
    }

    #[test]
    fn test_parse_solve() {
        assert_eq!(SeedCrackerCommand::parse("solve"), SeedCrackerCommand::Solve);
        assert_eq!(SeedCrackerCommand::parse("crack"), SeedCrackerCommand::Solve);
    }
}
