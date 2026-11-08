use vkyakee_mods::{
    get_bounding_boxes, get_hud_lines, get_mod_list, get_settings_buttons, on_biome_found,
    on_command, on_remote_seed, on_settings_click, on_structure_found,
    storage::{StorageManager, WorldSessionData},
    HorizonMaticaMod, VkMod, VlakeeUIMod,
};
use horizonmatica::schematic::palette::BlockState;
use seedcracker::structures::TreasureDatabase;

#[test]
fn test_vkyakee_mods_registration() {
    let mods = get_mod_list();
    assert!(!mods.is_empty(), "Mod list should contain at least SeedCracker");
    let sc = mods.iter().find(|m| m.id == "seedcracker");
    assert!(sc.is_some(), "SeedCracker must be registered in VkYakeeMods");
    println!("Registered mod: {:?}", sc.unwrap().name);
}

#[test]
fn test_seedcracker_command_interception() {
    let mut ran = false;
    on_command("seedcracker gui", &mut ran);
    assert!(ran, "Command /seedcracker gui should be handled by SeedCrackerMod");

    let mut ran2 = false;
    on_command("seedcracker gui", &mut ran2);
    assert!(ran2, "Second /seedcracker gui command should also be handled");

    let lines = get_hud_lines();
    assert!(!lines.is_empty(), "HUD lines should not be empty when HUD is active");
    println!("HUD output: {:?}", lines);
}

#[test]
fn test_treasure_database_embedded() {
    let db = TreasureDatabase::load_embedded();
    assert_eq!(db.formations.len(), 10, "Should load all 10 buried treasure formations");
    let total_seeds: usize = db.formations.iter().map(|f| f.seeds.len()).sum();
    assert!(total_seeds > 1000, "Should contain precomputed structure seeds");
    println!("Loaded {} buried treasure formations with {} total seeds", db.formations.len(), total_seeds);
}

#[test]
fn test_seedcracker_hashed_seed_flow() {
    let test_hash = 0x123456789ABCDEF0u64;
    on_remote_seed(test_hash);
    let lines = get_hud_lines();
    let found = lines.iter().any(|l| l.contains("Hashed Seed") || l.contains("123456789ABCDEF0"));
    assert!(found, "HUD should show the updated hashed seed");
}

#[test]
fn test_biome_and_structure_events_and_persistence() {
    // Dispatch discovery events
    on_biome_found(100, 200, 2, "Desert");
    on_structure_found("Desert_Pyramid", 6, 12);

    let lines = get_hud_lines();
    let has_biome = lines.iter().any(|l| l.contains("Desert"));
    let has_struct = lines.iter().any(|l| l.contains("Desert_Pyramid"));
    assert!(has_biome, "HUD toasts must show the discovered biome event");
    assert!(has_struct, "HUD toasts must show the discovered structure event");

    // Test storage manager save and load
    let storage = StorageManager::new();
    let mut data = WorldSessionData {
        world_id: "test_verification_world".to_string(),
        dimension: "minecraft:overworld".to_string(),
        hashed_seed: Some(0xDEADBEEF12345678),
        cracked_seed: Some(-123456789),
        ..Default::default()
    };
    data.structures.push(vkyakee_mods::storage::DiscoveredStructure {
        structure_type: "Desert_Pyramid".to_string(),
        chunk_x: 6,
        chunk_z: 12,
        block_x: 96,
        block_z: 192,
    });

    let save_res = storage.save(&data);
    assert!(save_res.is_ok(), "Storage manager must save world session successfully");

    let loaded = storage.load_or_create("test_verification_world");
    assert_eq!(loaded.world_id, "test_verification_world");
    assert_eq!(loaded.hashed_seed, Some(0xDEADBEEF12345678));
    assert_eq!(loaded.cracked_seed, Some(-123456789));
    assert_eq!(loaded.structures.len(), 1);

    // Test map CSV export
    let export_res = storage.export_map_csv(&loaded);
    assert!(export_res.is_ok(), "Map CSV export must succeed");
    println!("Exported map file to {:?}", export_res.unwrap());
}

#[test]
fn test_verifier_pruning_and_confirmation() {
    use vkyakee_mods::verifier::{SeedVerifier, VerifierState};

    let mut verifier = SeedVerifier::new();
    verifier.update_player_pos(0, 0);

    // 1. Single seed exact verification
    let exact_seed = 123456789012345u64;
    verifier.set_candidates(vec![exact_seed]);

    let found_target = match &verifier.state {
        VerifierState::VerifyingExact { seed, target, confirmed } => {
            assert_eq!(*seed, exact_seed);
            assert!(!confirmed);
            println!("Generated confirmation waypoint: {} @ ({}, {}) [{}m]", 
                target.structure_name, target.block_x, target.block_z, target.distance as i32);
            Some((target.structure_name.clone(), target.chunk_x, target.chunk_z))
        }
        _ => panic!("Expected VerifyingExact state"),
    };

    if let Some((name, cx, cz)) = found_target {
        verifier.on_structure_found(&name, cx, cz);
    }

    // Must be confirmed now!
    match &verifier.state {
        VerifierState::Confirmed { seed, .. } => {
            assert_eq!(*seed, exact_seed);
            println!("Successfully confirmed exact seed on terrain!");
        }
        _ => panic!("Expected Confirmed state after finding structure"),
    }

    // 2. Candidate pruning mode with multiple seeds
    let candidates: Vec<u64> = (1000..1050).collect();
    let mut pruning_verifier = SeedVerifier::new();
    pruning_verifier.update_player_pos(100, -100);
    pruning_verifier.set_candidates(candidates);

    let initial_count = pruning_verifier.candidates.len();
    assert_eq!(initial_count, 50);

    let hud = pruning_verifier.hud_lines();
    assert!(hud.iter().any(|l| l.contains("50 semillas")));
    assert!(hud.iter().any(|l| l.contains("Descarte")));
    println!("Pruning HUD lines: {:?}", hud);
}

#[test]
fn test_horizonmatica_registration_and_hub() {
    let mods = get_mod_list();
    let hm = mods.iter().find(|m| m.id == "horizonmatica");
    assert!(hm.is_some(), "HorizonMatica must be registered in VkYakeeMods");
    let meta = hm.unwrap();
    assert_eq!(meta.name, "HorizonMatica");
    assert!(meta.author.contains("VkYakee"));
    println!("Registered mod: {} by {}", meta.name, meta.author);
}

#[test]
fn test_horizonmatica_commands_and_hud() {
    let mut ran = false;
    on_command("hm list", &mut ran);
    assert!(ran, "Command /hm list should be handled by HorizonMaticaMod");

    let mut ran_rot = false;
    on_command("hm rot cw", &mut ran_rot);
    assert!(ran_rot, "Command /hm rot cw should be handled");

    let mut ran_pos = false;
    on_command("hm pos 15 70 25", &mut ran_pos);
    assert!(ran_pos, "Command /hm pos should be handled");

    let mut ran_mat = false;
    on_command("hm mat", &mut ran_mat);
    assert!(ran_mat, "Command /hm mat should be handled");

    let lines = get_hud_lines();
    let hm_hud_present = lines.iter().any(|l| l.contains("HorizonMatica"));
    assert!(hm_hud_present, "HUD must contain HorizonMatica panel");
    println!("HUD with HorizonMatica: {:?}", lines);
}

#[test]
fn test_horizonmatica_placement_and_bounding_boxes() {
    // Check that bounding boxes are populated from placement and selection
    let boxes = get_bounding_boxes();
    assert!(!boxes.is_empty(), "Should generate at least one 3D bounding box for active schematic or selection");
    for b in &boxes {
        assert!(b[3] >= b[0], "max_x >= min_x");
        assert!(b[4] >= b[1], "max_y >= min_y");
        assert!(b[5] >= b[2], "max_z >= min_z");
        // Color alpha check
        assert!(b[9] > 0.0, "Bounding box must have non-zero alpha");
    }
    println!("Rendered {} bounding boxes for world renderer", boxes.len());
}

#[test]
fn test_horizonmatica_touch_settings_buttons() {
    let buttons = get_settings_buttons("horizonmatica");
    assert!(!buttons.is_empty(), "HorizonMatica must provide touch UI buttons for Android");
    assert!(buttons.iter().any(|b| b.contains("Rotar")), "Must have Rotate button");
    assert!(buttons.iter().any(|b| b.contains("Espejo")), "Must have Mirror button");
    assert!(buttons.iter().any(|b| b.contains("EasyPlace")), "Must have EasyPlace button");

    // Click rotate button
    on_settings_click("horizonmatica", 2);
    // Click layer up
    on_settings_click("horizonmatica", 4);
    // Click easy place toggle
    on_settings_click("horizonmatica", 7);
}

#[test]
fn test_horizonmatica_storage_and_export() {
    let storage = StorageManager::new();

    // Verify directory paths
    assert!(storage.seedcracker_worlds_dir().ends_with("SeedCracker/worlds") || storage.seedcracker_worlds_dir().to_string_lossy().contains("SeedCracker"));
    assert!(storage.schematics_dir().ends_with("HorizonMatica/schematics") || storage.schematics_dir().to_string_lossy().contains("HorizonMatica"));
    assert!(storage.exports_dir().ends_with("HorizonMatica/exports") || storage.exports_dir().to_string_lossy().contains("HorizonMatica"));

    // Check that sample schematic was created
    let schematics = storage.list_schematics();
    assert!(!schematics.is_empty(), "StorageManager must detect at least one schematic in storage");
    println!("Discovered {} schematics in mobile storage: {:?}", schematics.len(), schematics);

    // Load schematic
    let first = &schematics[0];
    let loaded = storage.load_schematic(&first.to_string_lossy());
    assert!(loaded.is_ok(), "Must successfully load schematic from storage");
    let mut schem = loaded.unwrap();

    // Test exporting to exports/
    let exported = storage.export_schematic("test_export_shelter", &mut schem);
    assert!(exported.is_ok(), "Must export schematic successfully");
    let export_path = exported.unwrap();
    assert!(export_path.exists(), "Exported file must exist on disk");

    // Reload exported file to verify integrity
    let reloaded = storage.load_schematic(&export_path.to_string_lossy());
    assert!(reloaded.is_ok(), "Reloading exported schematic must succeed");
    assert_eq!(reloaded.unwrap().metadata.name, schem.metadata.name);
}

#[test]
fn test_horizonmatica_mini_renderer_preview() {
    use horizonmatica::schematic::mini_render::SchematicMiniRenderer;
    use horizonmatica::util::pos::Rotation;

    let storage = StorageManager::new();
    let schematics = storage.list_schematics();
    assert!(!schematics.is_empty());

    let schem = storage.load_schematic(&schematics[0].to_string_lossy()).expect("Load schem");

    // Test 3D isometric projection
    let preview_3d = SchematicMiniRenderer::render_preview(&schem, None, Rotation::None);
    assert!(!preview_3d.is_empty(), "3D preview must produce lines");
    assert!(preview_3d.iter().any(|l| l.contains("Isometric")), "Must contain isometric header");
    println!("--- 3D Isometric Mini-Render Preview ---");
    for l in &preview_3d {
        println!("{}", l);
    }

    // Test 2D layer slice
    let preview_layer = SchematicMiniRenderer::render_preview(&schem, Some(1), Rotation::Clockwise90);
    assert!(!preview_layer.is_empty(), "Layer preview must produce lines");
    assert!(preview_layer.iter().any(|l| l.contains("Capa 2D")), "Must contain layer header");
    println!("--- 2D Layer Slice Preview ---");
    for l in &preview_layer {
        println!("{}", l);
    }
}

#[test]
fn test_horizonmatica_import_schematic() {
    let storage = StorageManager::new();
    let schematics = storage.list_schematics();
    assert!(!schematics.is_empty());

    // Create a temporary source file simulating a downloaded file in /tmp or local dir
    let temp_source = std::env::temp_dir().join("downloaded_castle.litematic");
    std::fs::copy(&schematics[0], &temp_source).expect("copy to temp");

    let mut ran_import = false;
    let import_cmd = format!("hm import {}", temp_source.display());
    on_command(&import_cmd, &mut ran_import);
    assert!(ran_import, "Command /hm import must be handled");

    // Verify it was copied to schematics_dir
    let target_dest = storage.schematics_dir().join("downloaded_castle.litematic");
    assert!(target_dest.exists(), "Imported file must exist in HorizonMatica/schematics/");

    let _ = std::fs::remove_file(temp_source);
}

#[test]
fn test_horizonmatica_tab_switching_in_hud() {
    let mut ran_tab_hm = false;
    on_command("vk tab hm", &mut ran_tab_hm);
    assert!(ran_tab_hm, "/vk tab hm must switch active HUD tab");

    let lines_hm = get_hud_lines();
    assert!(lines_hm.iter().any(|l| l.contains("HorizonMatica")), "Must show HorizonMatica");
    assert!(!lines_hm.iter().any(|l| l.contains("SeedCracker v1.0.0")), "Should not show SeedCracker header");

    let mut ran_tab_all = false;
    on_command("vk tab all", &mut ran_tab_all);
    assert!(ran_tab_all, "/vk tab all must show all mods");

    let lines_all = get_hud_lines();
    assert!(lines_all.iter().any(|l| l.contains("HorizonMatica")));
    assert!(lines_all.iter().any(|l| l.contains("SeedCracker")));
}

#[test]
fn test_horizonmatica_interactive_workflow_and_tools_hud() {
    use vkyakee_mods::on_player_pos;

    // Simulate player at coordinates [50, 70, -30]
    on_player_pos(50, 70, -30);

    // 1. Start Hologram (/hm start)
    let mut ran_start = false;
    on_command("hm start", &mut ran_start);
    assert!(ran_start, "/hm start should place hologram and switch to 1-button HUD");

    // HUD lines should now be the compact 1-button widget [🛠️ Herramientas]
    let mini_lines = get_hud_lines();
    assert!(mini_lines.iter().any(|l| l.contains("Herramientas")), "Must display compact tools widget");

    // Bounding boxes should be visible with current opacity
    let boxes = get_bounding_boxes();
    assert!(!boxes.is_empty(), "Hologram bounding boxes must be rendered");

    // 2. Open Tools Drawer (/hm tools)
    let mut ran_tools = false;
    on_command("hm tools", &mut ran_tools);
    assert!(ran_tools, "/hm tools should open tools drawer");

    let tools_lines = get_hud_lines();
    assert!(tools_lines.iter().any(|l| l.contains("Creación de Nuevo Litematic")), "Must show creation tools");
    assert!(tools_lines.iter().any(|l| l.contains("Subir Y+1")), "Must show nudge tools");

    // 3. Nudge Up/Down
    let mut ran_up = false;
    on_command("hm up", &mut ran_up);
    assert!(ran_up);

    // 4. Change Opacity (/hm opacity 50)
    let mut ran_op = false;
    on_command("hm opacity 50", &mut ran_op);
    assert!(ran_op);

    let boxes_op = get_bounding_boxes();
    assert!(!boxes_op.is_empty());
    assert!((boxes_op[0][9] - 0.50).abs() < 0.05, "Opacity alpha must match 0.50");

    // 5. Creation Tools: Pos1, Pos2, and Save New Litematic
    on_player_pos(10, 64, 10);
    let mut ran_pos1 = false;
    on_command("hm pos1", &mut ran_pos1);
    assert!(ran_pos1);

    on_player_pos(20, 74, 20);
    let mut ran_pos2 = false;
    on_command("hm pos2", &mut ran_pos2);
    assert!(ran_pos2);

    let mut ran_save = false;
    on_command("hm save mi_nueva_granja", &mut ran_save);
    assert!(ran_save);

    let storage = StorageManager::new();
    let saved_path = storage.schematics_dir().join("mi_nueva_granja.litematic");
    assert!(saved_path.exists(), "New litematic must be saved in schematics/");

    // Clean up created test file
    let _ = std::fs::remove_file(saved_path);
}

#[test]
fn test_horizonmatica_verifier_parity() {
    let mut hm = HorizonMaticaMod::new();
    assert!(hm.load_schematic("sample_shelter").is_ok());

    // 1. Verify against all air (all blocks missing)
    let summary_air = hm.verify_schematic();
    assert!(summary_air.is_some());
    let s = summary_air.unwrap();
    assert!(s.total_schematic_blocks > 0, "Schematic must contain blocks");
    assert_eq!(s.missing_count, s.total_schematic_blocks, "All blocks should be missing if world is air");
    assert_eq!(s.correct_count, 0);

    // 2. Verify with matching blocks lookup
    let summary_perfect = hm.verify_schematic_with(|_pos| {
        BlockState::new("minecraft:oak_planks")
    });
    assert!(summary_perfect.is_some());
    let sp = summary_perfect.unwrap();
    assert!(sp.correct_count > 0, "Oak plank blocks in shelter must match correctly");

    // 3. Command interface /hm verify
    let mut ran_v = false;
    on_command("hm v", &mut ran_v);
    assert!(ran_v);
}

#[test]
fn test_horizonmatica_material_inventory_and_export() {
    let mut hm = HorizonMaticaMod::new();
    assert!(hm.load_schematic("sample_shelter").is_ok());

    // 1. Check without inventory (all missing)
    let progress_empty = hm.check_materials_progress();
    assert!(progress_empty.is_some());
    let p_empty = progress_empty.unwrap();
    assert_eq!(p_empty.total_available, 0);
    assert_eq!(p_empty.total_missing, p_empty.total_required);

    // 2. Provide partial inventory
    hm.set_inventory_item("minecraft:oak_planks", 20);
    let progress_partial = hm.check_materials_progress();
    assert!(progress_partial.is_some());
    let p_partial = progress_partial.unwrap();
    assert_eq!(p_partial.total_available, 20);
    assert_eq!(p_partial.total_missing, p_partial.total_required - 20);

    // 3. Export CSV and JSON to exports/
    let csv_path = hm.export_materials_csv().expect("CSV export must succeed");
    assert!(csv_path.exists(), "Exported CSV must exist");
    let csv_content = std::fs::read_to_string(&csv_path).unwrap();
    assert!(csv_content.contains("oak_planks") || csv_content.contains("Item"), "CSV must list schematic materials");

    let json_path = hm.export_materials_json().expect("JSON export must succeed");
    assert!(json_path.exists(), "Exported JSON must exist");
    let json_content = std::fs::read_to_string(&json_path).unwrap();
    assert!(json_content.contains("total_items"), "JSON must be valid material list format");

    // Clean up
    let _ = std::fs::remove_file(csv_path);
    let _ = std::fs::remove_file(json_path);
}

#[test]
fn test_horizonmatica_placement_lock_and_axis() {
    let mut hm = HorizonMaticaMod::new();
    assert!(hm.load_schematic("sample_shelter").is_ok());

    // 1. Normal nudge works
    hm.set_origin(100, 64, 100);
    hm.nudge_origin(1, 0, 0);

    // 2. Lock placement
    hm.toggle_placement_lock();
    // Nudge should be rejected
    hm.nudge_origin(5, 5, 5);

    // 3. Cycle layer axis
    hm.cycle_layer_axis(); // Y -> X
    hm.cycle_layer_axis(); // X -> Z
    hm.cycle_layer_axis(); // Z -> Y

    // 4. EasyPlace protocol cycle
    hm.cycle_easy_place_protocol();
}

#[test]
fn test_horizonmatica_projects_and_versioning() {
    let mut hm = HorizonMaticaMod::new();
    assert!(hm.load_schematic("sample_shelter").is_ok());

    let ver1 = hm.commit_project_version("v1.1", "Added roof decoration").expect("Commit version");
    assert_eq!(ver1, 2, "First commit beyond initial should be version 2");

    let ver2 = hm.commit_project_version("v1.2", "Added basement").expect("Commit version");
    assert_eq!(ver2, 3);
}

#[test]
fn test_vlakee_ui_registration_and_tab() {
    let mods = get_mod_list();
    let vui = mods.iter().find(|m| m.id == "vlakee_ui");
    assert!(vui.is_some(), "VlakeeUI must be registered in VkModHub");
    assert_eq!(vui.unwrap().name, "VlakeeUI");

    // Test tab switching to VlakeeUI
    let mut ran_tab = false;
    on_command("vk tab vui", &mut ran_tab);
    assert!(ran_tab, "/vk tab vui must switch active tab to VlakeeUI");

    let lines = get_hud_lines();
    assert!(!lines.is_empty(), "HUD lines should not be empty for VlakeeUI tab");
    assert!(
        lines.iter().any(|l| l.contains("SmartF3") || l.contains("Inventario")),
        "HUD lines must contain SmartF3 or Inventario"
    );

    // Switch back to all
    let mut ran_all = false;
    on_command("vk tab all", &mut ran_all);
    assert!(ran_all);
}

#[test]
fn test_vlakee_ui_commands_and_features() {
    let mut vui = VlakeeUIMod::new();
    vui.on_player_pos(128, 64, -256);
    vui.on_biome_found(128, -256, 1, "Cherry Grove");

    // 1. Test F3 command
    assert!(vui.on_command("vui f3 full"));
    assert_eq!(vui.f3.mode, vlakee_ui::F3Mode::Full);

    // 2. Test Nether portal target
    let (nx, ny, nz, label) = vui.f3.portal_target_pos();
    assert_eq!((nx, ny, nz), (16, 64, -32));
    assert!(label.contains("Nether"));

    // 3. Test In-game calculator
    vui.on_command("vui calc 128 * 8");
    assert!(vui.toasts.iter().any(|t| t.text.contains("1024")));

    // 4. Test Night Vision toggle
    vui.on_command("vui nv 80");
    assert!(vui.toggles.night_vision);
    assert!((vui.toggles.brightness_level - 0.80).abs() < 0.05);

    // 5. Test Chunk Viewer mode
    vui.on_command("vui chunk grid");
    assert_eq!(vui.chunk_viewer.mode, vlakee_ui::ChunkViewerMode::FullGrid);
    let boxes = vui.render_bounding_boxes();
    assert!(!boxes.is_empty(), "FullGrid chunk viewer must produce bounding boxes");

    // 6. Test Settings Buttons & Clicks
    let buttons = vui.settings_buttons();
    assert!(!buttons.is_empty(), "Must provide touch buttons for Page 1");
    vui.on_settings_click(6); // Switch to Page 2
    let buttons_p2 = vui.settings_buttons();
    assert!(buttons_p2.iter().any(|b| b.contains("Chunk Viewer")));
}

#[test]
fn test_vlakee_ui_in_game_options() {
    let mut vui = VlakeeUIMod::new();
    vui.on_player_pos(200, 72, -450);

    // 1. Camera perspective toggle via F5 and /vui cam
    assert_eq!(vui.options.perspective, vlakee_ui::CameraPerspective::FirstPerson);
    vui.on_command("f5");
    assert_eq!(vui.options.perspective, vlakee_ui::CameraPerspective::ThirdPersonBack);
    vui.on_command("vui cam front");
    assert_eq!(vui.options.perspective, vlakee_ui::CameraPerspective::ThirdPersonFront);
    vui.on_command("vui cam first");
    assert_eq!(vui.options.perspective, vlakee_ui::CameraPerspective::FirstPerson);

    // 2. Quick Gamemode switcher
    assert_eq!(vui.options.gamemode, vlakee_ui::QuickGamemode::Survival);
    vui.on_command("vui gm c");
    assert_eq!(vui.options.gamemode, vlakee_ui::QuickGamemode::Creative);
    vui.on_command("vui gm sp");
    assert_eq!(vui.options.gamemode, vlakee_ui::QuickGamemode::Spectator);
    vui.on_command("vui gm s");
    assert_eq!(vui.options.gamemode, vlakee_ui::QuickGamemode::Survival);

    // 3. Quick FOV Presets
    vui.on_command("vui fov zoom");
    assert_eq!(vui.options.fov, vlakee_ui::FovPreset::Zoom30);
    vui.on_command("vui fov quake");
    assert_eq!(vui.options.fov, vlakee_ui::FovPreset::QuakePro110);
    vui.on_command("vui fov normal");
    assert_eq!(vui.options.fov, vlakee_ui::FovPreset::Normal75);

    // 4. Temporary Notes
    vui.on_command("vui note Mina de diamantes en Y: -58");
    vui.on_command("vui note Fortalezas a explorar");
    assert_eq!(vui.options.notes.entries.len(), 2);
    vui.on_command("vui notes");
    vui.on_command("vui clearnotes");
    assert_eq!(vui.options.notes.entries.len(), 0);

    // 5. Coordinate Waypoints
    vui.on_command("vui mark Aldea Desierto");
    assert_eq!(vui.options.waypoints.len(), 1);
    assert_eq!(vui.options.waypoints[0].name, "Aldea Desierto");
    assert_eq!(vui.options.waypoints[0].x, 200);
    assert_eq!(vui.options.waypoints[0].z, -450);

    // 6. Chat coordinates sharing
    let chat_text = vui.share_current_coords();
    assert!(chat_text.contains("200, Y: 72, Z: -450"));

    // 7. Quick Container mode
    vui.on_command("vui container drop");
    assert_eq!(vui.options.container_mode, vlakee_ui::QuickContainerMode::DropAll);
    vui.on_command("vui container move");
    assert_eq!(vui.options.container_mode, vlakee_ui::QuickContainerMode::MoveAll);

    // 8. In-Game Pause Drawer lines
    vui.on_command("vui pause");
    assert!(vui.options.is_pause_menu_open);
    let hud = vui.hud_lines();
    assert!(hud.iter().any(|l| l.contains("Opciones Dentro del Juego")));

    // 9. 3-Page Navigation and Page 3 touch buttons
    vui.menu.set_page(vlakee_ui::VlakeeMenuPage::Page3);
    let p3_btns = vui.settings_buttons();
    assert!(p3_btns.iter().any(|b| b.contains("Cámara")));
    assert!(p3_btns.iter().any(|b| b.contains("Modo de Juego")));
    assert!(p3_btns.iter().any(|b| b.contains("Campo Visión")));

    // Click camera button on Page 3
    vui.on_settings_click(0);
    assert_eq!(vui.options.perspective, vlakee_ui::CameraPerspective::ThirdPersonBack);
}

#[test]
fn test_vlakee_ui_smart_f3_more_and_quick_craft_and_master_killswitch() {
    let mut vui = VlakeeUIMod::new();
    vui.on_player_pos(100, 64, -200);

    // 1. Master F3 Killswitch
    assert!(vui.master_f3_active);
    vui.on_command("vui f3mute");
    assert!(!vui.master_f3_active);
    assert_eq!(vui.f3.mode, vlakee_ui::F3Mode::Disabled);
    let hud = vui.hud_lines();
    assert!(hud.iter().any(|l| l.contains("F3 Maestro: APAGADO")));

    // Re-enable Master F3
    vui.on_command("vui f3mute");
    assert!(vui.master_f3_active);
    assert_eq!(vui.f3.mode, vlakee_ui::F3Mode::Compact);

    // Also test global toggle
    assert!(vkyakee_mods::toggle_master_f3_killswitch() == false);
    assert!(vkyakee_mods::is_default_f3_suppressed());
    assert!(vkyakee_mods::toggle_master_f3_killswitch() == true);
    assert!(!vkyakee_mods::is_default_f3_suppressed());

    // 2. SmartF3 More: Speedrunner F-Keys
    // F1: Cinema / Hide HUD
    vui.on_command("f1");
    assert!(vui.f3_more.f1_hud_hidden);
    assert!(vui.hud_lines().is_empty(), "F1 must completely hide all HUD elements");
    vui.on_command("f1");
    assert!(!vui.f3_more.f1_hud_hidden);

    // F3+B: Hitboxes
    vui.on_command("vui f3 b");
    assert!(vui.f3_more.entity_hitboxes);
    assert!(vui.hud_lines().iter().any(|l| l.contains("Hitboxes")));

    // F3+G: Chunk Boundaries
    vui.on_command("vui f3 g");
    assert!(vui.f3_more.chunk_boundaries);
    assert_eq!(vui.chunk_viewer.mode, vlakee_ui::ChunkViewerMode::FullGrid);

    // F3+H: Advanced Tooltips
    vui.on_command("vui f3 h");
    assert!(!vui.f3_more.advanced_tooltips);
    vui.on_command("vui f3 h");
    assert!(vui.f3_more.advanced_tooltips);

    // F3+N: Spectator swap
    assert_eq!(vui.options.gamemode, vlakee_ui::QuickGamemode::Survival);
    vui.on_command("vui f3 n");
    assert_eq!(vui.options.gamemode, vlakee_ui::QuickGamemode::Spectator);
    vui.on_command("vui f3 n");
    assert_eq!(vui.options.gamemode, vlakee_ui::QuickGamemode::Survival);

    // F3+C: Copy TP Coords
    let tp = vui.copy_tp_coords();
    assert!(tp.contains("/execute in minecraft:overworld run tp @s 100.00 64.00 -200.00"));

    // Shift+F3: Pie Chart Profiler
    vui.on_command("vui pie");
    assert!(vui.f3_more.pie_chart_profiler);

    // 3. Fast Crafting / Mass Crafting
    assert_eq!(vui.quick_craft.mode, vlakee_ui::QuickCraftMode::Normal);
    vui.on_command("vui craft mode");
    assert_eq!(vui.quick_craft.mode, vlakee_ui::QuickCraftMode::MassCraft);

    // Test quick craft recipe
    vui.on_command("vui craft planks");
    assert!(vui.quick_craft.last_crafted.is_some());
    let summary = vui.quick_craft.last_crafted.as_ref().unwrap();
    assert!(summary.contains("256x")); // 64 logs -> 256 planks in MassCraft

    // 4. Page 4 and Page 5 in Settings Menu
    vui.menu.set_page(vlakee_ui::VlakeeMenuPage::Page4More);
    let p4_btns = vui.settings_buttons();
    assert!(p4_btns.iter().any(|b| b.contains("F1 (Ocultar HUD)")));
    assert!(p4_btns.iter().any(|b| b.contains("F3+B (Hitboxes)")));
    assert!(p4_btns.iter().any(|b| b.contains("F3+G (Bordes Chunks)")));

    vui.menu.set_page(vlakee_ui::VlakeeMenuPage::Page5Craft);
    let p5_btns = vui.settings_buttons();
    assert!(p5_btns.iter().any(|b| b.contains("Modo Crafteo")));
    assert!(p5_btns.iter().any(|b| b.contains("Madera ➔ Tablones")));
    assert!(p5_btns.iter().any(|b| b.contains("Lingotes ➔ Bloque")));

    // Click quick craft planks from touch menu
    vui.on_settings_click(1);
    assert!(vui.toasts.iter().any(|t| t.text.contains("Crafteado:")));
}

#[test]
fn test_vkmap_registration_and_hud() {
    let mods = get_mod_list();
    let map_mod = mods.iter().find(|m| m.id == "vkmap");
    assert!(map_mod.is_some(), "VkMap must be registered in VkModHub");
    assert_eq!(map_mod.unwrap().name, "VkMap (Horizon Rust)");

    // Switch to VkMap HUD Tab
    let mut ran = false;
    on_command("vk tab map", &mut ran);
    assert!(ran, "Command 'vk tab map' must be claimed");

    let lines = get_hud_lines();
    assert!(!lines.is_empty(), "VkMap HUD lines must not be empty");
    assert!(lines.iter().any(|l| l.contains("[VkMap]")), "HUD must contain [VkMap] header");
    assert!(lines.iter().any(|l| l.contains("Radar")), "HUD must contain Radar line");
    assert!(lines.iter().any(|l| l.contains("🔍+ Ampliar")), "HUD must contain touch floating widgets line");
}

#[test]
fn test_vkmap_display_sizes_and_touch_widgets() {
    use vkyakee_mods::VkMapMod;
    use vkmap::{MinimapDisplaySize, TouchWidgetAction};

    let mut map = VkMapMod::new();

    // Default size is Medium (144px)
    assert_eq!(map.engine.minimap.config.display_size, MinimapDisplaySize::Medium);
    assert_eq!(map.engine.minimap.config.display_size.pixel_size(), 144);

    // Expand size to Fullscreen (280px)
    map.engine.touch_widgets.handle_action(TouchWidgetAction::ExpandSize);
    map.engine.minimap.config.display_size = map.engine.touch_widgets.display_size;
    assert_eq!(map.engine.minimap.config.display_size, MinimapDisplaySize::Fullscreen);
    assert_eq!(map.engine.minimap.config.display_size.pixel_size(), 280);

    // Shrink size back to Medium (144px) then Small (96px)
    map.engine.touch_widgets.handle_action(TouchWidgetAction::ShrinkSize);
    map.engine.minimap.config.display_size = map.engine.touch_widgets.display_size;
    assert_eq!(map.engine.minimap.config.display_size, MinimapDisplaySize::Medium);

    map.engine.touch_widgets.handle_action(TouchWidgetAction::ShrinkSize);
    map.engine.minimap.config.display_size = map.engine.touch_widgets.display_size;
    assert_eq!(map.engine.minimap.config.display_size, MinimapDisplaySize::Small);
    assert_eq!(map.engine.minimap.config.display_size.pixel_size(), 96);

    // Zoom controls
    assert_eq!(map.engine.minimap.config.zoom, 1.0);
    map.engine.touch_widgets.handle_action(TouchWidgetAction::ZoomIn);
    assert_eq!(map.engine.touch_widgets.zoom_level, 2.0);

    // Map Menu toggle
    assert!(!map.engine.touch_widgets.map_menu_open);
    map.engine.touch_widgets.handle_action(TouchWidgetAction::OpenMapMenu);
    assert!(map.engine.touch_widgets.map_menu_open);
}

#[test]
fn test_vkmap_radar_visibility_modes() {
    use vkyakee_mods::VkMapMod;
    use vkmap::{EntityCategory, RadarEntity, RadarVisibilityMode};

    let mut map = VkMapMod::new();
    map.player_pos = (0.0, 64.0, 0.0);
    map.player_yaw = 0.0; // Facing South (+Z)

    // Entity 1: in front of player (+Z), Y=64 (Line of sight & Surface)
    let zombie = RadarEntity::new(1, "Zombie", EntityCategory::Hostile, 0.0, 64.0, 20.0, 0.0);
    // Entity 2: behind player (-Z), Y=64
    let cow = RadarEntity::new(2, "Cow", EntityCategory::Passive, 0.0, 64.0, -20.0, 0.0);
    // Entity 3: underground cave, Y=-20
    let skeleton = RadarEntity::new(3, "Skeleton", EntityCategory::Hostile, 0.0, -20.0, 10.0, 0.0);

    let entities = vec![zombie, cow, skeleton];

    // 1. AllLayers Mode: sees all 3 entities
    map.engine.radar_tracker.visibility_mode = RadarVisibilityMode::AllLayers;
    let contacts_all = map.engine.radar_tracker.update_contacts(0.0, 64.0, 0.0, 0.0, false, 1.0, &entities);
    assert_eq!(contacts_all.len(), 3);

    // 2. LineOfSight Mode: only sees zombie (in front)
    map.engine.radar_tracker.visibility_mode = RadarVisibilityMode::LineOfSight;
    let contacts_los = map.engine.radar_tracker.update_contacts(0.0, 64.0, 0.0, 0.0, false, 1.0, &entities);
    assert_eq!(contacts_los.len(), 1);
    assert_eq!(contacts_los[0].entity.id, 1);

    // 3. SurfaceOnly Mode: sees zombie and cow (Y=64 +- 5), ignores underground skeleton
    map.engine.radar_tracker.visibility_mode = RadarVisibilityMode::SurfaceOnly;
    let contacts_surf = map.engine.radar_tracker.update_contacts(0.0, 64.0, 0.0, 0.0, false, 1.0, &entities);
    assert_eq!(contacts_surf.len(), 2);
    assert!(contacts_surf.iter().any(|c| c.entity.id == 1));
    assert!(contacts_surf.iter().any(|c| c.entity.id == 2));
    assert!(!contacts_surf.iter().any(|c| c.entity.id == 3));

    // 4. Fair-Play Mode (anti-cheat hides hostile entities)
    map.engine.radar_tracker.visibility_mode = RadarVisibilityMode::AllLayers;
    map.engine.radar_tracker.fair_play_mode = true;
    let contacts_fair = map.engine.radar_tracker.update_contacts(0.0, 64.0, 0.0, 0.0, false, 1.0, &entities);
    assert_eq!(contacts_fair.len(), 1);
    assert_eq!(contacts_fair[0].entity.id, 2); // only passive cow visible
}

#[test]
fn test_vkmap_seedcracker_synergy_and_subterranean() {
    use vkyakee_mods::VkMapMod;
    use vkmap::StructureSource;

    let mut map = VkMapMod::new();

    // 1. Mathematical seed prediction from SeedCracker candidate
    let test_seed = 0x12345678ABCD;
    map.sync_predicted_structures(&[test_seed]);

    let predicted = map.engine.structure_bridge.structures.iter()
        .filter(|s| s.source == StructureSource::SeedPrediction)
        .count();
    assert!(predicted > 0, "SeedCracker sync must generate [PRED] predicted structures");

    // 2. In-game verified structure discovery
    map.on_structure_found("minecraft:trial_chambers", 10, 20);
    let verified = map.engine.structure_bridge.structures.iter()
        .find(|s| s.structure_type == "minecraft:trial_chambers" && s.chunk_x == 10 && s.chunk_z == 20);
    assert!(verified.is_some(), "World discovery must register verified structure");
    assert_eq!(verified.unwrap().source, StructureSource::WorldScan);
    assert_eq!(verified.unwrap().source.badge(), "[VERIF]");

    // 3. Subterranean cave analyzer for negative Y layers (-64..0)
    map.cave_mode_active = true;
    map.on_chunk_loaded(17, 17, "minecraft:overworld", None);
    assert!(map.toasts.iter().any(|t| t.text.contains("Subterráneo:")));
}

#[test]
fn test_vkmap_multiworld_persistence_and_exports() {
    use vkyakee_mods::VkMapMod;

    let mut map = VkMapMod::new();
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let hypixel_id = format!("test_hypixel_{}", ts);
    let survival_id = format!("test_survival_{}", ts);

    // 1. World session isolation
    map.switch_world(&hypixel_id);
    map.create_waypoint_at_player("Hypixel Spawn", [255, 200, 0]);
    assert_eq!(map.engine.waypoint_manager.len(), 1);

    map.switch_world(&survival_id);
    assert_eq!(map.engine.waypoint_manager.len(), 0, "Survival world must have separate fresh state");
    map.create_waypoint_at_player("Home Base", [0, 255, 100]);
    assert_eq!(map.engine.waypoint_manager.len(), 1);

    // Switch back to Hypixel
    map.switch_world(&hypixel_id);
    assert_eq!(map.engine.waypoint_manager.len(), 1, "Hypixel waypoints must be cleanly reloaded from disk");
    assert_eq!(map.engine.waypoint_manager.list_all()[0].name, "Hypixel Spawn");

    // 2. PNG and JSON Exports
    let png_res = map.export_png();
    assert!(png_res.is_ok(), "PNG map export must succeed: {:?}", png_res.err());
    let png_path = png_res.unwrap();
    assert!(png_path.exists(), "Exported PNG file must exist on disk");

    let json_res = map.export_json();
    assert!(json_res.is_ok(), "JSON dataset export must succeed: {:?}", json_res.err());
    let json_path = json_res.unwrap();
    assert!(json_path.exists(), "Exported JSON file must exist on disk");

    // Verify JSON content
    let json_str = std::fs::read_to_string(&json_path).unwrap();
    assert!(json_str.contains("Hypixel Spawn"));
    assert!(json_str.contains("total_waypoints"));
}



