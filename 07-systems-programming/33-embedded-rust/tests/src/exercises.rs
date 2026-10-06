use crate::sut::bonus_pid::{Heater, Pid};
use crate::sut::ex01_blink::{self as blink, Led, Pattern, SOS};
use crate::sut::ex02_sensor::{self as sensor, Address, Error, Tmp102};
use crate::sut::ex03_framing::{self as framing, FrameDecoder, FrameError, MAX_PAYLOAD};
use crate::sut::ex04_registers::{self as registers, Gpio, GpioRegisters, Mode};
use crate::sut::ex05_input::{self as input, Button, Debouncer, Edge, Event, EventQueue};
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::ErrorKind;
use embedded_hal_mock::eh1::digital::{Mock as PinMock, State, Transaction as Pin};
use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2c};

/// Records the total time waited, in nanoseconds.
#[derive(Default)]
struct RecordingDelay(u64);

impl DelayNs for RecordingDelay {
    fn delay_ns(&mut self, ns: u32) {
        self.0 += ns as u64;
    }
}

// ---------------------------------------------------------------- Exercise 1

#[test]
fn ex1_led_polarity() {
    let pin = PinMock::new(&[Pin::set(State::High), Pin::set(State::Low)]);
    let mut led = Led::new(pin, false);
    led.on().unwrap();
    led.off().unwrap();
    led.release().done();

    let pin = PinMock::new(&[
        Pin::set(State::Low),
        Pin::set(State::High),
        Pin::set(State::Low),
    ]);
    let mut active_low = Led::new(pin, true);
    active_low.on().unwrap();
    active_low.off().unwrap();
    active_low.set(true).unwrap();
    active_low.release().done();
}

#[test]
fn ex1_blocking_blink() {
    let expected: Vec<Pin> = (0..3)
        .flat_map(|_| [Pin::set(State::High), Pin::set(State::Low)])
        .collect();
    let mut led = Led::new(PinMock::new(&expected), false);
    let mut delay = RecordingDelay::default();
    blink::blink(&mut led, &mut delay, 3, 200, 300).unwrap();
    assert_eq!(delay.0, 3 * 500 * 1_000_000);
    led.release().done();
}

#[test]
fn ex1_pattern_ticks() {
    let steps = [(true, 100), (false, 50)];
    let mut p = Pattern::new(&steps);
    assert_eq!(p.period_ms(), 150);
    assert_eq!(
        p.tick(1_000),
        Some(true),
        "the first tick starts the pattern"
    );
    assert_eq!(p.tick(1_050), None);
    assert_eq!(p.tick(1_099), None);
    assert_eq!(p.tick(1_100), Some(false));
    assert_eq!(p.tick(1_149), None);
    assert_eq!(p.tick(1_150), Some(true), "and repeats");
    // a late tick doesn't make the schedule drift
    assert_eq!(p.tick(1_260), Some(false));
    assert_eq!(p.tick(1_300), Some(true), "next step due at 1300, not 1310");
    assert_eq!(Pattern::new(&[]).tick(0), None);
}

#[test]
fn ex1_pattern_survives_clock_wraparound() {
    let steps = [(true, 100), (false, 100)];
    let mut p = Pattern::new(&steps);
    let start = u32::MAX - 50;
    assert_eq!(p.tick(start), Some(true));
    assert_eq!(p.tick(start.wrapping_add(99)), None);
    assert_eq!(p.tick(start.wrapping_add(100)), Some(false));
}

#[test]
fn ex1_sos() {
    let mut p = Pattern::new(&SOS);
    assert_eq!(p.period_ms(), 3_400);
    let mut changes = 0;
    for t in 0..p.period_ms() {
        changes += p.tick(t).is_some() as u32;
    }
    assert_eq!(changes, 18, "one change per step");
}

// ---------------------------------------------------------------- Exercise 2

const A: u8 = 0x48;

#[test]
fn ex2_conversion() {
    assert_eq!(sensor::raw_to_millicelsius([0x19, 0x00]), 25_000);
    assert_eq!(
        sensor::raw_to_millicelsius([0x00, 0x10]),
        62,
        "one bit is 62.5 m°C"
    );
    assert_eq!(sensor::raw_to_millicelsius([0x7F, 0xF0]), 127_937);
    assert_eq!(sensor::raw_to_millicelsius([0x00, 0x00]), 0);
    assert_eq!(
        sensor::raw_to_millicelsius([0xFF, 0xF0]),
        -62,
        "negative: two's complement"
    );
    assert_eq!(sensor::raw_to_millicelsius([0xE7, 0x00]), -25_000);
    assert_eq!(sensor::raw_to_millicelsius([0xC9, 0x00]), -55_000);
}

#[test]
fn ex2_reading_temperature() {
    let i2c = I2cMock::new(&[I2c::write_read(
        A,
        vec![sensor::REG_TEMPERATURE],
        vec![0x17, 0x80],
    )]);
    let mut tmp = Tmp102::new(i2c, Address::Gnd);
    assert_eq!(tmp.temperature_millicelsius().unwrap(), 23_500);
    tmp.release().done();
    // the address comes from the enum
    let i2c = I2cMock::new(&[I2c::write_read(
        0x4B,
        vec![sensor::REG_TEMPERATURE],
        vec![0, 0],
    )]);
    let mut scl = Tmp102::new(i2c, Address::Scl);
    assert_eq!(scl.temperature_millicelsius().unwrap(), 0);
    scl.release().done();
}

#[test]
fn ex2_configuration_is_read_modify_write() {
    let i2c = I2cMock::new(&[
        I2c::write_read(A, vec![sensor::REG_CONFIG], vec![0x60, 0xA0]),
        I2c::write(A, vec![sensor::REG_CONFIG, 0x61, 0xA0]),
        I2c::write_read(A, vec![sensor::REG_CONFIG], vec![0x61, 0xA0]),
        I2c::write(A, vec![sensor::REG_CONFIG, 0xE1, 0xA0]),
        I2c::write_read(A, vec![sensor::REG_CONFIG], vec![0x61, 0xA0]),
        I2c::write(A, vec![sensor::REG_CONFIG, 0x60, 0xA0]),
    ]);
    let mut tmp = Tmp102::new(i2c, Address::Gnd);
    tmp.set_shutdown(true).unwrap();
    tmp.trigger_one_shot().unwrap();
    tmp.set_shutdown(false).unwrap();
    tmp.release().done();
}

#[test]
fn ex2_bus_errors_propagate() {
    let i2c =
        I2cMock::new(&[I2c::write_read(A, vec![0x00], vec![0, 0]).with_error(ErrorKind::Other)]);
    let mut tmp = Tmp102::new(i2c, Address::Gnd);
    assert_eq!(
        tmp.temperature_millicelsius(),
        Err(Error::I2c(ErrorKind::Other))
    );
    tmp.release().done();
}

// ---------------------------------------------------------------- Exercise 3

fn encode(input: &[u8]) -> Vec<u8> {
    let mut buf = [0u8; 600];
    let n = framing::cobs_encode(input, &mut buf).unwrap();
    buf[..n].to_vec()
}

fn decode(input: &[u8]) -> Result<Vec<u8>, FrameError> {
    let mut buf = [0u8; 600];
    framing::cobs_decode(input, &mut buf).map(|n| buf[..n].to_vec())
}

#[test]
fn ex3_crc16() {
    assert_eq!(
        framing::crc16(b"123456789"),
        0x29B1,
        "the standard check value"
    );
    assert_eq!(framing::crc16(b""), 0xFFFF);
    assert_ne!(framing::crc16(b"hello"), framing::crc16(b"hellp"));
}

#[test]
fn ex3_cobs_vectors() {
    assert_eq!(encode(&[]), [0x01]);
    assert_eq!(encode(&[0x00]), [0x01, 0x01]);
    assert_eq!(encode(&[0x00, 0x00]), [0x01, 0x01, 0x01]);
    assert_eq!(
        encode(&[0x11, 0x22, 0x00, 0x33]),
        [0x03, 0x11, 0x22, 0x02, 0x33]
    );
    assert_eq!(
        encode(&[0x11, 0x00, 0x00, 0x00]),
        [0x02, 0x11, 0x01, 0x01, 0x01]
    );
    let run: Vec<u8> = (1..=254).collect();
    let mut expected = vec![0xFF];
    expected.extend(&run);
    assert_eq!(encode(&run), expected, "a full run needs no trailing code");
    let longer: Vec<u8> = (1..=255).collect();
    let encoded = encode(&longer);
    assert_eq!((encoded[0], encoded[255], encoded.len()), (0xFF, 0x02, 257));
}

#[test]
fn ex3_cobs_round_trips_without_zeros() {
    let mut seed: u32 = 7;
    for len in [0usize, 1, 2, 10, 253, 254, 255, 256, 300, 509] {
        let data: Vec<u8> = (0..len)
            .map(|_| {
                seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                if seed.is_multiple_of(5) {
                    0
                } else {
                    (seed >> 16) as u8
                }
            })
            .collect();
        let encoded = encode(&data);
        assert!(
            !encoded.contains(&0),
            "no zero bytes in the encoding (len {len})"
        );
        assert_eq!(decode(&encoded).unwrap(), data, "len {len}");
    }
}

#[test]
fn ex3_cobs_errors() {
    assert_eq!(
        decode(&[0x03, 0x11, 0x00, 0x33]),
        Err(FrameError::BadEncoding),
        "a zero inside"
    );
    assert_eq!(
        decode(&[0x05, 0x11]),
        Err(FrameError::BadEncoding),
        "a code past the end"
    );
    assert_eq!(
        framing::cobs_encode(&[1, 2, 3], &mut [0u8; 3]),
        Err(FrameError::Overflow)
    );
    assert_eq!(
        framing::cobs_decode(&[0x04, 1, 2, 3], &mut [0u8; 2]),
        Err(FrameError::Overflow)
    );
}

#[test]
fn ex3_frames() {
    let frame = framing::encode_frame(&[0x11, 0x22, 0x00, 0x33]).unwrap();
    assert_eq!(frame.last(), Some(&0));
    assert_eq!(
        frame.iter().filter(|b| **b == 0).count(),
        1,
        "the delimiter is the only zero"
    );
    let body = &frame[..frame.len() - 1];
    assert_eq!(
        framing::decode_frame(body).unwrap().as_slice(),
        [0x11, 0x22, 0x00, 0x33]
    );
    let mut corrupted = body.to_vec();
    corrupted[1] ^= 0x40;
    assert_eq!(framing::decode_frame(&corrupted), Err(FrameError::BadCrc));
    assert_eq!(
        framing::decode_frame(&[0x02, 0x05]),
        Err(FrameError::TooShort)
    );
    assert_eq!(
        framing::encode_frame(&[1; MAX_PAYLOAD + 1]),
        Err(FrameError::Overflow)
    );
    let biggest = framing::encode_frame(&[0; MAX_PAYLOAD]).unwrap();
    assert_eq!(
        framing::decode_frame(&biggest[..biggest.len() - 1])
            .unwrap()
            .len(),
        MAX_PAYLOAD
    );
}

#[test]
fn ex3_streaming_decoder() {
    let mut decoder = FrameDecoder::new();
    let a = framing::encode_frame(b"first").unwrap();
    let b = framing::encode_frame(b"second").unwrap();
    let mut results = Vec::new();
    // joining mid-stream (garbage), two delimiters in a row, two frames
    for byte in [0x41u8, 0x42, 0x00, 0x00]
        .iter()
        .chain(a.iter())
        .chain(b.iter())
    {
        if let Some(r) = decoder.push(*byte) {
            results.push(r.map(|v| v.to_vec()));
        }
    }
    assert_eq!(results.len(), 3, "{results:?}");
    assert!(
        results[0].is_err(),
        "the garbage before the first delimiter"
    );
    assert_eq!(results[1], Ok(b"first".to_vec()));
    assert_eq!(results[2], Ok(b"second".to_vec()));

    // an endless line of non-zero bytes overflows, then the decoder resyncs
    let mut decoder = FrameDecoder::new();
    let mut last = None;
    for _ in 0..500 {
        last = decoder.push(0x7F).or(last);
    }
    assert!(last.is_none(), "nothing until a delimiter");
    assert_eq!(decoder.push(0), Some(Err(FrameError::Overflow)));
    let frame = framing::encode_frame(b"ok").unwrap();
    let got: Vec<_> = frame.iter().filter_map(|b| decoder.push(*b)).collect();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].as_ref().unwrap().as_slice(), b"ok");
}

// ---------------------------------------------------------------- Exercise 4

#[test]
fn ex4_bit_fields() {
    assert_eq!(registers::set_field(0, 4, 2, 0b11), 0b11_0000);
    assert_eq!(registers::set_field(0xFFFF_FFFF, 4, 2, 0b01), 0xFFFF_FFDF);
    assert_eq!(
        registers::set_field(0, 4, 2, 0b111),
        0b11_0000,
        "extra bits are masked off"
    );
    assert_eq!(registers::get_field(0b1101_0000, 4, 3), 0b101);
    assert_eq!(registers::set_field(0x1234_5678, 0, 32, 7), 7);
    assert_eq!(registers::get_field(0x8000_0000, 31, 1), 1);
}

#[test]
fn ex4_gpio_driver() {
    let mut regs = GpioRegisters {
        mode: 0xFFFF_FFFF,
        ..Default::default()
    };
    let ptr: *mut GpioRegisters = &mut regs;
    // SAFETY: `regs` outlives `gpio`; until the end it's only accessed
    // through raw pointers (the driver and the hardware simulation).
    let mut gpio = unsafe { Gpio::new(ptr) };
    gpio.set_mode(3, Mode::Output);
    gpio.set_mode(0, Mode::Input);
    assert_eq!(
        (gpio.mode(3), gpio.mode(0), gpio.mode(1)),
        (Mode::Output, Mode::Input, Mode::Analog)
    );
    // SAFETY: as above.
    let mode = unsafe { (*ptr).mode };
    assert_eq!(mode, 0xFFFF_FF7C, "only pins 0 and 3 changed");

    let output = || {
        // SAFETY: as above.
        unsafe { registers::simulate_hardware(ptr) };
        // SAFETY: as above.
        unsafe { (*ptr).output }
    };
    gpio.write(3, true);
    assert_eq!(output(), 0b1000);
    gpio.write(15, true);
    assert_eq!(output(), 0b1000_0000_0000_1000);
    gpio.write(3, false);
    assert_eq!(output(), 0b1000_0000_0000_0000);

    // SAFETY: as above.
    unsafe { (*ptr).input = 0b100 };
    assert!(gpio.read(2) && !gpio.read(1));
}

#[test]
fn ex4_set_wins_over_reset() {
    let mut regs = GpioRegisters {
        output: 0b1,
        bit_set_reset: (1 << 16) | (1 << 1),
        ..Default::default()
    };
    // SAFETY: `regs` is valid and not otherwise borrowed during the call.
    unsafe { registers::simulate_hardware(&mut regs) };
    assert_eq!((regs.output, regs.bit_set_reset), (0b10, 0));
}

// ---------------------------------------------------------------- Exercise 5

#[test]
fn ex5_debouncer_ignores_bounces() {
    let mut d = Debouncer::new(3);
    let samples = [true, false, true, true, false, true, true, true, true];
    let edges: Vec<(usize, Edge)> = samples
        .iter()
        .enumerate()
        .filter_map(|(i, s)| d.update(*s).map(|e| (i, e)))
        .collect();
    assert_eq!(
        edges,
        [(7, Edge::Pressed)],
        "three agreeing samples in a row"
    );
    assert!(d.is_pressed());
    let edges: Vec<Edge> = [false, false, true, false, false, false]
        .iter()
        .filter_map(|s| d.update(*s))
        .collect();
    assert_eq!(edges, [Edge::Released]);
    let mut instant = Debouncer::new(1);
    assert_eq!(instant.update(true), Some(Edge::Pressed));
}

#[test]
fn ex5_button_is_active_low() {
    let pin = PinMock::new(&[
        Pin::get(State::High),
        Pin::get(State::Low),
        Pin::get(State::Low),
        Pin::get(State::High),
        Pin::get(State::High),
    ]);
    let mut button = Button::new(pin, 2);
    let edges: Vec<Option<Edge>> = (0..5).map(|_| button.poll().unwrap()).collect();
    assert_eq!(
        edges,
        [None, None, Some(Edge::Pressed), None, Some(Edge::Released)]
    );
    button.release().done();
}

#[test]
fn ex5_events_cross_the_queue() {
    let mut queue = EventQueue::new();
    let (mut producer, mut consumer) = queue.split();
    let (mut d, mut dropped) = (Debouncer::new(2), 0);
    let samples = [0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0];
    for (ms, s) in samples.iter().enumerate() {
        input::on_tick(&mut d, *s == 1, ms as u32 * 10, &mut producer, &mut dropped);
    }
    let events = input::drain(&mut consumer);
    let expected = [
        Event {
            edge: Edge::Pressed,
            at_ms: 20,
        },
        Event {
            edge: Edge::Released,
            at_ms: 60,
        },
        Event {
            edge: Edge::Pressed,
            at_ms: 90,
        },
        Event {
            edge: Edge::Released,
            at_ms: 110,
        },
    ];
    assert_eq!(events.as_slice(), expected);
    assert_eq!(dropped, 0);
    assert!(input::drain(&mut consumer).is_empty());
    assert_eq!(input::press_durations(&events).as_slice(), [40, 20]);
}

#[test]
fn ex5_a_full_queue_drops_and_counts() {
    let mut queue = EventQueue::new();
    let (mut producer, mut consumer) = queue.split();
    let (mut d, mut dropped) = (Debouncer::new(1), 0);
    for ms in 0..40u32 {
        input::on_tick(&mut d, ms % 2 == 0, ms, &mut producer, &mut dropped);
    }
    assert_eq!(
        input::drain(&mut consumer).len(),
        15,
        "a Queue<_, 16> holds 15"
    );
    assert_eq!(dropped, 25);
    let lone = [
        Event {
            edge: Edge::Released,
            at_ms: 5,
        },
        Event {
            edge: Edge::Pressed,
            at_ms: 7,
        },
    ];
    assert!(
        input::press_durations(&lone).is_empty(),
        "unmatched edges are ignored"
    );
}

// --------------------------------------------------------------------- Bonus

#[test]
fn bonus_proportional_and_limits() {
    let mut p_only = Pid::new(2_000, 0, 0, -100, 100);
    assert_eq!(p_only.update(50, 40), 20, "kp 2.0 x error 10");
    assert_eq!(p_only.update(50, 0), 100, "clamped to the maximum");
    assert_eq!(p_only.update(0, 90), -100, "and the minimum");
}

#[test]
fn bonus_integral_removes_offset_and_does_not_wind_up() {
    let mut pi = Pid::new(0, 100, 0, 0, 50);
    let outputs: Vec<i32> = (0..5).map(|_| pi.update(10, 0)).collect();
    assert_eq!(outputs, [1, 2, 3, 4, 5], "ki 0.1 x accumulated error");
    for _ in 0..10_000 {
        pi.update(10, 0); // saturated for a long time
    }
    // the moment the error reverses, the output responds within a few steps
    let after: Vec<i32> = (0..3).map(|_| pi.update(0, 10)).collect();
    assert!(after[2] < 50, "anti-windup: {after:?}");
    pi.reset();
    assert_eq!(pi.update(10, 0), 1);
}

#[test]
fn bonus_no_derivative_kick_on_setpoint_change() {
    let mut pd = Pid::new(0, 0, 1_000, -1_000, 1_000);
    assert_eq!(pd.update(10, 5), 0, "no derivative on the first step");
    assert_eq!(
        pd.update(500, 5),
        0,
        "the setpoint jumped; the measurement didn't"
    );
    assert_eq!(pd.update(500, 15), -10, "the measurement rose by 10");
}

#[test]
fn bonus_controls_a_heater() {
    let mut pid = Pid::new(4_000, 400, 1_000, 0, 1_000);
    let mut heater = Heater {
        temperature: 20,
        ambient: 20,
        loss: 20,
        gain: 20,
    };
    let mut peak = 0;
    for _ in 0..100 {
        let power = pid.update(60, heater.temperature);
        heater.step(power);
        peak = peak.max(heater.temperature);
    }
    assert!(
        (heater.temperature - 60).abs() <= 2,
        "settles near 60: {}",
        heater.temperature
    );
    assert!(peak <= 70, "without overshooting much: {peak}");
}
