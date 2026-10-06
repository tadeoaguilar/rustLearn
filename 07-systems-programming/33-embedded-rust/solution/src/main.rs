// Reference solution for 33-embedded-rust.
//
//     cargo run -p m33-embedded-rust-solution -- <1-5|bonus|all>
//
// The library is no_std; this host program plays the hardware: an LED that
// prints, a pretend TMP102 on a pretend I2C bus, a bouncing button.

use core::convert::Infallible;

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{ErrorType, OutputPin};
use embedded_hal::i2c::{self, I2c, Operation};
use m33_embedded_rust_solution::bonus_pid::{Heater, Pid};
use m33_embedded_rust_solution::ex01_blink::{self, Led, Pattern, SOS};
use m33_embedded_rust_solution::ex02_sensor::{Address, Tmp102};
use m33_embedded_rust_solution::ex03_framing::{self, FrameDecoder};
use m33_embedded_rust_solution::ex04_registers::{self, Gpio, GpioRegisters, Mode};
use m33_embedded_rust_solution::ex05_input::{self, Debouncer, EventQueue};

/// An "LED" that prints its pin level.
struct ConsolePin;

impl ErrorType for ConsolePin {
    type Error = Infallible;
}

impl OutputPin for ConsolePin {
    fn set_low(&mut self) -> Result<(), Infallible> {
        print!("_");
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), Infallible> {
        print!("#");
        Ok(())
    }
}

/// A delay that only counts (no real waiting in a demo).
struct CountingDelay(u64);

impl DelayNs for CountingDelay {
    fn delay_ns(&mut self, ns: u32) {
        self.0 += ns as u64;
    }
}

/// A pretend TMP102 at 0x48 reading 23.5 °C, with a config register.
struct FakeTmp102 {
    pointer: u8,
    config: [u8; 2],
}

impl i2c::ErrorType for FakeTmp102 {
    type Error = Infallible;
}

impl I2c for FakeTmp102 {
    fn transaction(
        &mut self,
        _address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Infallible> {
        for op in operations {
            match op {
                Operation::Write(bytes) => {
                    self.pointer = bytes[0];
                    if bytes.len() == 3 && self.pointer == 1 {
                        self.config = [bytes[1], bytes[2]];
                    }
                }
                Operation::Read(buf) => {
                    let value = if self.pointer == 0 {
                        [0x17, 0x80]
                    } else {
                        self.config
                    };
                    buf.copy_from_slice(&value);
                }
            }
        }
        Ok(())
    }
}

fn ex1() {
    println!("--- 1: blink (# = on, _ = off)");
    let mut led = Led::new(ConsolePin, false);
    let mut delay = CountingDelay(0);
    print!("blocking blink x3: ");
    ex01_blink::blink(&mut led, &mut delay, 3, 200, 300).unwrap();
    println!("   ({} ms of delays)", delay.0 / 1_000_000);
    let mut pattern = Pattern::new(&SOS);
    print!("SOS, ticked every 100 ms: ");
    let mut on = false;
    for t in (0..pattern.period_ms()).step_by(100) {
        if let Some(state) = pattern.tick(t) {
            on = state;
        }
        print!("{}", if on { '#' } else { '_' });
    }
    println!();
}

fn ex2() {
    println!("--- 2: an I2C temperature sensor");
    let mut sensor = Tmp102::new(
        FakeTmp102 {
            pointer: 0,
            config: [0x60, 0xA0],
        },
        Address::Gnd,
    );
    let mc = sensor.temperature_millicelsius().unwrap();
    println!("temperature: {}.{:03} °C", mc / 1000, mc % 1000);
    sensor.set_shutdown(true).unwrap();
    let bus = sensor.release();
    println!("config after shutdown: {:02x?}", bus.config);
}

fn ex3() {
    println!("--- 3: COBS + CRC framing");
    let payload = [0x11, 0x22, 0x00, 0x33];
    let frame = ex03_framing::encode_frame(&payload).unwrap();
    println!("payload {payload:02x?} -> frame {:02x?}", frame.as_slice());
    let mut decoder = FrameDecoder::new();
    let mut corrupted = frame.clone();
    corrupted[2] ^= 0x01;
    // garbage (a receiver joining mid-stream), the good frame, a corrupted copy
    for byte in [0x55, 0xAA, 0x00]
        .iter()
        .chain(frame.iter())
        .chain(corrupted.iter())
    {
        if let Some(result) = decoder.push(*byte) {
            println!("decoder: {result:02x?}");
        }
    }
}

fn ex4() {
    println!("--- 4: memory-mapped registers");
    let mut regs = GpioRegisters::default();
    let ptr: *mut GpioRegisters = &mut regs;
    // SAFETY: `regs` outlives `gpio`, and until we read it at the end it is
    // only accessed through raw pointers (the driver and the simulation).
    let mut gpio = unsafe { Gpio::new(ptr) };
    gpio.set_mode(5, Mode::Output);
    gpio.set_mode(13, Mode::Output);
    for (pin, high) in [(5, true), (13, true), (5, false)] {
        gpio.write(pin, high);
        // SAFETY: as above; the "chip" reacts to each write.
        unsafe { ex04_registers::simulate_hardware(ptr) };
        // SAFETY: as above.
        let output = unsafe { (*ptr).output };
        println!(
            "pin {pin:>2} {} -> output register {output:#018b}",
            if high { "high" } else { "low " }
        );
    }
    println!(
        "mode register: {:#034b} (pins 5 and 13 = 01, output)",
        regs.mode
    );
}

fn ex5() {
    println!("--- 5: a bouncing button");
    let samples = [
        0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0,
    ];
    let mut queue = EventQueue::new();
    let (mut producer, mut consumer) = queue.split();
    let (mut debouncer, mut dropped) = (Debouncer::new(4), 0);
    for (ms, raw) in samples.iter().enumerate() {
        ex05_input::on_tick(
            &mut debouncer,
            *raw == 1,
            ms as u32,
            &mut producer,
            &mut dropped,
        );
    }
    let events = ex05_input::drain(&mut consumer);
    println!("raw samples {samples:?}");
    println!("debounced events {events:?}");
    println!("press lasted {:?} ms", ex05_input::press_durations(&events));
}

fn bonus() {
    println!("--- bonus: a PID controller holding a heater at 60 (from 20)");
    let mut pid = Pid::new(4_000, 400, 1_000, 0, 1_000);
    let mut heater = Heater {
        temperature: 20,
        ambient: 20,
        loss: 20,
        gain: 20,
    };
    for step in 0..60 {
        let power = pid.update(60, heater.temperature);
        heater.step(power);
        if step % 6 == 0 {
            println!(
                "step {step:>2}: power {power:>4}, temperature {}",
                heater.temperature
            );
        }
    }
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("1") => ex1(),
        Some("2") => ex2(),
        Some("3") => ex3(),
        Some("4") => ex4(),
        Some("5") => ex5(),
        Some("bonus") => bonus(),
        Some("all") => {
            ex1();
            ex2();
            ex3();
            ex4();
            ex5();
            bonus();
        }
        _ => println!(
            "33-embedded-rust -- reference solution\n\n  cargo run -p m33-embedded-rust-solution -- <1-5|bonus|all>"
        ),
    }
}
