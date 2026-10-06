# Answers · 37 WebAssembly

## Exercise 2: `Vec<u8>` versus `cells_ptr`

Returning a `Vec<u8>` means, every frame: allocate a vector in WASM memory,
copy the cells into it, then the glue copies them again into a new JS
`Uint8Array`, and frees the WASM vector -- two copies and two allocations of
the whole grid, 60 times a second, plus garbage for the JS collector. With
`cells_ptr`, JS builds a *view* over WASM's linear memory (no copy at all)
and reads the bytes where Rust left them. For a 64x64 grid it hardly
matters; for a 1000x1000 grid it's the difference between smooth and
stuttering. The catch: the view is invalidated if WASM memory grows (an
allocation can move `memory.buffer`), so create it fresh each frame, as
`main.js` does.

## Exercise 4: Reading outside the preopened directory

It fails with a "not permitted"/"no such file" error. A WASI module has no
ambient authority: it can't name the host's filesystem at all, only the
directories the host passed with `--dir`, which appear to it as its own
roots. Absolute paths and `..` escapes resolve inside those preopens or not
at all. That's *capability-based security*: the program can only touch what
it was explicitly handed -- which is why WASI is attractive for plugins and
untrusted code (and why the same `std::fs` code needs no changes: the
sandboxing happens in the runtime, not the program).
