//! Bonus: image filters for a `<canvas>`.
//!
//! `ctx.getImageData()` gives an RGBA byte array; pass it to WASM, filter it
//! in place, `putImageData` it back. Pixel loops are where WASM beats plain
//! JavaScript most clearly.

use wasm_bindgen::prelude::*;

/// Grayscale in place (Rec. 601 luma, integer maths); alpha untouched.
#[wasm_bindgen]
pub fn grayscale(rgba: &mut [u8]) {
    todo!("Bonus")
}

#[wasm_bindgen]
pub fn invert(rgba: &mut [u8]) {
    todo!("Bonus")
}

/// A 3x3 box blur (edge pixels average the neighbours that exist); a new image.
#[wasm_bindgen]
pub fn box_blur(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    todo!("Bonus")
}
