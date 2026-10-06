//! Exercise 1: blinking an LED -- two ways.
//!
//! `embedded-hal` defines traits (`OutputPin`, `DelayNs`, `I2c`, ...) that
//! every microcontroller's HAL implements, so a driver written against them
//! runs on an STM32, an RP2040 or an ESP32 unchanged -- and in tests against
//! mocks. The first blinker blocks in `delay_ms`; the second is a state
//! machine you `tick` from a main loop, so the CPU can do other work (or
//! sleep) between transitions.

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{OutputPin, PinState};

/// An LED that may be wired active-low (on when the pin is low), as many
/// board LEDs are. Callers think in on/off, never in pin levels.
pub struct Led<P> {
    pin: P,
    active_low: bool,
}

impl<P: OutputPin> Led<P> {
    pub fn new(pin: P, active_low: bool) -> Self {
        Led { pin, active_low }
    }

    pub fn set(&mut self, on: bool) -> Result<(), P::Error> {
        self.pin.set_state(PinState::from(on != self.active_low))
    }

    pub fn on(&mut self) -> Result<(), P::Error> {
        self.set(true)
    }

    pub fn off(&mut self) -> Result<(), P::Error> {
        self.set(false)
    }

    /// Give the pin back (drivers should never swallow their peripherals).
    pub fn release(self) -> P {
        self.pin
    }
}

/// Blink `times` times, blocking: on for `on_ms`, off for `off_ms`.
pub fn blink<P: OutputPin, D: DelayNs>(
    led: &mut Led<P>,
    delay: &mut D,
    times: u32,
    on_ms: u32,
    off_ms: u32,
) -> Result<(), P::Error> {
    for _ in 0..times {
        led.on()?;
        delay.delay_ms(on_ms);
        led.off()?;
        delay.delay_ms(off_ms);
    }
    Ok(())
}

/// A repeating pattern of `(on, duration_ms)` steps, advanced by `tick`.
pub struct Pattern<'a> {
    steps: &'a [(bool, u32)],
    index: usize,
    step_started: u32,
    started: bool,
}

/// SOS in Morse: dot = 1 unit on, dash = 3, gaps of 1 inside a letter, 3
/// between letters, 7 before repeating. Unit = 100 ms.
pub const SOS: [(bool, u32); 18] = [
    (true, 100),
    (false, 100),
    (true, 100),
    (false, 100),
    (true, 100),
    (false, 300),
    (true, 300),
    (false, 100),
    (true, 300),
    (false, 100),
    (true, 300),
    (false, 300),
    (true, 100),
    (false, 100),
    (true, 100),
    (false, 100),
    (true, 100),
    (false, 700),
];

impl<'a> Pattern<'a> {
    pub fn new(steps: &'a [(bool, u32)]) -> Self {
        Pattern {
            steps,
            index: 0,
            step_started: 0,
            started: false,
        }
    }

    /// Call with the current time (ms, from any monotonic clock, wrapping).
    /// Returns `Some(on)` when the LED should change -- on the first call,
    /// and each time a step's duration has passed -- else `None`.
    pub fn tick(&mut self, now_ms: u32) -> Option<bool> {
        if self.steps.is_empty() {
            return None;
        }
        if !self.started {
            self.started = true;
            self.step_started = now_ms;
            return Some(self.steps[0].0);
        }
        let (_, duration) = self.steps[self.index];
        if now_ms.wrapping_sub(self.step_started) < duration {
            return None;
        }
        self.step_started = self.step_started.wrapping_add(duration);
        self.index = (self.index + 1) % self.steps.len();
        Some(self.steps[self.index].0)
    }

    /// The total length of one repetition.
    pub fn period_ms(&self) -> u32 {
        self.steps.iter().map(|(_, d)| d).sum()
    }
}
