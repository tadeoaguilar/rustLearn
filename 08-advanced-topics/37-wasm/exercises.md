# Exercises: WebAssembly

WebAssembly (WASM) is a portable bytecode that browsers -- and runtimes
like wasmtime -- execute at near-native speed in a sandbox. Rust compiles to
it directly (`--target wasm32-unknown-unknown` for the web,
`wasm32-wasip1` for WASI), and `wasm-bindgen` generates the JavaScript glue.

The trick this module uses: `#[wasm_bindgen]` functions and structs are
ordinary Rust when compiled for your machine, so all the logic is tested
natively with `cargo test`. Building the `.wasm` and running it in a
browser or wasmtime is optional (GETTING_STARTED.md).

**Setup**: `wasm-bindgen` only. No wasm target, wasm-pack or browser is
needed for the tests.

---

## Exercise 1: A Library for JavaScript

**Difficulty**: Medium
**Time**: 1.5 hours

**Learning Objectives**:
- Export functions with `#[wasm_bindgen]`
- Know what each call costs (strings are copied both ways)
- Never let input inject markup

1. `word_count`, `slugify` (lowercase ASCII alphanumerics, other runs ->
   one `-`, none at the ends).
2. `escape_html` (`& < > " '`).
3. `markdown_to_html`: `#`-`######` headings (`# ` with a space),
   consecutive `- ` items in one `<ul>`, paragraphs (lines joined with a
   space, split by blank lines), inline `**strong**`, `*em*`, `` `code` ``
   (no markup inside code; unmatched markers stay). Escape everything first.

---

## Exercise 2: Game of Life

**Difficulty**: Medium
**Time**: 1 hour

**Learning Objectives**:
- Export a struct as a JS class
- Share memory with JS instead of copying

`new`, the getters, `index`, `render` and `parse` are provided.

1. `is_alive`, `set`, `toggle` (coordinates wrap).
2. `live_neighbours` (the 8 around, wrapping at the edges).
3. `tick`: the rules (2-3 neighbours survive, exactly 3 is born).
4. `population`, `cells_ptr` -- JS reads the cells in place:
   `new Uint8Array(memory.buffer, universe.cells_ptr(), w * h)`.

**Question**: what would `cells(&self) -> Vec<u8>` cost per frame in the
browser, compared with `cells_ptr`?

---

## Exercise 3: The JS Boundary

**Difficulty**: Easy
**Time**: 45 minutes

**Learning Objectives**:
- Pass slices and return vectors
- Report errors as JS exceptions with `Result`
- Keep state in WASM with getters

1. `stats(&[f64]) -> Result<Stats, String>`: count, mean, median, min,
   max, population standard deviation; errors for empty input and NaN.
2. `normalize` to 0..=1 (constant input -> zeros).
3. `Signup::new(email, age)`: an email with one `@`, a non-empty local
   part and a dotted domain (not starting or ending with `.`); age 13-120;
   stored lowercased. Getters `email`, `age`; method `domain`.

---

## Exercise 4: WASI File Processing

**Difficulty**: Easy
**Time**: 45 minutes

**Learning Objectives**:
- Write code that runs both natively and in the WASI sandbox

1. `stats_of(name, text)`: lines, words, bytes.
2. `scan_dir(dir, extension)`: regular files only, skip non-UTF-8, sorted.
3. `write_report`: `wc`-style lines and a total.

Optionally run it sandboxed: build for `wasm32-wasip1` and give wasmtime
access to one directory (GETTING_STARTED.md). What happens if the program
tries to read a file outside it?

---

## Bonus: Image Filters

**Difficulty**: Easy
**Time**: 30 minutes

`grayscale` and `invert` in place on RGBA bytes (alpha untouched),
`box_blur` returning a new image (edges average the neighbours that exist).
