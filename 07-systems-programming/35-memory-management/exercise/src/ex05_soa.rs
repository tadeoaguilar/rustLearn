//! Exercise 5: cache-friendly data -- array of structs vs struct of arrays.
//!
//! Memory is fetched in 64-byte cache lines. A loop that touches one field
//! of every particle drags the *whole* particle through the cache when
//! particles are stored as an array of structs (AoS); stored as a struct of
//! arrays (SoA), each field is contiguous, and every byte fetched is used.
//! Same for 2D arrays: walk them in the order they're laid out.
//!
//! The functions compute identical results either way; the demo times them
//! (`cargo run --release ... -- 5`).

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Particle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub mass: f32,
    pub alive: bool,
}

/// Array of structs.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParticlesAos {
    pub particles: Vec<Particle>,
}

/// Struct of arrays: one vector per field.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParticlesSoa {
    pub x: Vec<f32>,
    pub y: Vec<f32>,
    pub z: Vec<f32>,
    pub vx: Vec<f32>,
    pub vy: Vec<f32>,
    pub vz: Vec<f32>,
    pub mass: Vec<f32>,
    pub alive: Vec<bool>,
}

/// `n` deterministic particles (a simple LCG, no `rand` needed).
pub fn generate(n: usize) -> ParticlesAos {
    todo!("Exercise 5")
}

impl ParticlesAos {
    /// Move every live particle by `velocity * dt`.
    pub fn step(&mut self, dt: f32) {
        todo!("Exercise 5")
    }

    /// Sum of ½·m·v² over live particles.
    pub fn kinetic_energy(&self) -> f64 {
        todo!("Exercise 5")
    }

    /// Mean x of live particles -- touching one field, but loading whole particles.
    pub fn mean_x(&self) -> f64 {
        todo!("Exercise 5")
    }

    pub fn to_soa(&self) -> ParticlesSoa {
        todo!("Exercise 5")
    }
}

impl ParticlesSoa {
    pub fn len(&self) -> usize {
        todo!("Exercise 5")
    }

    pub fn is_empty(&self) -> bool {
        todo!("Exercise 5")
    }

    pub fn step(&mut self, dt: f32) {
        todo!("Exercise 5")
    }

    /// Mean x of live particles: reads 5 bytes per particle, not 32.
    pub fn mean_x(&self) -> f64 {
        todo!("Exercise 5")
    }

    pub fn kinetic_energy(&self) -> f64 {
        todo!("Exercise 5")
    }

    pub fn to_aos(&self) -> ParticlesAos {
        todo!("Exercise 5")
    }
}

/// Sum a row-major `rows x cols` matrix row by row (the memory order).
pub fn sum_row_major(data: &[u32], rows: usize, cols: usize) -> u64 {
    todo!("Exercise 5")
}

/// The same sum, column by column: a stride of `cols` between accesses.
pub fn sum_col_major(data: &[u32], rows: usize, cols: usize) -> u64 {
    todo!("Exercise 5")
}
