# Getting Started with 33 · Embedded Rust

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m33-embedded-rust-solution -- all
```

blinks a printed LED (`#_#_#_`) and an SOS pattern, reads a pretend TMP102,
frames a message (and rejects garbage and a corrupted copy), drives GPIO
registers, debounces a bouncing button and runs a PID loop on a toy heater.

## What Is Already Here

```
33-embedded-rust/
├── README.md, exercises.md, GETTING_STARTED.md
├── exercise/src/              # ← YOUR WORKSPACE (package m33-embedded-rust, #![no_std] lib)
│   ├── ex01_blink.rs          #   Ex 1
│   ├── ex02_sensor.rs         #   Ex 2
│   ├── ex03_framing.rs        #   Ex 3
│   ├── ex04_registers.rs      #   Ex 4  (simulate_hardware provided)
│   ├── ex05_input.rs          #   Ex 5
│   ├── bonus_pid.rs           #   bonus
│   └── main.rs                #   provided: host demo with fake hardware
├── solution/                  # ← REFERENCE (m33-embedded-rust-solution) + ANSWERS.md
└── tests/                     # ← 26 tests with embedded-hal-mock (m33-embedded-rust-tests)
```

## The Commands You Need

```bash
cargo test -p m33-embedded-rust-tests --features mine ex3_
cargo test -p m33-embedded-rust-tests --features mine
cargo test -p m33-embedded-rust-tests                    # the solution: always green
```

## Building for a Microcontroller (Optional, Not Run in This Repository's Checks)

The library compiles for bare-metal targets as is:

```bash
rustup target add thumbv7em-none-eabihf            # Cortex-M4F/M7F (STM32F4, nRF52, ...)
cargo build -p m33-embedded-rust-solution --lib --target thumbv7em-none-eabihf
```

To run on a board you'd add a binary crate with `cortex-m-rt`, a panic
handler and your chip's HAL (`stm32f4xx-hal`, `rp2040-hal`, `esp-hal`),
implement the pins with the HAL's types, and flash with `probe-rs run`.
The Embedded Rust Book and the HAL's examples walk through it.

## If You Get Stuck

1. **"A mock was dropped without calling `.done()`"** -- call `release()` and `.done()` on the mock at the end; the panic means a test path skipped it.
2. **The mock reports "unexpected transaction"** -- your driver's bus traffic differs from the transcript in the test: compare byte by byte.
3. **Negative temperatures wrong** -- shift the `i16`, not a `u16`, so the sign is kept.
4. **COBS round trip fails at 254/255 bytes** -- handle the full-run case (code `0xFF`, no implied zero).
5. **`use std::...` doesn't compile** -- the crate is `no_std`; use `core::` and `heapless`.
6. **Queue holds one less than its size** -- `Queue<T, 16>` stores 15.
7. Compare with `solution/src/` -- same file and function names.
