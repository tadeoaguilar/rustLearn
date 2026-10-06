# 33 · Embedded Rust

## Overview

On a microcontroller there's no operating system, no heap, often no
floating-point unit -- and a bug can't be fixed by restarting a process.
Rust fits well: `#![no_std]` gives you the language with only `core`,
ownership models "who may use this peripheral", and the `embedded-hal`
traits let drivers be written once for every chip. This module writes a
`no_std` library -- LEDs, an I2C sensor driver, a serial framing protocol,
memory-mapped registers, button handling and a PID controller -- and tests
it on your machine with mock hardware.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Blink | `OutputPin`, `DelayNs`; blocking vs a state machine |
| 2 | I2C driver | Registers to values; read-modify-write; mock transcripts |
| 3 | Framing | COBS + CRC-16; resynchronisation; no heap |
| 4 | Registers | Volatile access; bit fields; set/reset registers |
| 5 | Buttons | Debouncing; an interrupt-safe SPSC queue |
| Bonus | PID | Fixed-point control with anti-windup |

## Key Concepts

### `no_std`

```rust
#![no_std]                 // core only: no Vec, String, Box, println!, threads
use heapless::Vec;         // fixed capacity, on the stack or in a static
let mut buf: Vec<u8, 64> = Vec::new();
buf.push(1).ok();          // push can fail: the capacity is all there is
```

### embedded-hal

```rust
pub fn blink<P: OutputPin, D: DelayNs>(led: &mut Led<P>, delay: &mut D, ...)
```

Every chip's HAL implements these traits; so do `embedded-hal-mock`'s pins
and buses, which record expected transactions and fail the test on any
difference. Drivers take their peripheral by value and give it back with
`release()`.

### Volatile

```rust
unsafe { addr_of_mut!((*regs).bit_set_reset).write_volatile(1 << pin) }
```

Without `volatile`, the optimizer may drop a write nobody reads, or merge
two reads of a status register into one.

### Interrupts and queues

An interrupt can fire between any two instructions of the main loop. A
`heapless::spsc::Queue` split into a `Producer` (in the interrupt) and a
`Consumer` (in the main loop) passes data with no lock and no heap.

## Common Pitfalls

1. **Floats on a chip without an FPU** -- software emulation, slow; use fixed point
2. **Blocking delays in the main loop** -- everything else stops
3. **Read-modify-write races with interrupts** -- use set/reset registers or critical sections
4. **Non-volatile register access** -- the optimizer removes it
5. **Ignoring `push` failures on heapless collections** -- decide what to drop
6. **Integer overflow in sensor maths** -- widen before multiplying
7. **Forgetting button bounce** -- one press, five events

## Running This Module

```bash
cargo run  -p m33-embedded-rust -- 1                     # your code (1-5, bonus, all)
cargo test -p m33-embedded-rust-tests --features mine    # test your code
cargo run  -p m33-embedded-rust-solution -- all          # LEDs as ###___, a pretend sensor, frames, a PID loop
cargo test -p m33-embedded-rust-tests                    # 26 tests against the solution
```

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of earlier modules and the exercises in
[the phase README](../README.md) (blink an LED, read a sensor over I2C/SPI,
implement a simple protocol, an embedded web server).

- **No board is needed, and none was used in this repository's checks.**
  The library is `no_std` and builds for a microcontroller target
  (GETTING_STARTED.md shows how); running it on hardware is optional.
- **Not covered**: an embedded web server (it needs a network stack such as
  embassy-net on a Wi-Fi chip), RTOS/Embassy, DMA, defmt and probe-rs
  debugging -- mentioned in the README's resources.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[34 · OS Concepts](../34-os-concepts/)
