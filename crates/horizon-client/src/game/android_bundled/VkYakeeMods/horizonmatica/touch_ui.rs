// Copyright (c) 2026 HorizonMatica Authors / Starling Android
//! Bedrock Edition touch action bar, pill toggle, and modal touch event routing.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModalKind {
    None,
    FileBrowser,
    MaterialList,
    LayerConfig,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TouchAction {
    ToggleBar,
    OpenFileBrowser,
    ToggleLayer,
    LayerUp,
    LayerDown,
    CycleRotation,
    CycleMirror,
    ToggleVerifier,
    OpenMaterialList,
    SetPositionHere,
    UnloadSchematic,
    LoadFile(String),
    CloseModal,
    ScrollUp,
    ScrollDown,
    RescanFiles,
}

#[derive(Clone, Debug)]
pub struct TouchButton {
    pub id: &'static str,
    pub rect: [f32; 4], // [x, y, w, h]
    pub label: String,
    pub active: bool,
    pub action: TouchAction,
}

impl TouchButton {
    pub fn contains(&self, cursor: [f32; 2]) -> bool {
        let x = self.rect[0];
        let y = self.rect[1];
        let w = self.rect[2];
        let h = self.rect[3];
        cursor[0] >= x && cursor[0] <= x + w && cursor[1] >= y && cursor[1] <= y + h
    }
}

#[derive(Clone, Debug)]
pub struct TouchUiState {
    pub bar_open: bool,
    pub active_modal: ModalKind,
    pub scroll_offset: f32,
    pub buttons: Vec<TouchButton>,
    pub modal_buttons: Vec<TouchButton>,
}

impl Default for TouchUiState {
    fn default() -> Self {
        Self {
            bar_open: false,
            active_modal: ModalKind::None,
            scroll_offset: 0.0,
            buttons: Vec::new(),
            modal_buttons: Vec::new(),
        }
    }
}

impl TouchUiState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn close_modal(&mut self) {
        self.active_modal = ModalKind::None;
        self.scroll_offset = 0.0;
        self.modal_buttons.clear();
    }
}
