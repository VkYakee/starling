//! VlakeeUI: Native Bedrock-inspired UI shortcuts, SmartF3, InvHUD, and quick toggles
//! ported from Déesse UI for Minecraft Starling (Horizon Android Rust).
//!
//! Important Architecture Constraint:
//! The main start/title screen of Minecraft is preserved 100% in Java Edition style.
//! All VlakeeUI enhancements, quick settings, pause menu overlays, container shortcuts,
//! coordinate bookmarks and temporary notes operate strictly INSIDE the game world.

pub mod calculator;
pub mod chunk_viewer;
pub mod debug_f3;
pub mod hud_renderer;
pub mod in_game_options;
pub mod inv_hud;
pub mod quick_craft;
pub mod quick_toggles;
pub mod smart_f3_more;

pub use calculator::InGameCalculator;
pub use chunk_viewer::{ChunkViewer, ChunkViewerMode};
pub use debug_f3::{Direction, F3Mode, SmartF3};
pub use hud_renderer::{VlakeeHudMenu, VlakeeMenuPage};
pub use in_game_options::{
    CameraPerspective, CoordWaypoint, FovPreset, InGameOptions, QuickContainerMode,
    QuickGamemode, QuickTime, QuickWeather, TemporaryNotes,
};
pub use inv_hud::{HeldItemInfo, InvHud, ItemDurability};
pub use quick_craft::{QuickCraftManager, QuickCraftMode, QuickCraftResult, QuickRecipe};
pub use quick_toggles::QuickToggles;
pub use smart_f3_more::SmartF3More;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_f3_calculations() {
        let mut f3 = SmartF3::new();
        f3.update_pos(100.5, 64.0, -200.5, 180.0, 0.0);
        f3.set_time(36000); // Day 2, 12000 ticks

        assert_eq!(f3.block_pos(), (100, 64, -201));
        assert_eq!(f3.direction(), Direction::North);
        assert_eq!(f3.day_count, 2);

        // Nether portal target: in Overworld, (100/8, 64, -201/8) = (12, 64, -25)
        let (nx, ny, nz, _) = f3.portal_target_pos();
        assert_eq!((nx, ny, nz), (12, 64, -25));

        // Test HUD generation
        f3.mode = F3Mode::Compact;
        let lines = f3.generate_hud_lines();
        assert!(!lines.is_empty());
        assert!(lines.iter().any(|l| l.contains("SmartF3")));
    }

    #[test]
    fn test_inv_hud_and_durability() {
        let mut inv = InvHud::new();
        inv.update_inventory_counts(9, 27); // 36 filled
        assert!(inv.is_inventory_full());
        assert_eq!(inv.empty_slots(), 0);

        let lines = inv.generate_hud_lines();
        assert!(lines.iter().any(|l| l.contains("INVENTARIO LLENO")));

        // Test tool durability
        let tool = ItemDurability::new("Netherite Pickaxe", 100, 2031);
        assert!(tool.is_low());
    }

    #[test]
    fn test_chunk_viewer_boxes() {
        let mut cv = ChunkViewer::new();
        cv.update_pos(24, 64, 40); // Chunk (1, 2), rel (8, 8)
        assert_eq!(cv.chunk_x, 1);
        assert_eq!(cv.chunk_z, 2);
        assert!(cv.is_at_middle());

        cv.mode = ChunkViewerMode::FullGrid;
        let boxes = cv.render_bounding_boxes();
        assert!(!boxes.is_empty(), "Must produce chunk 3D bounding boxes");
    }

    #[test]
    fn test_in_game_calculator() {
        let res = InGameCalculator::evaluate("128 * 4 + 64").expect("Math eval");
        assert_eq!(res, 576.0);

        let stacks = InGameCalculator::calculate_stacks(576);
        assert!(stacks.contains("9 stacks"));

        let (nx, nz, _) = InGameCalculator::calculate_portal(800, -1600, true);
        assert_eq!((nx, nz), (100, -200));
    }

    #[test]
    fn test_in_game_options_integration() {
        let mut opts = InGameOptions::new();
        assert_eq!(opts.perspective, CameraPerspective::FirstPerson);
        opts.cycle_perspective();
        assert_eq!(opts.perspective, CameraPerspective::ThirdPersonBack);

        assert_eq!(opts.gamemode, QuickGamemode::Survival);
        opts.cycle_gamemode();
        assert_eq!(opts.gamemode, QuickGamemode::Creative);

        let wp_id = opts.add_waypoint("Spawn", 0, 70, 0, "overworld");
        assert_eq!(wp_id, 1);
        assert_eq!(opts.waypoints.len(), 1);

        opts.notes.add("Terminar portal");
        assert_eq!(opts.notes.entries.len(), 1);

        let lines = opts.generate_in_game_menu_lines();
        assert!(lines.iter().any(|l| l.contains("Cámara")));
        assert!(lines.iter().any(|l| l.contains("Modo de Juego")));
    }

    #[test]
    fn test_smart_f3_more_combos() {
        let mut f3_more = SmartF3More::new();
        assert!(!f3_more.entity_hitboxes);
        f3_more.toggle_entity_hitboxes();
        assert!(f3_more.entity_hitboxes);

        assert!(!f3_more.chunk_boundaries);
        f3_more.toggle_chunk_boundaries();
        assert!(f3_more.chunk_boundaries);

        let tp = f3_more.generate_f3_c_tp(50.0, 70.0, -80.0, 180.0, 0.0, "minecraft:overworld");
        assert!(tp.contains("/execute in minecraft:overworld run tp @s 50.00 70.00 -80.00 180.0 0.0"));
    }

    #[test]
    fn test_quick_craft_shortcuts() {
        let mut qc = QuickCraftManager::new();
        qc.mode = QuickCraftMode::MassCraft;

        let res = qc.craft(QuickRecipe::WoodToPlanks, 8); // 8 logs -> 32 planks
        assert_eq!(res.crafted_count, 32);
        assert_eq!(res.input_consumed, 8);
    }
}
