use super::waypoint::{Dimension, Waypoint};

/// Generates a vanilla Minecraft teleport command to a waypoint.
pub fn generate_teleport_command(wp: &Waypoint, execute_syntax: bool) -> String {
    if execute_syntax {
        let dim_str = match &wp.dimension {
            Dimension::Overworld => "minecraft:overworld",
            Dimension::Nether => "minecraft:the_nether",
            Dimension::TheEnd => "minecraft:the_end",
            Dimension::Custom(s) => s.as_str(),
        };
        format!("/execute in {} run tp @s {} {} {}", dim_str, wp.x, wp.y, wp.z)
    } else {
        format!("/tp @s {} {} {}", wp.x, wp.y, wp.z)
    }
}

/// Converts horizontal coordinates from Nether to Overworld (1:8 ratio).
pub fn nether_to_overworld(nether_x: i32, nether_z: i32) -> (i32, i32) {
    (nether_x * 8, nether_z * 8)
}

/// Converts horizontal coordinates from Overworld to Nether (1:8 ratio).
pub fn overworld_to_nether(overworld_x: i32, overworld_z: i32) -> (i32, i32) {
    (overworld_x / 8, overworld_z / 8)
}

/// Automatically translates coordinates to their dimension counterpart (Nether <-> Overworld).
pub fn get_counterpart_coords(x: i32, z: i32, dim: &Dimension) -> Option<(i32, i32)> {
    match dim {
        Dimension::Overworld => Some(overworld_to_nether(x, z)),
        Dimension::Nether => Some(nether_to_overworld(x, z)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_teleport_commands() {
        let wp = Waypoint::new("Outpost", 150, 72, -300, Dimension::Overworld, [255, 0, 0]);
        let cmd_simple = generate_teleport_command(&wp, false);
        assert_eq!(cmd_simple, "/tp @s 150 72 -300");

        let cmd_exec = generate_teleport_command(&wp, true);
        assert_eq!(cmd_exec, "/execute in minecraft:overworld run tp @s 150 72 -300");
    }

    #[test]
    fn test_nether_conversion() {
        let (ow_x, ow_z) = nether_to_overworld(100, -50);
        assert_eq!((ow_x, ow_z), (800, -400));

        let (n_x, n_z) = overworld_to_nether(800, -400);
        assert_eq!((n_x, n_z), (100, -50));
    }
}
