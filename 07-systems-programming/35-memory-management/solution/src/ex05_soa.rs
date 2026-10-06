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
    let mut seed: u32 = 12345;
    let mut next = || {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (seed >> 8) as f32 / (1 << 24) as f32
    };
    let particles = (0..n)
        .map(|i| Particle {
            position: [next(), next(), next()],
            velocity: [next() - 0.5, next() - 0.5, next() - 0.5],
            mass: 1.0 + next(),
            alive: i % 7 != 0,
        })
        .collect();
    ParticlesAos { particles }
}

impl ParticlesAos {
    /// Move every live particle by `velocity * dt`.
    pub fn step(&mut self, dt: f32) {
        for p in self.particles.iter_mut().filter(|p| p.alive) {
            for k in 0..3 {
                p.position[k] += p.velocity[k] * dt;
            }
        }
    }

    /// Sum of ½·m·v² over live particles.
    pub fn kinetic_energy(&self) -> f64 {
        self.particles
            .iter()
            .filter(|p| p.alive)
            .map(|p| {
                0.5 * p.mass as f64 * p.velocity.iter().map(|v| (*v as f64).powi(2)).sum::<f64>()
            })
            .sum()
    }

    /// Mean x of live particles -- touching one field, but loading whole particles.
    pub fn mean_x(&self) -> f64 {
        let (sum, n) = self
            .particles
            .iter()
            .filter(|p| p.alive)
            .fold((0.0, 0usize), |(s, n), p| (s + p.position[0] as f64, n + 1));
        if n == 0 { 0.0 } else { sum / n as f64 }
    }

    pub fn to_soa(&self) -> ParticlesSoa {
        let mut soa = ParticlesSoa::default();
        for p in &self.particles {
            soa.x.push(p.position[0]);
            soa.y.push(p.position[1]);
            soa.z.push(p.position[2]);
            soa.vx.push(p.velocity[0]);
            soa.vy.push(p.velocity[1]);
            soa.vz.push(p.velocity[2]);
            soa.mass.push(p.mass);
            soa.alive.push(p.alive);
        }
        soa
    }
}

impl ParticlesSoa {
    pub fn len(&self) -> usize {
        self.x.len()
    }

    pub fn is_empty(&self) -> bool {
        self.x.is_empty()
    }

    pub fn step(&mut self, dt: f32) {
        // Zipped iterators: no bounds checks, and the compiler can vectorize.
        let axes = [
            (&mut self.x, &self.vx),
            (&mut self.y, &self.vy),
            (&mut self.z, &self.vz),
        ];
        for (positions, velocities) in axes {
            for ((p, v), alive) in positions.iter_mut().zip(velocities).zip(&self.alive) {
                if *alive {
                    *p += v * dt;
                }
            }
        }
    }

    /// Mean x of live particles: reads 5 bytes per particle, not 32.
    pub fn mean_x(&self) -> f64 {
        let (sum, n) = self
            .x
            .iter()
            .zip(&self.alive)
            .filter(|(_, a)| **a)
            .fold((0.0, 0usize), |(s, n), (x, _)| (s + *x as f64, n + 1));
        if n == 0 { 0.0 } else { sum / n as f64 }
    }

    pub fn kinetic_energy(&self) -> f64 {
        (0..self.len())
            .filter(|&i| self.alive[i])
            .map(|i| {
                let v2 = (self.vx[i] as f64).powi(2)
                    + (self.vy[i] as f64).powi(2)
                    + (self.vz[i] as f64).powi(2);
                0.5 * self.mass[i] as f64 * v2
            })
            .sum()
    }

    pub fn to_aos(&self) -> ParticlesAos {
        let particles = (0..self.len())
            .map(|i| Particle {
                position: [self.x[i], self.y[i], self.z[i]],
                velocity: [self.vx[i], self.vy[i], self.vz[i]],
                mass: self.mass[i],
                alive: self.alive[i],
            })
            .collect();
        ParticlesAos { particles }
    }
}

/// Sum a row-major `rows x cols` matrix row by row (the memory order).
pub fn sum_row_major(data: &[u32], rows: usize, cols: usize) -> u64 {
    let mut sum = 0u64;
    for r in 0..rows {
        for c in 0..cols {
            sum += data[r * cols + c] as u64;
        }
    }
    sum
}

/// The same sum, column by column: a stride of `cols` between accesses.
pub fn sum_col_major(data: &[u32], rows: usize, cols: usize) -> u64 {
    let mut sum = 0u64;
    for c in 0..cols {
        for r in 0..rows {
            sum += data[r * cols + c] as u64;
        }
    }
    sum
}
