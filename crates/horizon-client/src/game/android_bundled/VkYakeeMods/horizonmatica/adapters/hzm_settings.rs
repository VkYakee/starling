// Copyright (c) 2026 HorizonMatica Authors / Starling Android
//! Settings button labels for HorizonMatica.

pub(in crate::game) fn run(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    let _ = game;
    if mod_id != "horizonmatica" {
        return;
    }
    buttons.push("Toggle Touch Action Bar".to_string());
    buttons.push("Toggle Real-Time Verifier".to_string());
    buttons.push("Rescan Schematics Folder".to_string());
    buttons.push("Unload Active Schematic".to_string());
}
