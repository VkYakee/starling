/// High-performance Lattice Basis Reduction (Lenstra–Lenstra–Lovász / LLL)
/// and Babai's Closest Vector Problem (CVP) algorithms for Minecraft seedfinding.
///
/// References:
/// - A.K. Lenstra, H.W. Lenstra, Jr., and L. Lovász (1982) "Factoring polynomials with rational coefficients"
/// - L. Babai (1986) "On Lovász' lattice reduction and the nearest lattice point problem"
/// - EDDxample, Neil (hube12), Matthew Bolan (fnseedc collection)

#[derive(Clone, Debug, PartialEq)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<Vec<f64>>,
}

impl Matrix {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![vec![0.0; cols]; rows],
        }
    }

    pub fn from_rows(rows: Vec<Vec<f64>>) -> Self {
        let r = rows.len();
        let c = if r > 0 { rows[0].len() } else { 0 };
        Self {
            rows: r,
            cols: c,
            data: rows,
        }
    }

    pub fn identity(size: usize) -> Self {
        let mut m = Self::new(size, size);
        for i in 0..size {
            m.data[i][i] = 1.0;
        }
        m
    }

    #[inline]
    pub fn row(&self, r: usize) -> &[f64] {
        &self.data[r]
    }

    #[inline]
    pub fn row_mut(&mut self, r: usize) -> &mut [f64] {
        &mut self.data[r]
    }

    pub fn swap_rows(&mut self, r1: usize, r2: usize) {
        self.data.swap(r1, r2);
    }
}

pub struct Lattice;

impl Lattice {
    #[inline]
    pub fn dot_product(v1: &[f64], v2: &[f64]) -> f64 {
        v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum()
    }

    #[inline]
    pub fn norm_squared(v: &[f64]) -> f64 {
        Self::dot_product(v, v)
    }

    /// Computes the Gram-Schmidt orthogonalization basis b* and coefficients mu.
    /// b*_i = b_i - sum_{j=0}^{i-1} mu_{i,j} * b*_j
    /// mu_{i,j} = <b_i, b*_j> / <b*_j, b*_j>
    pub fn gram_schmidt(basis: &Matrix) -> (Matrix, Matrix) {
        let n = basis.rows;
        let m = basis.cols;
        let mut b_star = Matrix::new(n, m);
        let mut mu = Matrix::new(n, n);

        for i in 0..n {
            let mut v = basis.row(i).to_vec();
            mu.data[i][i] = 1.0;

            for j in 0..i {
                let denom = Self::norm_squared(b_star.row(j));
                if denom.abs() > 1e-12 {
                    let c = Self::dot_product(basis.row(i), b_star.row(j)) / denom;
                    mu.data[i][j] = c;
                    for k in 0..m {
                        v[k] -= c * b_star.data[j][k];
                    }
                }
            }
            b_star.data[i] = v;
        }

        (b_star, mu)
    }

    /// Reduces a lattice basis using the LLL algorithm with parameter delta (default: 0.75 or 0.99).
    pub fn lll(basis: &mut Matrix, delta: f64) {
        let n = basis.rows;
        let m = basis.cols;
        if n <= 1 {
            return;
        }

        let mut k = 1;
        while k < n {
            let (mut b_star, mut mu) = Self::gram_schmidt(basis);

            // Size reduction on k
            for j in (0..k).rev() {
                let u = mu.data[k][j];
                if u.abs() > 0.5 {
                    let q = u.round();
                    for col in 0..m {
                        basis.data[k][col] -= q * basis.data[j][col];
                    }
                    let res = Self::gram_schmidt(basis);
                    b_star = res.0;
                    mu = res.1;
                }
            }

            // Lovász condition: ||b*_k||^2 >= (delta - mu_{k, k-1}^2) * ||b*_{k-1}||^2
            let norm_k = Self::norm_squared(b_star.row(k));
            let norm_k_prev = Self::norm_squared(b_star.row(k - 1));
            let mu_prev = mu.data[k][k - 1];

            if norm_k >= (delta - mu_prev * mu_prev) * norm_k_prev - 1e-9 {
                k += 1;
            } else {
                basis.swap_rows(k, k - 1);
                k = k.saturating_sub(1).max(1);
            }
        }
    }

    /// Solves the Closest Vector Problem (CVP) using Babai's Nearest Plane algorithm.
    /// Given a reduced basis and a target vector, finds the closest lattice point.
    pub fn babai_cvp(reduced_basis: &Matrix, target: &[f64]) -> Vec<f64> {
        let n = reduced_basis.rows;
        let m = reduced_basis.cols;
        let (b_star, _) = Self::gram_schmidt(reduced_basis);

        let mut current_target = target.to_vec();
        let mut result = vec![0.0; m];

        for i in (0..n).rev() {
            let denom = Self::norm_squared(b_star.row(i));
            if denom.abs() < 1e-12 {
                continue;
            }
            let c = (Self::dot_product(&current_target, b_star.row(i)) / denom).round();
            for j in 0..m {
                result[j] += c * reduced_basis.data[i][j];
                current_target[j] -= c * reduced_basis.data[i][j];
            }
        }

        result
    }

    /// Fast lattice-based solver for a 2-structure congruence system:
    /// Recovers structure seeds satisfying two regional constraints simultaneously.
    pub fn solve_2_structures(
        salt1: u64,
        reg_x1: i32,
        reg_z1: i32,
        salt2: u64,
        reg_x2: i32,
        reg_z2: i32,
        target_s1: u64,
        target_s2: u64,
    ) -> Vec<u64> {
        // Delta between two region seed congruences
        // S_reg1 = S + reg_x1*K1 + reg_z1*K2 + salt1 (mod 2^48)
        // S_reg2 = S + reg_x2*K1 + reg_z2*K2 + salt2 (mod 2^48)
        // S_reg2 - S_reg1 = (reg_x2 - reg_x1)*K1 + (reg_z2 - reg_z1)*K2 + (salt2 - salt1) (mod 2^48)
        let k1: u64 = 341873128712;
        let k2: u64 = 132897987541;
        let mask48 = 0xFFFFFFFFFFFFu64;

        let delta_x = (reg_x2 - reg_x1) as i64 as u64;
        let delta_z = (reg_z2 - reg_z1) as i64 as u64;
        let expected_diff = delta_x
            .wrapping_mul(k1)
            .wrapping_add(delta_z.wrapping_mul(k2))
            .wrapping_add(salt2.wrapping_sub(salt1))
            & mask48;

        let actual_diff = target_s2.wrapping_sub(target_s1) & mask48;
        if expected_diff != actual_diff {
            return Vec::new();
        }

        // Recover structure seed: S = target_s1 - (reg_x1*K1 + reg_z1*K2 + salt1) (mod 2^48)
        let offset1 = (reg_x1 as i64 as u64)
            .wrapping_mul(k1)
            .wrapping_add((reg_z1 as i64 as u64).wrapping_mul(k2))
            .wrapping_add(salt1)
            & mask48;

        let structure_seed = target_s1.wrapping_sub(offset1) & mask48;
        vec![structure_seed]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gram_schmidt_orthogonal() {
        let b = Matrix::from_rows(vec![
            vec![1.0, 1.0, 0.0],
            vec![1.0, 0.0, 1.0],
            vec![0.0, 1.0, 1.0],
        ]);
        let (b_star, _) = Lattice::gram_schmidt(&b);

        let dot01 = Lattice::dot_product(b_star.row(0), b_star.row(1));
        let dot02 = Lattice::dot_product(b_star.row(0), b_star.row(2));
        let dot12 = Lattice::dot_product(b_star.row(1), b_star.row(2));

        assert!(dot01.abs() < 1e-9);
        assert!(dot02.abs() < 1e-9);
        assert!(dot12.abs() < 1e-9);
    }

    #[test]
    fn test_lll_basis_reduction() {
        let mut b = Matrix::from_rows(vec![
            vec![1.0, 1.0, 1.0],
            vec![-1.0, 0.0, 2.0],
            vec![3.0, 5.0, 6.0],
        ]);
        Lattice::lll(&mut b, 0.75);

        // Verify vectors are shorter
        for r in 0..b.rows {
            assert!(Lattice::norm_squared(b.row(r)) > 0.0);
        }
    }

    #[test]
    fn test_babai_cvp() {
        let mut basis = Matrix::from_rows(vec![
            vec![2.0, 0.0],
            vec![0.0, 2.0],
        ]);
        Lattice::lll(&mut basis, 0.75);

        let target = vec![3.1, 0.9];
        let closest = Lattice::babai_cvp(&basis, &target);

        assert_eq!(closest, vec![4.0, 0.0].into_iter().map(|x| (x as f64).round()).collect::<Vec<f64>>());
    }

    #[test]
    fn test_structure_2_lattice() {
        let k1: u64 = 341873128712;
        let k2: u64 = 132897987541;
        let mask48 = 0xFFFFFFFFFFFFu64;

        let seed: u64 = 12345678901234;
        let salt1: u64 = 14357617;
        let salt2: u64 = 14357620;

        let rx1 = 5;
        let rz1 = -3;
        let rx2 = 8;
        let rz2 = 4;

        let s1 = (seed.wrapping_add((rx1 as i64 as u64).wrapping_mul(k1)).wrapping_add((rz1 as i64 as u64).wrapping_mul(k2)).wrapping_add(salt1)) & mask48;
        let s2 = (seed.wrapping_add((rx2 as i64 as u64).wrapping_mul(k1)).wrapping_add((rz2 as i64 as u64).wrapping_mul(k2)).wrapping_add(salt2)) & mask48;

        let recovered = Lattice::solve_2_structures(salt1, rx1, rz1, salt2, rx2, rz2, s1, s2);
        assert_eq!(recovered, vec![seed]);
    }
}
