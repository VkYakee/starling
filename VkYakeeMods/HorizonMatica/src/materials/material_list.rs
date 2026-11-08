use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::schematic::litematic::LitematicaSchematic;
use crate::schematic::palette::BlockState;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialEntry {
    pub item_name: String,
    pub count: usize,
    pub stack_size: usize,
    pub stacks: usize,
    pub remainder: usize,
    pub shulkers: f64,
    pub chests: f64,
}

impl MaterialEntry {
    pub fn new(item_name: impl Into<String>, count: usize, stack_size: usize) -> Self {
        let item_name = item_name.into();
        let stack_size = stack_size.max(1);
        let stacks = count / stack_size;
        let remainder = count % stack_size;
        let shulkers = count as f64 / (stack_size * 27) as f64;
        let chests = count as f64 / (stack_size * 54) as f64;

        Self {
            item_name,
            count,
            stack_size,
            stacks,
            remainder,
            shulkers,
            chests,
        }
    }

    pub fn formatted_stacks(&self) -> String {
        if self.stacks == 0 {
            format!("{}", self.remainder)
        } else if self.remainder == 0 {
            format!("{} stacks", self.stacks)
        } else {
            format!("{} stacks + {}", self.stacks, self.remainder)
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MaterialList {
    pub entries: Vec<MaterialEntry>,
    pub total_items: usize,
}

impl MaterialList {
    pub fn from_schematic(schematic: &LitematicaSchematic) -> Self {
        let mut raw_counts: HashMap<String, usize> = HashMap::new();

        for region in schematic.regions.values() {
            for (state, count) in region.container.count_blocks() {
                let item_id = Self::block_state_to_item(&state);
                *raw_counts.entry(item_id).or_insert(0) += count;
            }
        }

        let mut entries: Vec<MaterialEntry> = raw_counts
            .into_iter()
            .map(|(item, count)| {
                let stack_size = Self::get_stack_size_for_item(&item);
                MaterialEntry::new(item, count, stack_size)
            })
            .collect();

        entries.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.item_name.cmp(&b.item_name)));

        let total_items = entries.iter().map(|e| e.count).sum();

        Self { entries, total_items }
    }

    pub fn block_state_to_item(state: &BlockState) -> String {
        let base_name = state.name.strip_prefix("minecraft:").unwrap_or(&state.name);

        match base_name {
            "wall_torch" => "minecraft:torch".to_string(),
            "soul_wall_torch" => "minecraft:soul_torch".to_string(),
            "redstone_wall_torch" => "minecraft:redstone_torch".to_string(),
            s if s.ends_with("_wall_sign") => format!("minecraft:{}", s.replace("_wall_sign", "_sign")),
            s if s.ends_with("_wall_fan") => format!("minecraft:{}", s.replace("_wall_fan", "_fan")),
            s if s.ends_with("_wall_hanging_sign") => format!("minecraft:{}", s.replace("_wall_hanging_sign", "_hanging_sign")),
            s if s.ends_with("_wire") => "minecraft:redstone".to_string(),
            "water" | "bubble_column" => "minecraft:water_bucket".to_string(),
            "lava" => "minecraft:lava_bucket".to_string(),
            _ => state.name.clone(),
        }
    }

    pub fn get_stack_size_for_item(item: &str) -> usize {
        if item.contains("bed") || item.contains("boat") || item.contains("ender_pearl") || item.contains("snowball") || item.contains("sign") {
            16
        } else if item.contains("bucket") || item.contains("shulker_box") || item.contains("minecart") {
            1
        } else {
            64
        }
    }
}
