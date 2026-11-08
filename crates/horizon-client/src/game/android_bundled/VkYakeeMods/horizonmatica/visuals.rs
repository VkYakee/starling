// Copyright (c) 2026 HorizonMatica Authors / Starling Android
//! 3D visual projection: renders depth-tested outlines (NativeBlockRim) for
//! schematic block mismatches and placement bounding boxes.

use crate::game::native_hooks::NativeBlockRim;
use super::format::LoadedSchematic;
use super::placement::SchematicPlacement;
use super::verifier::{MismatchType, SchematicVerifier};

pub struct VisualRenderer;

impl VisualRenderer {
    pub fn collect_rims(
        rims: &mut Vec<NativeBlockRim>,
        schematic: Option<&LoadedSchematic>,
        placement: &SchematicPlacement,
        verifier: &SchematicVerifier,
        player_pos: [f64; 3],
    ) {
        let Some(schem) = schematic else {
            return;
        };
        if !placement.enabled {
            return;
        }

        // 1. Highlight closest mismatches when verifier is active
        if verifier.active && !verifier.mismatches.is_empty() {
            let px = player_pos[0];
            let py = player_pos[1];
            let pz = player_pos[2];

            // Sort a small batch of closest mismatches
            let mut sorted = verifier.mismatches.clone();
            sorted.sort_by(|a, b| {
                let da = (a.pos[0] as f64 - px).powi(2)
                    + (a.pos[1] as f64 - py).powi(2)
                    + (a.pos[2] as f64 - pz).powi(2);
                let db = (b.pos[0] as f64 - px).powi(2)
                    + (b.pos[1] as f64 - py).powi(2)
                    + (b.pos[2] as f64 - pz).powi(2);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            });

            for m in sorted.into_iter().take(16) {
                rims.push(NativeBlockRim {
                    pos: m.pos,
                    argb: m.mismatch_type.color_argb(),
                    width: 2.0,
                });
            }
        }

        // 2. Highlight placement bounding box corners (Gold)
        let (min_b, max_b) = placement.compute_world_bounds(schem);
        let corners = [
            min_b,
            [max_b[0], min_b[1], min_b[2]],
            [min_b[0], max_b[1], min_b[2]],
            [min_b[0], min_b[1], max_b[2]],
            max_b,
            [min_b[0], max_b[1], max_b[2]],
            [max_b[0], min_b[1], max_b[2]],
            [max_b[0], max_b[1], min_b[2]],
        ];

        for c in corners {
            if rims.len() >= 32 {
                break;
            }
            rims.push(NativeBlockRim {
                pos: c,
                argb: 0xFFFF_D700, // Gold
                width: 2.5,
            });
        }
    }
}
