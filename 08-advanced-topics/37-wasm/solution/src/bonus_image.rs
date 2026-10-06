//! Bonus: image filters for a `<canvas>`.
//!
//! `ctx.getImageData()` gives an RGBA byte array; pass it to WASM, filter it
//! in place, `putImageData` it back. Pixel loops are where WASM beats plain
//! JavaScript most clearly.

use wasm_bindgen::prelude::*;

/// Grayscale in place (Rec. 601 luma, integer maths); alpha untouched.
#[wasm_bindgen]
pub fn grayscale(rgba: &mut [u8]) {
    for px in rgba.chunks_exact_mut(4) {
        let y = ((px[0] as u32 * 299 + px[1] as u32 * 587 + px[2] as u32 * 114 + 500) / 1000) as u8;
        px[0] = y;
        px[1] = y;
        px[2] = y;
    }
}

#[wasm_bindgen]
pub fn invert(rgba: &mut [u8]) {
    for px in rgba.chunks_exact_mut(4) {
        px[0] = 255 - px[0];
        px[1] = 255 - px[1];
        px[2] = 255 - px[2];
    }
}

/// A 3x3 box blur (edge pixels average the neighbours that exist); a new image.
#[wasm_bindgen]
pub fn box_blur(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    let (w, h) = (width as usize, height as usize);
    assert_eq!(
        rgba.len(),
        w * h * 4,
        "the buffer must be width * height * 4 bytes"
    );
    let mut out = vec![0u8; rgba.len()];
    for y in 0..h {
        for x in 0..w {
            let mut sum = [0u32; 4];
            let mut n = 0;
            for ny in y.saturating_sub(1)..=(y + 1).min(h - 1) {
                for nx in x.saturating_sub(1)..=(x + 1).min(w - 1) {
                    let i = (ny * w + nx) * 4;
                    for k in 0..4 {
                        sum[k] += rgba[i + k] as u32;
                    }
                    n += 1;
                }
            }
            let i = (y * w + x) * 4;
            for k in 0..4 {
                out[i + k] = ((sum[k] + n / 2) / n) as u8;
            }
        }
    }
    out
}
