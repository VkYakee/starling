use serde::{Deserialize, Serialize};

/// Categorization of entities for radar tracking (VoxelMap & Xaero standard).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityCategory {
    Player,
    Hostile,
    Passive,
    Neutral,
    Item,
    Other,
}

impl EntityCategory {
    /// Standard radar dot RGBA color.
    pub fn default_color(&self) -> [u8; 4] {
        match self {
            EntityCategory::Player => [80, 160, 255, 255],  // Cyan-Blue
            EntityCategory::Hostile => [255, 60, 60, 255],   // Vibrant Red
            EntityCategory::Passive => [80, 220, 80, 255],   // Vibrant Green
            EntityCategory::Neutral => [240, 240, 60, 255],  // Yellow
            EntityCategory::Item => [255, 170, 0, 255],     // Orange
            EntityCategory::Other => [180, 180, 180, 255],  // Gray
        }
    }
}

/// An entity tracked in the world for radar rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadarEntity {
    pub id: u32,
    pub name: String,
    pub category: EntityCategory,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
}

impl RadarEntity {
    pub fn new(
        id: u32,
        name: impl Into<String>,
        category: EntityCategory,
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            category,
            x,
            y,
            z,
            yaw,
        }
    }

    /// Automatically classifies an entity type name into an `EntityCategory`.
    pub fn classify(type_id: &str) -> EntityCategory {
        let clean = type_id.trim().to_ascii_lowercase();
        let name = clean.strip_prefix("minecraft:").unwrap_or(&clean);

        if name == "player" {
            EntityCategory::Player
        } else if name == "item" || name.contains("item") {
            EntityCategory::Item
        } else if is_hostile_mob(name) {
            EntityCategory::Hostile
        } else if is_neutral_mob(name) {
            EntityCategory::Neutral
        } else if is_passive_mob(name) {
            EntityCategory::Passive
        } else {
            EntityCategory::Other
        }
    }
}

fn is_hostile_mob(name: &str) -> bool {
    matches!(
        name,
        "zombie"
            | "skeleton"
            | "creeper"
            | "spider"
            | "cave_spider"
            | "witch"
            | "enderman"
            | "slime"
            | "magma_cube"
            | "blaze"
            | "ghast"
            | "wither_skeleton"
            | "wither"
            | "ender_dragon"
            | "phantom"
            | "drowned"
            | "husk"
            | "stray"
            | "pillager"
            | "ravager"
            | "vindicator"
            | "evoker"
            | "vex"
            | "warden"
            | "breeze"
            | "bogged"
            | "piglin_brute"
            | "hoglin"
            | "zoglin"
    )
}

fn is_neutral_mob(name: &str) -> bool {
    matches!(
        name,
        "iron_golem"
            | "wolf"
            | "piglin"
            | "zombified_piglin"
            | "bee"
            | "dolphin"
            | "polar_bear"
            | "llama"
            | "trader_llama"
            | "panda"
            | "goat"
    )
}

fn is_passive_mob(name: &str) -> bool {
    matches!(
        name,
        "cow"
            | "sheep"
            | "pig"
            | "chicken"
            | "horse"
            | "donkey"
            | "mule"
            | "villager"
            | "wandering_trader"
            | "cat"
            | "ocelot"
            | "parrot"
            | "bat"
            | "fox"
            | "rabbit"
            | "turtle"
            | "strider"
            | "axolotl"
            | "glow_squid"
            | "squid"
            | "allay"
            | "frog"
            | "tadpole"
            | "camel"
            | "sniffer"
            | "armadillo"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_classification() {
        assert_eq!(RadarEntity::classify("minecraft:creeper"), EntityCategory::Hostile);
        assert_eq!(RadarEntity::classify("minecraft:cow"), EntityCategory::Passive);
        assert_eq!(RadarEntity::classify("minecraft:player"), EntityCategory::Player);
        assert_eq!(RadarEntity::classify("minecraft:wolf"), EntityCategory::Neutral);
        assert_eq!(RadarEntity::classify("minecraft:item"), EntityCategory::Item);
    }
}
