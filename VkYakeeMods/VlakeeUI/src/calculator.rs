//! In-game math, stack, and portal calculator ported from Déesse UI for Starling.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationResult {
    pub expression: String,
    pub result: f64,
    pub stack_info: Option<String>,
}

pub struct InGameCalculator;

impl InGameCalculator {
    /// Evaluates a simple arithmetic expression (supports +, -, *, /)
    pub fn evaluate(expr: &str) -> Result<f64, String> {
        let clean = expr.trim().replace(' ', "");
        if clean.is_empty() {
            return Err("Expresión vacía".to_string());
        }

        // Try parsing direct number
        if let Ok(val) = clean.parse::<f64>() {
            return Ok(val);
        }

        // Parse addition / subtraction
        for (idx, ch) in clean.char_indices().rev() {
            if (ch == '+' || ch == '-') && idx > 0 {
                let left = &clean[..idx];
                let right = &clean[idx + 1..];
                let l_val = Self::evaluate(left)?;
                let r_val = Self::evaluate(right)?;
                return Ok(if ch == '+' { l_val + r_val } else { l_val - r_val });
            }
        }

        // Parse multiplication / division
        for (idx, ch) in clean.char_indices().rev() {
            if ch == '*' || ch == '/' {
                let left = &clean[..idx];
                let right = &clean[idx + 1..];
                let l_val = Self::evaluate(left)?;
                let r_val = Self::evaluate(right)?;
                if ch == '/' && r_val.abs() < f64::EPSILON {
                    return Err("División por cero".to_string());
                }
                return Ok(if ch == '*' { l_val * r_val } else { l_val / r_val });
            }
        }

        Err(format!("Sintaxis no válida: {}", expr))
    }

    /// Converts an item count into stacks, remainder, shulker boxes, and chests.
    pub fn calculate_stacks(count: usize) -> String {
        let stacks = count / 64;
        let remainder = count % 64;
        let shulkers = count as f64 / (64.0 * 27.0);
        let chests = count as f64 / (64.0 * 54.0);

        format!(
            "{} items = {} stacks + {} (shulkers: {:.2}, cofres: {:.2})",
            count, stacks, remainder, shulkers, chests
        )
    }

    /// Calculates Nether to Overworld coordinates and vice-versa.
    pub fn calculate_portal(x: i32, z: i32, to_nether: bool) -> (i32, i32, String) {
        if to_nether {
            let nx = x / 8;
            let nz = z / 8;
            (nx, nz, format!("Overworld [{}, {}] ➔ Nether [{}, {}]", x, z, nx, nz))
        } else {
            let ox = x * 8;
            let oz = z * 8;
            (ox, oz, format!("Nether [{}, {}] ➔ Overworld [{}, {}]", x, z, ox, oz))
        }
    }
}
