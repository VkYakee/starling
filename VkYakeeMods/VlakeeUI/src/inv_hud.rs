//! Inventory HUD, Durability Viewer and Slot Counter ported from Déesse UI for Starling.
//! Alerts the player when inventory is full and tracks tool/armor durability in real time.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemDurability {
    pub name: String,
    pub current: u32,
    pub max: u32,
}

impl ItemDurability {
    pub fn new(name: impl Into<String>, current: u32, max: u32) -> Self {
        Self {
            name: name.into(),
            current,
            max,
        }
    }

    pub fn percentage(&self) -> f32 {
        if self.max == 0 {
            100.0
        } else {
            (self.current as f32 / self.max as f32 * 100.0).clamp(0.0, 100.0)
        }
    }

    pub fn is_low(&self) -> bool {
        self.percentage() < 15.0 && self.max > 0
    }

    pub fn formatted(&self) -> String {
        format!("{}: {}/{} ({:.0}%)", self.name, self.current, self.max, self.percentage())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeldItemInfo {
    pub name: String,
    pub total_count: usize,
    pub slots_occupied: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvHud {
    pub enabled: bool,
    pub show_durability: bool,
    pub show_item_info: bool,
    pub total_slots: usize,
    pub filled_slots: usize,
    pub hotbar_filled: usize,
    pub main_inv_filled: usize,
    pub main_hand_item: Option<ItemDurability>,
    pub offhand_item: Option<ItemDurability>,
    pub helmet: Option<ItemDurability>,
    pub chestplate: Option<ItemDurability>,
    pub leggings: Option<ItemDurability>,
    pub boots: Option<ItemDurability>,
    pub held_item_info: Option<HeldItemInfo>,
}

impl Default for InvHud {
    fn default() -> Self {
        Self {
            enabled: true,
            show_durability: true,
            show_item_info: true,
            total_slots: 36,
            filled_slots: 14,
            hotbar_filled: 6,
            main_inv_filled: 8,
            main_hand_item: Some(ItemDurability::new("Diamond Pickaxe", 1340, 1561)),
            offhand_item: Some(ItemDurability::new("Shield", 300, 336)),
            helmet: Some(ItemDurability::new("Diamond Helmet", 320, 363)),
            chestplate: Some(ItemDurability::new("Diamond Chestplate", 490, 528)),
            leggings: Some(ItemDurability::new("Diamond Leggings", 450, 495)),
            boots: Some(ItemDurability::new("Diamond Boots", 380, 429)),
            held_item_info: Some(HeldItemInfo {
                name: "Cobblestone".to_string(),
                total_count: 128,
                slots_occupied: 2,
            }),
        }
    }
}

impl InvHud {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn empty_slots(&self) -> usize {
        self.total_slots.saturating_sub(self.filled_slots)
    }

    pub fn is_inventory_full(&self) -> bool {
        self.empty_slots() == 0
    }

    pub fn is_inventory_almost_full(&self) -> bool {
        self.empty_slots() <= 2
    }

    pub fn update_inventory_counts(&mut self, hotbar_filled: usize, main_inv_filled: usize) {
        self.hotbar_filled = hotbar_filled.min(9);
        self.main_inv_filled = main_inv_filled.min(27);
        self.filled_slots = self.hotbar_filled + self.main_inv_filled;
    }

    pub fn set_main_hand(&mut self, name: impl Into<String>, current: u32, max: u32) {
        self.main_hand_item = Some(ItemDurability::new(name, current, max));
    }

    pub fn set_held_info(&mut self, name: impl Into<String>, total_count: usize, slots_occupied: usize) {
        self.held_item_info = Some(HeldItemInfo {
            name: name.into(),
            total_count,
            slots_occupied,
        });
    }

    pub fn generate_hud_lines(&self) -> Vec<String> {
        if !self.enabled {
            return Vec::new();
        }

        let mut lines = Vec::new();

        // Inventory Capacity Status
        let empty = self.empty_slots();
        let status_str = if self.is_inventory_full() {
            "§c¡INVENTARIO LLENO! (0 espacios)"
        } else if self.is_inventory_almost_full() {
            "§e⚠️ Casi lleno"
        } else {
            "§aEspacio disponible"
        };

        lines.push(format!(
            "🎒 Inventario: {}/36 ocupados ({} vacíos) - {}",
            self.filled_slots, empty, status_str
        ));
        lines.push(format!("   Hotbar: {}/9 | Mochila: {}/27", self.hotbar_filled, self.main_inv_filled));

        // Held Item Info
        if self.show_item_info {
            if let Some(ref info) = self.held_item_info {
                lines.push(format!("📦 Mano: {} ({} items en {} slots)", info.name, info.total_count, info.slots_occupied));
            }
        }

        // Durability Tracking
        if self.show_durability {
            if let Some(ref tool) = self.main_hand_item {
                let warn = if tool.is_low() { " §c[¡DURABILIDAD CRÍTICA!]" } else { "" };
                lines.push(format!("⚔️ Herramienta: {}{}", tool.formatted(), warn));
            }

            if let Some(ref shield) = self.offhand_item {
                lines.push(format!("🛡️ Offhand: {}", shield.formatted()));
            }

            // Compact Armor Durability overview
            let armor = [&self.helmet, &self.chestplate, &self.leggings, &self.boots];
            let armor_labels = ["H", "C", "L", "B"];
            let mut armor_parts = Vec::new();

            for (opt, label) in armor.iter().zip(armor_labels.iter()) {
                if let Some(piece) = opt {
                    armor_parts.push(format!("{}:{:.0}%", label, piece.percentage()));
                }
            }

            if !armor_parts.is_empty() {
                lines.push(format!("🛡️ Armadura: [{}]", armor_parts.join(" | ")));
            }
        }

        lines
    }
}
