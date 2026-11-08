// Copyright (c) 2026 VkYakee
// VkYakee VoxelMap x seedcrackerX
//! Entity radar scanning and classification for VoxelMap in Rust.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityCategory {
    Player,
    Monster,
    Animal,
    Neutral,
}

impl EntityCategory {
    pub fn color(self) -> u32 {
        match self {
            Self::Player => 0xFF55_FFFF,  // Aqua / Cyan
            Self::Monster => 0xFFFF_5555, // Red
            Self::Animal => 0xFF55_FF55,  // Green
            Self::Neutral => 0xFFFF_AA00, // Gold / Yellow
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Self::Player => "●",
            Self::Monster => "▲",
            Self::Animal => "◆",
            Self::Neutral => "■",
        }
    }
}

#[derive(Clone, Debug)]
pub struct RadarBlip {
    pub pos: [f64; 3],
    pub category: EntityCategory,
    pub name: String,
    pub y_diff: i32,
    pub color: u32,
}

impl RadarBlip {
    pub fn new(pos: [f64; 3], player_y: f64, category: EntityCategory, name: impl Into<String>) -> Self {
        let y_diff = (pos[1] - player_y).round() as i32;
        let color = category.color();
        Self {
            pos,
            category,
            name: name.into(),
            y_diff,
            color,
        }
    }

    /// Elevation indicator string (+, -, or space)
    pub fn elevation_symbol(&self) -> &'static str {
        if self.y_diff > 3 {
            "↑"
        } else if self.y_diff < -3 {
            "↓"
        } else {
            "="
        }
    }
}

/// Classify entity by namespace identifier or type string
pub fn classify_entity_type(type_id: &str) -> EntityCategory {
    let lower = type_id.to_ascii_lowercase();
    if lower.contains("player") {
        EntityCategory::Player
    } else if lower.contains("zombie")
        || lower.contains("skeleton")
        || lower.contains("creeper")
        || lower.contains("spider")
        || lower.contains("witch")
        || lower.contains("enderman")
        || lower.contains("slime")
        || lower.contains("phantom")
        || lower.contains("warden")
        || lower.contains("pillager")
        || lower.contains("vindicator")
        || lower.contains("evoker")
        || lower.contains("ravager")
        || lower.contains("blaze")
        || lower.contains("ghast")
        || lower.contains("piglin")
        || lower.contains("wither")
        || lower.contains("dragon")
        || lower.contains("drowned")
        || lower.contains("stray")
        || lower.contains("husk")
    {
        EntityCategory::Monster
    } else if lower.contains("cow")
        || lower.contains("pig")
        || lower.contains("sheep")
        || lower.contains("chicken")
        || lower.contains("horse")
        || lower.contains("villager")
        || lower.contains("cat")
        || lower.contains("dog")
        || lower.contains("wolf")
        || lower.contains("rabbit")
        || lower.contains("fox")
        || lower.contains("squid")
        || lower.contains("fish")
        || lower.contains("bee")
        || lower.contains("bat")
        || lower.contains("allay")
        || lower.contains("armadillo")
        || lower.contains("sniffer")
    {
        EntityCategory::Animal
    } else {
        EntityCategory::Neutral
    }
}
