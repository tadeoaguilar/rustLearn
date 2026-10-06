# Getting Started with 37 · WebAssembly

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m37-wasm-solution -- all
```

runs the library natively: text utilities and Markdown (with a `<script>`
tag safely escaped), a glider crossing a Game of Life grid, statistics and
validation with `Result` errors, a `wc`-style report over a directory, and
image filters.

## What Is Already Here

```
37-wasm/
├── README.md, exercises.md, GETTING_STARTED.md
├── web/                       # optional browser demo: index.html, main.js
├── exercise/src/              # ← YOUR WORKSPACE (package m37-wasm; crate-type cdylib + rlib)
│   ├── ex01_text.rs           #   Ex 1
│   ├── ex02_life.rs           #   Ex 2  (new, getters, parse, render provided)
│   ├── ex03_interop.rs        #   Ex 3
│   ├── ex04_wasi.rs           #   Ex 4
│   └── bonus_image.rs         #   bonus
├── solution/                  # ← REFERENCE (m37-wasm-solution) + ANSWERS.md
└── tests/                     # ← 13 tests (m37-wasm-tests)
```

## The Commands You Need

```bash
cargo test -p m37-wasm-tests --features mine ex2_
cargo test -p m37-wasm-tests --features mine
cargo test -p m37-wasm-tests                    # the solution: always green
```

## In the Browser (Optional, Not Run in This Repository's Checks)

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build 08-advanced-topics/37-wasm/solution --target web --out-dir ../web/pkg
python3 -m http.server -d 08-advanced-topics/37-wasm/web 8000      # open http://localhost:8000
```

Type Markdown and watch Rust render it; the Game of Life canvas reads cells
straight from WASM memory. (To use your own crate, build `exercise` and
change the import in `main.js` to `./pkg/m37_wasm.js`.)

## With WASI (Optional, Not Run in This Repository's Checks)

```bash
rustup target add wasm32-wasip1
cargo build --target wasm32-wasip1 -p m37-wasm-solution --bin m37-wasm-solution
# install wasmtime: https://wasmtime.dev
mkdir -p /tmp/wasi-data && echo "hello wasi" > /tmp/wasi-data/a.txt
wasmtime run --dir=/tmp/wasi-data::/data target/wasm32-wasip1/debug/m37-wasm-solution.wasm wc /data
```

The module sees only `/data`; try `wc /etc` to see the sandbox refuse.

## If You Get Stuck

1. **Markdown output differs by a newline** -- every block element ends with `\n`, and a list still open at the end must be closed.
2. **Nested `**bold *em***`** -- find the closing marker for the outer one, then format its inside recursively (but not inside `code`).
3. **Glider test fails after many ticks** -- compute the next generation from the *old* grid (a copy), not in place.
4. **Neighbour count wrong at edges** -- add `height - 1` instead of subtracting 1, then `%`: `u32` can't go negative.
5. **Median off for even counts** -- average the two middle values of the *sorted* data.
6. Compare with `solution/src/` -- same file and function names.
