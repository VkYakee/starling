// Copyright (c) 2026 HorizonMatica Authors / Starling Android
//! Intercepts /horizonmatica, /hzm, and /litematica in game chat.

pub(in crate::game) fn run(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    if *ran {
        return;
    }
    use super::horizonmatica as r;
    let rt = r::get();
    let handled = r::commands::CommandDispatcher::execute(game, command, rt);
    if handled {
        *ran = true;
    }
}
