use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::material_list::MaterialList;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventoryProgressEntry {
    pub item_name: String,
    pub required: usize,
    pub available: usize,
    pub missing: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventoryProgress {
    pub entries: Vec<InventoryProgressEntry>,
    pub total_required: usize,
    pub total_available: usize,
    pub total_missing: usize,
}

pub struct InventoryTracker;

impl InventoryTracker {
    pub fn calculate_progress(
        material_list: &MaterialList,
        available_inventory: &HashMap<String, usize>,
    ) -> InventoryProgress {
        let mut entries = Vec::new();
        let mut total_required = 0;
        let mut total_available = 0;
        let mut total_missing = 0;

        for mat in &material_list.entries {
            let req = mat.count;
            let avail = *available_inventory.get(&mat.item_name).unwrap_or(&0);
            let missing = req.saturating_sub(avail);

            total_required += req;
            total_available += avail.min(req);
            total_missing += missing;

            entries.push(InventoryProgressEntry {
                item_name: mat.item_name.clone(),
                required: req,
                available: avail,
                missing,
            });
        }

        InventoryProgress {
            entries,
            total_required,
            total_available,
            total_missing,
        }
    }
}
