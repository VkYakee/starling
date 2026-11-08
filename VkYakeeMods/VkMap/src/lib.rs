//! # Map_Rust: High-Performance Minecraft Map Engine
//!
//! A modular, all-in-one mapping engine written in Rust combining the finest features from
//! the most acclaimed Java Minecraft mapping mods:
//! - **Squaremap & Pl3xMap**: Ultra-fast ray casting, slope/elevation relief shading, and fluid depth blending.
//! - **Dynmap**: Multi-face directional block colors, isometric 3D perspectives, cave horizontal slicing, and lighting curves.
//! - **VoxelMap**: In-game circular & square Minimap HUD with stencils, waypoint death tracking, and entity radar with elevation indicators.
//! - **XaeroPlus**: Multi-world/server cache separation, fair-play anti-cheat radar restrictions, 1:8 Nether dimension conversion, and waypoint ETA.
//! - **BlueMap**: Multi-resolution Level-of-Detail (LOD) tile pyramid hierarchies.
//! - **Conflux-Map**: Vector drawing annotations, towny/chunk claim overlays, and highway paths.
//! - **Diorama & Minecraft-Overviewer**: Frame masks, clean radar icons, and axonometric projections.

pub mod annotations;
pub mod color;
pub mod export;
pub mod hud;
pub mod radar;
pub mod render;
pub mod storage;
pub mod structure_bridge;
pub mod waypoints;

pub use annotations::{Annotation, AnnotationManager, AnnotationShape};
pub use color::{BiomePalette, BlockPalette, DepthShadingConfig, DepthShadingEngine};
pub use export::{MapExportDataset, MapExporter};
pub use hud::{
    CoordinatesFormatter, FacingDirection, MinimapCompositor, MinimapConfig, MinimapDisplaySize, MinimapShape,
    MinimapTouchWidgets, TouchWidgetAction,
};
pub use radar::{
    ElevationIndicator, EntityCategory, RadarContact, RadarEntity, RadarRenderer, RadarTracker,
    RadarVisibilityMode,
};
pub use render::{
    BlockColumn, CaveSliceRenderer, ChunkData, ChunkScanner, ImageBuffer, IsometricRenderer, LightingEngine,
    LightingMode, RenderMode, SubterraneanStructureAnalyzer,
};
pub use storage::{ChunkTileCache, MapStorage, TileCoord};
pub use structure_bridge::{StructureBridge, StructureInfo, StructureSource};
pub use waypoints::{
    generate_teleport_command, nether_to_overworld, overworld_to_nether, Dimension, TravelSpeed, Waypoint,
    WaypointManager,
};

use std::path::Path;

/// Unified Map Engine facade integrating all mapping, radar, waypoint, and rendering features.
#[derive(Debug, Clone)]
pub struct MapEngine {
    pub block_palette: BlockPalette,
    pub biome_palette: BiomePalette,
    pub shading_engine: DepthShadingEngine,
    pub scanner: ChunkScanner,
    pub cave_renderer: CaveSliceRenderer,
    pub iso_renderer: IsometricRenderer,
    pub lighting_engine: LightingEngine,
    pub waypoint_manager: WaypointManager,
    pub radar_tracker: RadarTracker,
    pub radar_renderer: RadarRenderer,
    pub annotation_manager: AnnotationManager,
    pub tile_cache: ChunkTileCache,
    pub storage: MapStorage,
    pub minimap: MinimapCompositor,
    pub structure_bridge: StructureBridge,
    pub touch_widgets: MinimapTouchWidgets,
}

impl MapEngine {
    /// Creates a new MapEngine initialized with default vanilla color palettes and data directory.
    pub fn new<P: AsRef<Path>>(data_dir: P) -> Self {
        let block_palette = BlockPalette::new_with_defaults();
        let biome_palette = BiomePalette::new_with_defaults();
        let shading_engine = DepthShadingEngine::default();

        let scanner = ChunkScanner::new(
            block_palette.clone(),
            biome_palette.clone(),
            shading_engine.clone(),
        );
        let cave_renderer = CaveSliceRenderer::new(block_palette.clone(), 64, 48);
        let iso_renderer = IsometricRenderer::new(block_palette.clone());
        let lighting_engine = LightingEngine::default();
        let waypoint_manager = WaypointManager::new();
        let radar_tracker = RadarTracker::default();
        let radar_renderer = RadarRenderer::default();
        let annotation_manager = AnnotationManager::new();
        let tile_cache = ChunkTileCache::default();
        let storage = MapStorage::new(data_dir);
        let minimap = MinimapCompositor::default();
        let structure_bridge = StructureBridge::new();
        let touch_widgets = MinimapTouchWidgets::new();

        Self {
            block_palette,
            biome_palette,
            shading_engine,
            scanner,
            cave_renderer,
            iso_renderer,
            lighting_engine,
            waypoint_manager,
            radar_tracker,
            radar_renderer,
            annotation_manager,
            tile_cache,
            storage,
            minimap,
            structure_bridge,
            touch_widgets,
        }
    }

    /// Loads external palette files from the assets directory (`assets/colors/`).
    pub fn load_assets<P: AsRef<Path>>(&mut self, assets_dir: P) -> std::io::Result<(usize, usize)> {
        let dir = assets_dir.as_ref();
        let block_file = dir.join("colors").join("block_colors.txt");
        let biome_file = dir.join("colors").join("biome_colors.txt");

        let mut block_count = 0;
        let mut biome_count = 0;

        if block_file.exists() {
            block_count = self.block_palette.load_from_dynmap_file(&block_file)?;
        }
        if biome_file.exists() {
            biome_count = self.biome_palette.load_from_voxelmap_file(&biome_file)?;
        }

        // Re-synchronize scanner and renderers
        self.scanner.block_palette = self.block_palette.clone();
        self.scanner.biome_palette = self.biome_palette.clone();
        self.cave_renderer.palette = self.block_palette.clone();
        self.iso_renderer.palette = self.block_palette.clone();

        Ok((block_count, biome_count))
    }

    /// Renders a chunk into an `ImageBuffer` using the selected render perspective mode.
    pub fn render_chunk(
        &self,
        chunk: &ChunkData,
        north_neighbor_heights: Option<&[i32; 16]>,
        mode: RenderMode,
    ) -> (ImageBuffer, [i32; 16]) {
        match mode {
            RenderMode::CaveMode { slice_y, depth_blocks } => {
                let mut cave = self.cave_renderer.clone();
                cave.slice_y = slice_y;
                cave.depth_blocks = depth_blocks;
                let buf = cave.render_cave_chunk(chunk);
                (buf, [slice_y; 16])
            }
            RenderMode::Isometric => {
                let buf = self.iso_renderer.render_isometric_chunk(chunk);
                (buf, [64; 16])
            }
            _ => self.scanner.scan_chunk(chunk, north_neighbor_heights, mode),
        }
    }

    /// Generates a complete composite Minimap frame for the player HUD.
    pub fn generate_minimap_frame(
        &self,
        base_terrain: &ImageBuffer,
        player_x: f64,
        player_y: f64,
        player_z: f64,
        player_yaw: f32,
        dimension: &Dimension,
        entities: &[RadarEntity],
    ) -> ImageBuffer {
        let contacts = self.radar_tracker.update_contacts(
            player_x,
            player_y,
            player_z,
            player_yaw,
            self.minimap.config.rotating,
            self.minimap.config.zoom,
            entities,
        );

        let active_waypoints = self.waypoint_manager.get_active_in_dimension(dimension);
        let structures = self.structure_bridge.get_in_dimension(dimension);

        self.minimap.composite_minimap(
            base_terrain,
            player_x,
            player_z,
            player_yaw,
            &contacts,
            &active_waypoints,
            &structures,
        )
    }

    /// Automatically records a death point.
    pub fn on_player_death(&mut self, x: i32, y: i32, z: i32, dimension: Dimension) -> &Waypoint {
        self.waypoint_manager.create_death_point(x, y, z, dimension)
    }

    /// Saves all world data (waypoints and vector annotations) to disk.
    pub fn save_world_data(&self, world_id: &str) -> std::io::Result<()> {
        self.storage.save_waypoints(world_id, &self.waypoint_manager)?;
        self.storage.save_annotations(world_id, &self.annotation_manager)?;
        Ok(())
    }

    /// Loads world data from disk if present.
    pub fn load_world_data(&mut self, world_id: &str) {
        if let Some(wps) = self.storage.load_waypoints(world_id) {
            self.waypoint_manager = wps;
        }
        if let Some(anns) = self.storage.load_annotations(world_id) {
            self.annotation_manager = anns;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_engine_facade() {
        let mut engine = MapEngine::new("/tmp/vkmap_engine_test");

        // Add a waypoint
        let wp = Waypoint::new("Spawn Point", 0, 64, 0, Dimension::Overworld, [0, 255, 0]);
        engine.waypoint_manager.add(wp);

        // Add an entity
        let cow = RadarEntity::new(1, "Cow", EntityCategory::Passive, 8.0, 64.0, 4.0, 0.0);

        // Generate minimap frame
        let terrain = ImageBuffer::new_filled(128, 128, [60, 140, 60, 255]);
        let frame = engine.generate_minimap_frame(
            &terrain,
            0.0,
            64.0,
            0.0,
            0.0,
            &Dimension::Overworld,
            &[cow],
        );

        assert_eq!(frame.width, 144);
        assert_eq!(frame.height, 144);
    }
}
