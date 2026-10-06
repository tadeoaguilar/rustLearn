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
        todo!("Exercise 3")
    }
}

/// i, j, k: the inner loop strides down B's column.
pub fn multiply_naive(a: &Matrix, b: &Matrix) -> Matrix {
    todo!("Exercise 3")
}

/// i, k, j: the inner loop walks a row of B and a row of C.
pub fn multiply_ikj(a: &Matrix, b: &Matrix) -> Matrix {
    todo!("Exercise 3")
}

/// Transpose B first; then both inner operands are rows (a dot product).
pub fn multiply_transposed(a: &Matrix, b: &Matrix) -> Matrix {
    todo!("Exercise 3")
}

/// Tiled `i, k, j` over `tile x tile` blocks (any `n`; edge tiles are partial).
pub fn multiply_blocked(a: &Matrix, b: &Matrix, tile: usize) -> Matrix {
    todo!("Exercise 3")
}
