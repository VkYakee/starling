// Copyright (c) 2026 HorizonMatica Authors / Starling Android
//! Material list aggregator: counts required blocks in schematic and compares
//! against player inventory to report available and missing quantities.

use std::collections::BTreeMap;
use super::format::LoadedSchematic;
use super::placement::{LayerMode, SchematicPlacement};

#[derive(Clone, Debug)]
pub struct MaterialEntry {
    pub item_name: String,
    pub display_name: String,
    pub required: usize,
    pub available: usize,
    pub missing: usize,
}

#[derive(Clone, Debug, Default)]
pub struct MaterialReport {
    pub entries: Vec<MaterialEntry>,
    pub total_required: usize,
    pub total_available: usize,
    pub total_missing: usize,
}

impl MaterialReport {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn generate<F>(
        schematic: &LoadedSchematic,
        placement: &SchematicPlacement,
        mut count_in_inventory: F,
    ) -> Self
    where
        F: FnMut(&str) -> usize,
    {
        let mut required_counts: BTreeMap<String, usize> = BTreeMap::new();

        for r in &schematic.regions {
            if !placement.is_subregion_enabled(&r.name) {
                continue;
            }

            for ly in 0..r.container.size_y {
                if !placement.layer_mode.is_layer_visible(ly) {
                    continue;
                }

                for lz in 0..r.container.size_z {
                    for lx in 0..r.container.size_x {
                        let Some(st) = r.container.get(lx, ly, lz) else {
                            continue;
                        };
                        if st.is_air() {
                            continue;
                        }

                        let item_key = block_to_item_name(&st.name);
                        *required_counts.entry(item_key).or_insert(0) += 1;
                    }
                }
            }
        }

        let mut entries = Vec::new();
        let mut total_req = 0;
        let mut total_avail = 0;
        let mut total_miss = 0;

        for (item_name, req) in required_counts {
            let avail = count_in_inventory(&item_name);
            let miss = req.saturating_sub(avail);

            total_req += req;
            total_avail += avail.min(req);
            total_miss += miss;

            let display_name = format_item_name(&item_name);

            entries.push(MaterialEntry {
                item_name,
                display_name,
                required: req,
                available: avail,
                missing: miss,
            });
        }

        // Sort: missing items first, then by required count descending
        entries.sort_by(|a, b| {
            b.missing
                .cmp(&a.missing)
                .then_with(|| b.required.cmp(&a.required))
                .then_with(|| a.item_name.cmp(&b.item_name))
        });

        Self {
            entries,
            total_required: total_req,
            total_available: total_avail,
            total_missing: total_miss,
        }
    }
}

fn block_to_item_name(block: &str) -> String {
    let raw = block.strip_prefix("minecraft:").unwrap_or(block);
    let mapped = match raw {
        "wall_torch" => "torch",
        "redstone_wall_torch" => "redstone_torch",
        "soul_wall_torch" => "soul_torch",
        "tripwire" => "string",
        "carrots" => "carrot",
        "potatoes" => "potato",
        "beetroots" => "beetroot",
        "cocoa" => "cocoa_beans",
        "sweet_berry_bush" => "sweet_berries",
        "melon_stem" | "attached_melon_stem" => "melon_seeds",
        "pumpkin_stem" | "attached_pumpkin_stem" => "pumpkin_seeds",
        "water" | "bubble_column" => "water_bucket",
        "lava" => "lava_bucket",
        other => other,
    };
    format!("minecraft:{mapped}")
}

fn format_item_name(item: &str) -> String {
    let clean = item.strip_prefix("minecraft:").unwrap_or(item);
    clean
        .split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
