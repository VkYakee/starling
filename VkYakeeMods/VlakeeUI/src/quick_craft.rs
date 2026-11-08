//! Quick crafting and Mass Crafting shortcuts ported from Déesse UI (mass_craft.json).
//! Provides rapid crafting for speedrunners and technical Minecraft players.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QuickCraftMode {
    #[default]
    Normal,     // 1 unit at a time
    MassCraft,  // Craft all possible from inventory in 1 touch
    OneStack,   // Craft exactly 1 stack (max 64)
}

impl QuickCraftMode {
    pub fn name(&self) -> &'static str {
        match self {
            QuickCraftMode::Normal => "Normal (1 Crafteo)",
            QuickCraftMode::MassCraft => "Mass Craft (Todo el Inventario)",
            QuickCraftMode::OneStack => "1 Stack (Máx 64)",
        }
    }

    pub fn cycle(&self) -> Self {
        match self {
            QuickCraftMode::Normal => QuickCraftMode::MassCraft,
            QuickCraftMode::MassCraft => QuickCraftMode::OneStack,
            QuickCraftMode::OneStack => QuickCraftMode::Normal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuickRecipe {
    WoodToPlanks,
    PlanksToSticks,
    PlanksToCraftingTable,
    SticksCoalToTorches,
    IngotsToBlock,
    BlockToIngots,
    PaperGunpowderToRockets,
    WheatToBread,
    IronToShield,
    IronToBucket,
}

impl QuickRecipe {
    pub fn name(&self) -> &'static str {
        match self {
            QuickRecipe::WoodToPlanks => "Madera ➔ Tablones (1:4)",
            QuickRecipe::PlanksToSticks => "Tablones ➔ Palos (2:4)",
            QuickRecipe::PlanksToCraftingTable => "Tablones ➔ Mesa Crafteo (4:1)",
            QuickRecipe::SticksCoalToTorches => "Palos + Carbón ➔ Antorchas (1+1:4)",
            QuickRecipe::IngotsToBlock => "Lingotes ➔ Bloque Compacto (9:1)",
            QuickRecipe::BlockToIngots => "Bloque ➔ Lingotes (1:9)",
            QuickRecipe::PaperGunpowderToRockets => "Pólvora + Papel ➔ Cohetes (1+1:3)",
            QuickRecipe::WheatToBread => "Trigo ➔ Pan (3:1)",
            QuickRecipe::IronToShield => "Hierro + Madera ➔ Escudo (1+6:1)",
            QuickRecipe::IronToBucket => "Hierro ➔ Cubo (3:1)",
        }
    }

    /// Returns: (primary_cost_per_craft, output_per_craft)
    pub fn ratio(&self) -> (usize, usize) {
        match self {
            QuickRecipe::WoodToPlanks => (1, 4),
            QuickRecipe::PlanksToSticks => (2, 4),
            QuickRecipe::PlanksToCraftingTable => (4, 1),
            QuickRecipe::SticksCoalToTorches => (1, 4),
            QuickRecipe::IngotsToBlock => (9, 1),
            QuickRecipe::BlockToIngots => (1, 9),
            QuickRecipe::PaperGunpowderToRockets => (1, 3),
            QuickRecipe::WheatToBread => (3, 1),
            QuickRecipe::IronToShield => (1, 1),
            QuickRecipe::IronToBucket => (3, 1),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickCraftResult {
    pub recipe: QuickRecipe,
    pub crafted_count: usize,
    pub input_consumed: usize,
    pub input_leftover: usize,
}

impl QuickCraftResult {
    pub fn format_summary(&self) -> String {
        format!(
            "⚡ Crafteado: {}x {} (Usados: {} materiales, Sobran: {})",
            self.crafted_count,
            self.recipe.name(),
            self.input_consumed,
            self.input_leftover
        )
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuickCraftManager {
    pub mode: QuickCraftMode,
    pub last_crafted: Option<String>,
}

impl QuickCraftManager {
    pub fn new() -> Self {
        Self {
            mode: QuickCraftMode::Normal,
            last_crafted: None,
        }
    }

    pub fn cycle_mode(&mut self) -> QuickCraftMode {
        self.mode = self.mode.cycle();
        self.mode
    }

    /// Simulates/executes quick crafting based on available input units
    pub fn craft(&mut self, recipe: QuickRecipe, available_inputs: usize) -> QuickCraftResult {
        let (cost_per_craft, output_per_craft) = recipe.ratio();
        if cost_per_craft == 0 || available_inputs < cost_per_craft {
            return QuickCraftResult {
                recipe,
                crafted_count: 0,
                input_consumed: 0,
                input_leftover: available_inputs,
            };
        }

        let max_possible_crafts = available_inputs / cost_per_craft;
        let crafts_to_do = match self.mode {
            QuickCraftMode::Normal => 1,
            QuickCraftMode::MassCraft => max_possible_crafts,
            QuickCraftMode::OneStack => {
                // Determine how many crafts fit within 64 output items
                let target_crafts = (64 + output_per_craft - 1) / output_per_craft;
                max_possible_crafts.min(target_crafts)
            }
        };

        let crafted_count = crafts_to_do * output_per_craft;
        let input_consumed = crafts_to_do * cost_per_craft;
        let input_leftover = available_inputs.saturating_sub(input_consumed);

        let result = QuickCraftResult {
            recipe,
            crafted_count,
            input_consumed,
            input_leftover,
        };

        self.last_crafted = Some(result.format_summary());
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_craft_mode_cycling() {
        let mut mgr = QuickCraftManager::new();
        assert_eq!(mgr.mode, QuickCraftMode::Normal);
        mgr.cycle_mode();
        assert_eq!(mgr.mode, QuickCraftMode::MassCraft);
        mgr.cycle_mode();
        assert_eq!(mgr.mode, QuickCraftMode::OneStack);
        mgr.cycle_mode();
        assert_eq!(mgr.mode, QuickCraftMode::Normal);
    }

    #[test]
    fn test_mass_craft_wood_to_planks() {
        let mut mgr = QuickCraftManager::new();
        mgr.mode = QuickCraftMode::MassCraft;

        // 16 logs = 64 planks
        let res = mgr.craft(QuickRecipe::WoodToPlanks, 16);
        assert_eq!(res.crafted_count, 64);
        assert_eq!(res.input_consumed, 16);
        assert_eq!(res.input_leftover, 0);
    }

    #[test]
    fn test_one_stack_limit() {
        let mut mgr = QuickCraftManager::new();
        mgr.mode = QuickCraftMode::OneStack;

        // 64 logs in MassCraft would give 256 planks, but OneStack targets ~64
        let res = mgr.craft(QuickRecipe::WoodToPlanks, 64);
        assert_eq!(res.crafted_count, 64);
        assert_eq!(res.input_consumed, 16);
        assert_eq!(res.input_leftover, 48);
    }

    #[test]
    fn test_compact_blocks_to_ingots() {
        let mut mgr = QuickCraftManager::new();
        mgr.mode = QuickCraftMode::MassCraft;

        // 5 iron blocks = 45 ingots
        let res = mgr.craft(QuickRecipe::BlockToIngots, 5);
        assert_eq!(res.crafted_count, 45);
        assert_eq!(res.input_consumed, 5);
        assert_eq!(res.input_leftover, 0);
    }
}
