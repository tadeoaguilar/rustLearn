//! Exercise 5: buttons -- debouncing, and passing events out of an interrupt.
//!
//! A mechanical button doesn't switch cleanly: for a few milliseconds the
//! contacts bounce, and a pin sampled every millisecond reads
//! `0 1 0 1 1 0 1 1 1 1`. An *integrating debouncer* counts consistent
//! samples and only changes its idea of the button's state after `N` in a
//! row agree.
//!
//! The sampling usually happens in a timer interrupt, and the application
//! wants the resulting events in its main loop. A single-producer
//! single-consumer queue (`heapless::spsc`) moves them across without locks
//! or a heap: the interrupt holds the `Producer`, the main loop the `Consumer`.

use embedded_hal::digital::InputPin;
use heapless::spsc::{Consumer, Producer, Queue};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Pressed,
    Released,
}

/// A debouncer: the state flips after `threshold` consecutive samples
/// disagree with it.
#[derive(Debug, Clone)]
pub struct Debouncer {
    threshold: u8,
    count: u8,
    pressed: bool,
}

impl Debouncer {
    /// Starts released.
    pub const fn new(threshold: u8) -> Self {
        Debouncer {
            threshold,
            count: 0,
            pressed: false,
        }
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    /// Feed one raw sample (`true` = pressed); returns an edge when the
    /// debounced state changes.
    pub fn update(&mut self, raw_pressed: bool) -> Option<Edge> {
        if raw_pressed == self.pressed {
            self.count = 0;
            return None;
        }
        self.count += 1;
        if self.count < self.threshold.max(1) {
            return None;
        }
        self.count = 0;
        self.pressed = raw_pressed;
        Some(if raw_pressed {
            Edge::Pressed
        } else {
            Edge::Released
        })
    }
}

/// A button on a pin, active-low (pressed connects the pin to ground, an
/// internal pull-up keeps it high otherwise -- the usual wiring).
pub struct Button<P> {
    pin: P,
    debouncer: Debouncer,
}

impl<P: InputPin> Button<P> {
    pub fn new(pin: P, threshold: u8) -> Self {
        Button {
            pin,
            debouncer: Debouncer::new(threshold),
        }
    }

    /// Sample the pin once (call from a 1 kHz timer interrupt).
    pub fn poll(&mut self) -> Result<Option<Edge>, P::Error> {
        let pressed = self.pin.is_low()?;
        Ok(self.debouncer.update(pressed))
    }

    pub fn release(self) -> P {
        self.pin
    }
}

/// Events with the time they happened (ms).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Event {
    pub edge: Edge,
    pub at_ms: u32,
}

/// Holds 15 events (a `Queue<_, N>` holds N - 1).
pub type EventQueue = Queue<Event, 16>;

/// The interrupt side: debounce a sample, and on an edge, enqueue it. If
/// the queue is full the event is dropped and counted -- an interrupt must
/// never block.
pub fn on_tick(
    debouncer: &mut Debouncer,
    raw_pressed: bool,
    now_ms: u32,
    events: &mut Producer<'_, Event>,
    dropped: &mut u32,
) {
    if let Some(edge) = debouncer.update(raw_pressed) {
        if events
            .enqueue(Event {
                edge,
                at_ms: now_ms,
            })
            .is_err()
        {
            *dropped += 1;
        }
    }
}

/// The main-loop side: take every waiting event.
pub fn drain(events: &mut Consumer<'_, Event>) -> heapless::Vec<Event, 16> {
    let mut out = heapless::Vec::new();
    while let Some(event) = events.dequeue() {
        let _ = out.push(event);
    }
    out
}

/// Press durations (ms) from a stream of events: Pressed..Released pairs.
/// A long press is how one-button devices get a second function.
pub fn press_durations(events: &[Event]) -> heapless::Vec<u32, 16> {
    let mut out = heapless::Vec::new();
    let mut pressed_at = None;
    for event in events {
        match event.edge {
            Edge::Pressed => pressed_at = Some(event.at_ms),
            Edge::Released => {
                if let Some(start) = pressed_at.take() {
                    let _ = out.push(event.at_ms.wrapping_sub(start));
                }
            }
        }
    }
    out
}
