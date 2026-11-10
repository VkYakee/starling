//! Lattice reduction for seed searching
//!
//! This module provides lattice-based algorithms for finding seeds
//! that satisfy multiple modular constraints simultaneously.

use std::ops::{Add, Mul, Sub};

/// Matrix for lattice operations
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Matrix {
    data: Vec<Vec<i128>>,
    rows: usize,
    cols: usize,
}

impl Matrix {
    /// Create a new matrix
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            data: vec![vec![0; cols]; rows],
            rows,
            cols,
        }
    }

    /// Create from 2D vector
    pub fn from_vec(data: Vec<Vec<i128>>) -> Self {
        let rows = data.len();
        let cols = if rows > 0 { data[0].len() } else { 0 };
        Self { data, rows, cols }
    }

    /// Get element
    pub fn get(&self, row: usize, col: usize) -> i128 {
        self.data[row][col]
    }

    /// Set element
    pub fn set(&mut self, row: usize, col: usize, value: i128) {
        self.data[row][col] = value;
    }

    /// Get row
    pub fn row(&self, row: usize) -> &[i128] {
        &self.data[row]
    }

    /// Get mutable row
    pub fn row_mut(&mut self, row: usize) -> &mut [i128] {
        &mut self.data[row]
    }

    /// Number of rows
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Number of columns
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Swap rows
    pub fn swap_rows(&mut self, r1: usize, r2: usize) {
        self.data.swap(r1, r2);
    }

    /// Multiply row by scalar
    pub fn mul_row(&mut self, row: usize, scalar: i128) {
        for val in &mut self.data[row] {
            *val *= scalar;
        }
    }

    /// Add multiple of one row to another
    pub fn add_row_multiple(&mut self, target: usize, source: usize, mult: i128) {
        for c in 0..self.cols {
            self.data[target][c] += self.data[source][c] * mult;
        }
    }

    /// LLL lattice reduction (simplified)
    pub fn lll_reduce(&mut self, delta: f64) {
        let n = self.rows.min(self.cols);
        let mut k = 1;
        
        while k < n {
            // Size reduction
            for j in (0..k).rev() {
                let mu = (self.get(k, j) as f64) / (self.get(j, j) as f64);
                let mu_round = mu.round() as i128;
                if mu_round != 0 {
                    self.add_row_multiple(k, j, -mu_round);
                }
            }

            // Lovász condition
            let b_k_sq = self.row_norm_sq(k);
            let b_k1_sq = if k > 0 { self.row_norm_sq(k - 1) } else { 1 };
            
            if (b_k_sq as f64) < (delta * b_k1_sq as f64) {
                if k > 0 {
                    self.swap_rows(k, k - 1);
                    k -= 1;
                } else {
                    k += 1;
                }
            } else {
                k += 1;
            }
        }
    }

    fn row_norm_sq(&self, row: usize) -> i128 {
        self.data[row].iter().map(|&x| x * x).sum()
    }
}

/// Lattice for seed searching
#[derive(Clone, Debug)]
pub struct Lattice {
    basis: Matrix,
}

impl Lattice {
    /// Create a new lattice from basis vectors
    pub fn new(basis: Matrix) -> Self {
        Self { basis }
    }

    /// Create lattice for solving modular equations
    /// Given constraints: seed ≡ a_i (mod m_i)
    pub fn from_constraints(constraints: &[(u64, u64)]) -> Self {
        let n = constraints.len();
        let mut basis = Matrix::new(n + 1, n + 1);
        
        // Diagonal: moduli
        for i in 0..n {
            basis.set(i, i, constraints[i].1 as i128);
        }
        
        // Last row: target values
        for i in 0..n {
            basis.set(n, i, constraints[i].0 as i128);
        }
        basis.set(n, n, 1);
        
        Self { basis }
    }

    /// Get the basis matrix
    pub fn basis(&self) -> &Matrix {
        &self.basis
    }

    /// Get mutable basis matrix
    pub fn basis_mut(&mut self) -> &mut Matrix {
        &mut self.basis
    }

    /// Run LLL reduction
    pub fn reduce(&mut self, delta: f64) {
        self.basis.lll_reduce(delta);
    }

    /// Find short vectors (potential solutions)
    pub fn find_short_vectors(&self, max_norm: i128) -> Vec<Vec<i128>> {
        let mut results = Vec::new();
        
        for i in 0..self.basis.rows() {
            let norm_sq = self.basis.row_norm_sq(i);
            if norm_sq <= max_norm * max_norm {
                results.push(self.basis.row(i).to_vec());
            }
        }
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_basic() {
        let mut m = Matrix::new(2, 2);
        m.set(0, 0, 1);
        m.set(0, 1, 2);
        m.set(1, 0, 3);
        m.set(1, 1, 4);
        
        assert_eq!(m.get(0, 0), 1);
        assert_eq!(m.get(1, 1), 4);
    }

    #[test]
    fn test_lattice_from_constraints() {
        // seed ≡ 5 (mod 7)
        // seed ≡ 3 (mod 11)
        let constraints = [(5u64, 7u64), (3u64, 11u64)];
        let lattice = Lattice::from_constraints(&constraints);
        
        assert_eq!(lattice.basis().rows(), 3);
        assert_eq!(lattice.basis().cols(), 3);
    }
}
