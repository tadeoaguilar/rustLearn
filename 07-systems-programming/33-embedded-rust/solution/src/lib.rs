//! Module 33 -- Embedded Rust. Reference solution.
//!
//! A `#![no_std]` library: no heap, no OS, nothing but `core` -- it would
//! build for a microcontroller as is. Hardware access goes through the
//! `embedded-hal` traits, so the tests run it on your machine with mocks.
//!
//! | File               | Exercise |
//! |--------------------|----------|
//! | `ex01_blink.rs`    | 1  an LED: blocking blink, and a non-blocking pattern |
//! | `ex02_sensor.rs`   | 2  an I2C driver for the TMP102 temperature sensor |
//! | `ex03_framing.rs`  | 3  a serial protocol: COBS framing and CRC-16 |
//! | `ex04_registers.rs`| 4  memory-mapped registers: volatile access, bit fields |
//! | `ex05_input.rs`    | 5  debouncing, and an interrupt-safe event queue |
//! | `bonus_pid.rs`     | bonus: a fixed-point PID controller |

#![no_std]

pub mod bonus_pid;
pub mod ex01_blink;
pub mod ex02_sensor;
pub mod ex03_framing;
pub mod ex04_registers;
pub mod ex05_input;
