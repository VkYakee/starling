use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ToolMode {
    #[default]
    AreaSelection,
    SchematicPlacement,
    Fill,
    ReplaceBlock,
    PasteSchematic,
    GridPaste,
    Move,
    Delete,
    Rebuild,
}

impl ToolMode {
    pub fn name(&self) -> &'static str {
        match self {
            ToolMode::AreaSelection => "Area Selection",
            ToolMode::SchematicPlacement => "Schematic Placement",
            ToolMode::Fill => "Fill",
            ToolMode::ReplaceBlock => "Replace Block",
            ToolMode::PasteSchematic => "Paste Schematic",
            ToolMode::GridPaste => "Grid Paste",
            ToolMode::Move => "Move",
            ToolMode::Delete => "Delete",
            ToolMode::Rebuild => "Rebuild",
        }
    }

    pub fn config_name(&self) -> &'static str {
        match self {
            ToolMode::AreaSelection => "area_selection",
            ToolMode::SchematicPlacement => "schematic_placement",
            ToolMode::Fill => "fill",
            ToolMode::ReplaceBlock => "replace_block",
            ToolMode::PasteSchematic => "paste_schematic",
            ToolMode::GridPaste => "grid_paste",
            ToolMode::Move => "move",
            ToolMode::Delete => "delete",
            ToolMode::Rebuild => "rebuild",
        }
    }

    pub fn is_creative_only(&self) -> bool {
        matches!(
            self,
            ToolMode::Fill
                | ToolMode::ReplaceBlock
                | ToolMode::PasteSchematic
                | ToolMode::GridPaste
                | ToolMode::Move
                | ToolMode::Delete
        )
    }

    pub fn uses_schematic(&self) -> bool {
        matches!(
            self,
            ToolMode::SchematicPlacement | ToolMode::PasteSchematic | ToolMode::GridPaste | ToolMode::Rebuild
        )
    }

    pub fn uses_area_selection(&self) -> bool {
        !self.uses_schematic()
    }

    pub fn uses_primary_block(&self) -> bool {
        matches!(self, ToolMode::Fill | ToolMode::ReplaceBlock | ToolMode::Rebuild)
    }

    pub fn uses_secondary_block(&self) -> bool {
        matches!(self, ToolMode::ReplaceBlock)
    }

    pub fn cycle_next(self) -> Self {
        match self {
            ToolMode::AreaSelection => ToolMode::SchematicPlacement,
            ToolMode::SchematicPlacement => ToolMode::Fill,
            ToolMode::Fill => ToolMode::ReplaceBlock,
            ToolMode::ReplaceBlock => ToolMode::PasteSchematic,
            ToolMode::PasteSchematic => ToolMode::GridPaste,
            ToolMode::GridPaste => ToolMode::Move,
            ToolMode::Move => ToolMode::Delete,
            ToolMode::Delete => ToolMode::Rebuild,
            ToolMode::Rebuild => ToolMode::AreaSelection,
        }
    }
}
