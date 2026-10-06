//! Exercise 2: Conway's Game of Life -- state that lives in WASM memory.
//!
//! The classic Rust-and-WebAssembly design: the `Universe` lives in WASM
//! linear memory; JavaScript calls `tick()` and then reads the cells
//! *directly* from that memory through `cells_ptr()` (a `Uint8Array` view
//! over `memory.buffer`) to draw them -- no copying, no serialization per
//! frame. Exported structs become JS classes; their methods, JS methods.
//!
//! The grid wraps around at the edges (a torus).

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Universe {
    width: u32,
    height: u32,
    /// Row-major; 1 = alive, 0 = dead.
    cells: Vec<u8>,
}

#[wasm_bindgen]
impl Universe {
    /// An empty universe.
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> Universe {
        Universe {
            width,
            height,
            cells: vec![0; (width * height) as usize],
        }
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }

    fn index(&self, row: u32, col: u32) -> usize {
        (row * self.width + col) as usize
    }

    pub fn is_alive(&self, row: u32, col: u32) -> bool {
        todo!("Exercise 2")
    }

    pub fn set(&mut self, row: u32, col: u32, alive: bool) {
        todo!("Exercise 2")
    }

    pub fn toggle(&mut self, row: u32, col: u32) {
        todo!("Exercise 2")
    }

    /// Live neighbours of a cell, wrapping around the edges.
    pub fn live_neighbours(&self, row: u32, col: u32) -> u8 {
        todo!("Exercise 2")
    }

    /// One generation: a live cell with 2 or 3 live neighbours lives on; a
    /// dead cell with exactly 3 becomes alive; everything else dies.
    pub fn tick(&mut self) {
        todo!("Exercise 2")
    }

    pub fn population(&self) -> u32 {
        todo!("Exercise 2")
    }

    /// Where the cells start in WASM memory, for JavaScript to read them in
    /// place: `new Uint8Array(memory.buffer, universe.cells_ptr(), w * h)`.
    pub fn cells_ptr(&self) -> *const u8 {
        todo!("Exercise 2")
    }

    /// `#` for alive, `.` for dead, one line per row.
    pub fn render(&self) -> String {
        let mut out = String::with_capacity(((self.width + 1) * self.height) as usize);
        for row in self.cells.chunks(self.width as usize) {
            out.extend(row.iter().map(|&c| if c == 1 { '#' } else { '.' }));
            out.push('\n');
        }
        out
    }

    /// Parse `render`'s format (`#`/`.` lines, all the same length).
    pub fn parse(text: &str) -> Result<Universe, String> {
        let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
        let width = lines.first().map_or(0, |l| l.len());
        if width == 0 || lines.iter().any(|l| l.len() != width) {
            return Err("rows must be non-empty and all the same length".into());
        }
        let mut universe = Universe::new(width as u32, lines.len() as u32);
        for (r, line) in lines.iter().enumerate() {
            for (c, ch) in line.chars().enumerate() {
                match ch {
                    '#' => universe.set(r as u32, c as u32, true),
                    '.' => {}
                    other => return Err(format!("unexpected {other:?}")),
                }
            }
        }
        Ok(universe)
    }
}
