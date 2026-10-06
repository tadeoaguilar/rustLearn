//! Exercise 3: cache-friendly matrix multiplication.
//!
//! `C = A x B` for n x n matrices is 2n^3 operations whatever the loop order
//! -- but the order decides how memory is walked. The textbook `i, j, k`
//! order reads B down a *column*: a cache miss per element once rows are
//! bigger than the cache. Swapping the loops to `i, k, j` walks B along rows;
//! transposing B first does the same; *blocking* (tiling) works on tiles
//! small enough to stay in cache while they're reused.
//!
//! Integer matrices keep every version's result exactly equal.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Matrix {
    pub n: usize,
    /// Row-major: element (r, c) at `r * n + c`.
    pub data: Vec<i64>,
}

impl Matrix {
    pub fn zeros(n: usize) -> Self {
        Matrix {
            n,
            data: vec![0; n * n],
        }
    }

    /// Deterministic small values, so products don't overflow.
    pub fn sample(n: usize, seed: i64) -> Self {
        Matrix {
            n,
            data: (0..(n * n) as i64)
                .map(|i| (i * 31 + seed * 17) % 19 - 9)
                .collect(),
        }
    }

    pub fn get(&self, r: usize, c: usize) -> i64 {
        self.data[r * self.n + c]
    }

    pub fn transposed(&self) -> Matrix {
        let n = self.n;
        let mut t = Matrix::zeros(n);
        for r in 0..n {
            for c in 0..n {
                t.data[c * n + r] = self.data[r * n + c];
            }
        }
        t
    }
}

/// i, j, k: the inner loop strides down B's column.
pub fn multiply_naive(a: &Matrix, b: &Matrix) -> Matrix {
    let n = a.n;
    let mut c = Matrix::zeros(n);
    for i in 0..n {
        for j in 0..n {
            let mut sum = 0;
            for k in 0..n {
                sum += a.data[i * n + k] * b.data[k * n + j];
            }
            c.data[i * n + j] = sum;
        }
    }
    c
}

/// i, k, j: the inner loop walks a row of B and a row of C.
pub fn multiply_ikj(a: &Matrix, b: &Matrix) -> Matrix {
    let n = a.n;
    let mut c = Matrix::zeros(n);
    for i in 0..n {
        let c_row = &mut c.data[i * n..(i + 1) * n];
        for k in 0..n {
            let aik = a.data[i * n + k];
            let b_row = &b.data[k * n..(k + 1) * n];
            for (cij, bkj) in c_row.iter_mut().zip(b_row) {
                *cij += aik * bkj;
            }
        }
    }
    c
}

/// Transpose B first; then both inner operands are rows (a dot product).
pub fn multiply_transposed(a: &Matrix, b: &Matrix) -> Matrix {
    let n = a.n;
    let bt = b.transposed();
    let mut c = Matrix::zeros(n);
    for i in 0..n {
        let a_row = &a.data[i * n..(i + 1) * n];
        for j in 0..n {
            let bt_row = &bt.data[j * n..(j + 1) * n];
            c.data[i * n + j] = a_row.iter().zip(bt_row).map(|(x, y)| x * y).sum();
        }
    }
    c
}

/// Tiled `i, k, j` over `tile x tile` blocks (any `n`; edge tiles are partial).
pub fn multiply_blocked(a: &Matrix, b: &Matrix, tile: usize) -> Matrix {
    let n = a.n;
    let tile = tile.max(1);
    let mut c = Matrix::zeros(n);
    for ii in (0..n).step_by(tile) {
        for kk in (0..n).step_by(tile) {
            for jj in (0..n).step_by(tile) {
                for i in ii..(ii + tile).min(n) {
                    for k in kk..(kk + tile).min(n) {
                        let aik = a.data[i * n + k];
                        let j_end = (jj + tile).min(n);
                        let b_row = &b.data[k * n + jj..k * n + j_end];
                        let c_row = &mut c.data[i * n + jj..i * n + j_end];
                        for (cij, bkj) in c_row.iter_mut().zip(b_row) {
                            *cij += aik * bkj;
                        }
                    }
                }
            }
        }
    }
    c
}
