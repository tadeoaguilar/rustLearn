# 37 · WebAssembly

## Overview

WebAssembly lets Rust run in the browser next to JavaScript, at
near-native speed, and in sandboxed runtimes (wasmtime, wasmer, edge
platforms) through WASI. Rust is the best-supported language for it: no
garbage collector to ship, small binaries, and `wasm-bindgen` to generate
the JS glue. This module writes a WASM library -- text utilities, a Game of
Life whose memory JS reads directly, typed values across the boundary, a
WASI file tool and image filters -- and tests all of it natively.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Exported functions | `#[wasm_bindgen]`; string copies; HTML safety |
| 2 | Exported structs | JS classes; reading WASM memory without copying |
| 3 | The boundary | Slices, `Vec`s, getters, `Result` as exceptions |
| 4 | WASI | The same `std::fs` code, natively and sandboxed |
| Bonus | Pixels | Canvas `ImageData` filters |

## Key Concepts

### What `wasm-bindgen` does

```rust
#[wasm_bindgen]
pub fn slugify(text: &str) -> String { ... }
```

```js
import init, { slugify } from "./pkg/m37_wasm_solution.js";
await init();
slugify("Hello, World");   // "hello-world"
```

The generated glue encodes the JS string as UTF-8 into WASM memory, calls
the function, decodes the result. Numbers pass directly; strings, slices
and `Vec`s are copied; exported structs stay in WASM memory behind a handle.

### Targets

| Target | For | Tooling |
|---|---|---|
| `wasm32-unknown-unknown` | browsers, Node | `wasm-bindgen` / `wasm-pack` |
| `wasm32-wasip1` | wasmtime, wasmer, servers | plain `std`, `cargo build --target` |

### Size

`opt-level = "z"`, `lto = true`, `wasm-opt -Oz` (from binaryen; wasm-pack
runs it), and avoiding heavy formatting keep modules small -- module 36's
`min-size` profile applies here too.

## Common Pitfalls

1. **Calling JS-backed APIs natively** -- `JsValue`, `web_sys`, `js_sys` panic outside wasm32; keep logic in plain Rust
2. **Passing big data back and forth** -- each crossing copies; keep state in WASM
3. **Holding a memory view across allocations** -- WASM memory can grow and move
4. **`innerHTML` with unescaped output** -- XSS; escape in Rust or use `textContent`
5. **Panics in WASM** -- an `unreachable` trap with no message; add `console_error_panic_hook`
6. **Threads and blocking** -- the browser's main thread can't block; WASI preview 1 has no threads
7. **Absolute paths in WASI** -- only preopened directories exist

## Running This Module

```bash
cargo run  -p m37-wasm -- 1                     # your code (1-4, bonus, all)
cargo test -p m37-wasm-tests --features mine    # test your code
cargo run  -p m37-wasm-solution -- all          # the library, natively
cargo test -p m37-wasm-tests                    # 13 tests against the solution
```

The browser demo (`web/`) and the WASI build are optional -- see
[GETTING_STARTED.md](GETTING_STARTED.md).

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of earlier modules and the exercises in
[the phase README](../README.md) (a WASM library for JavaScript, a web app
with Yew, optimizing WASM size, WASI file operations).

- **The WASM builds were not run in this repository's checks**: no wasm32
  target, wasm-pack or wasmtime was installed on the machine it was built
  on. The library compiles and is tested natively; `web/index.html` and
  `web/main.js` follow wasm-pack's `--target web` output.
- **Yew isn't used**: a framework would add a large dependency tree to every
  build; Exercise 2 is the same model (Rust state, rendered by the page)
  with plain JS. Yew/Leptos are pointers in the README's resources.
- **Size optimization** is covered by module 36's profile and the notes
  above, without a test (it needs the wasm target).

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[38 · Procedural Macros](../38-proc-macros/)
