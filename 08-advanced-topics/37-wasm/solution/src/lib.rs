//! Module 37 -- WebAssembly. Reference solution.
//!
//! A library meant to be compiled to WASM (`wasm-pack build --target web`)
//! and called from JavaScript -- tested natively, because `#[wasm_bindgen]`
//! functions are ordinary Rust functions when compiled for your machine.
//!
//! | File               | Exercise |
//! |--------------------|----------|
//! | `ex01_text.rs`     | 1  exported functions: text utilities, a Markdown subset |
//! | `ex02_life.rs`     | 2  an exported struct: Game of Life, cells read in place by JS |
//! | `ex03_interop.rs`  | 3  the boundary: slices, `Vec`s, getters, `Result` errors |
//! | `ex04_wasi.rs`     | 4  WASI: file processing that runs natively and sandboxed |
//! | `bonus_image.rs`   | bonus: image filters for a canvas |

pub mod bonus_image;
pub mod ex01_text;
pub mod ex02_life;
pub mod ex03_interop;
pub mod ex04_wasi;
