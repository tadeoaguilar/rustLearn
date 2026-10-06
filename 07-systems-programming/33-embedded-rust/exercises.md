# Exercises: Embedded Rust

A microcontroller has kilobytes of RAM, no operating system and no heap
unless you bring one. Embedded Rust is `#![no_std]` Rust -- only `core` --
talking to hardware through registers, and through the `embedded-hal`
traits that let one driver run on any chip. This module's library is
`no_std` throughout; the tests run it on your machine against mock pins and
a mock I2C bus (`embedded-hal-mock`), and the demo binary plays the hardware.

**Setup**: `embedded-hal` 1.0, `heapless` (fixed-capacity collections),
`embedded-hal-mock` in the tests. No board needed; flashing a real one is
optional (GETTING_STARTED.md).

---

## Exercise 1: Blink

**Difficulty**: Easy
**Time**: 45 minutes

**Learning Objectives**:
- Write drivers generic over `OutputPin` and `DelayNs`
- Hide wiring details (active-low LEDs)
- Replace blocking delays with a state machine

1. `Led`: `new(pin, active_low)`, `set`, `on`, `off`, `release` (give the pin back).
2. `blink(led, delay, times, on_ms, off_ms)`, blocking.
3. `Pattern::tick(now_ms)`: `Some(on)` on the first tick and whenever the
   current step's duration has elapsed; schedule from the step's start (no
   drift when ticks are late); handle a wrapping `u32` clock. `period_ms`.

**Question**: why is `tick` better than `delay_ms` in a device that also
reads a sensor and listens to a serial port?

---

## Exercise 2: An I2C Sensor Driver (TMP102)

**Difficulty**: Medium
**Time**: 1 hour

**Learning Objectives**:
- Turn register reads into physical values with integer maths
- Read-modify-write a configuration register
- Test a driver against an exact expected bus transcript

1. `raw_to_millicelsius([msb, lsb])`: 12 bits, left-justified, two's
   complement, 62.5 m°C per bit.
2. `Tmp102::new(i2c, Address)` (an enum: invalid addresses don't compile),
   `temperature_millicelsius` (`write_read` the pointer register, two bytes).
3. `set_shutdown(bool)` and `trigger_one_shot`: read the configuration,
   change one bit, write `[REG_CONFIG, high, low]`.
4. Bus errors come back as `Error::I2c(e)` (`From` impl); `release`.

---

## Exercise 3: A Serial Protocol -- COBS and CRC

**Difficulty**: Medium
**Time**: 1.5 hours

**Learning Objectives**:
- Frame a byte stream so a receiver can resynchronise
- Detect corruption
- Work without a heap

1. `crc16`: CRC-16/CCITT-FALSE (poly 0x1021, init 0xFFFF); `crc16(b"123456789") == 0x29B1`.
2. `cobs_encode` / `cobs_decode` into caller buffers (`Overflow`,
   `BadEncoding`); no `0x00` in the output; a final 254-byte run gets no
   extra code byte.
3. `encode_frame` (COBS of payload + big-endian CRC, then `0x00`),
   `decode_frame` (`TooShort`, `BadCrc`).
4. `FrameDecoder::push(byte)`: a result at each delimiter; skip empty
   frames; after an overflow, discard until the next delimiter.

**Question**: why does a receiver that powers up mid-transmission recover
after at most one bad frame?

---

## Exercise 4: Memory-Mapped Registers

**Difficulty**: Medium
**Time**: 1 hour

**Learning Objectives**:
- Access registers with volatile reads and writes
- Manipulate bit fields
- Use a set/reset register instead of read-modify-write

1. `set_field(value, offset, width, field)`, `get_field`.
2. `Gpio::set_mode` (2 bits per pin in `mode`, other pins untouched),
   `mode`, `write` (via `bit_set_reset`: bit n sets, bit n+16 resets),
   `read` (from `input`) -- every access volatile, through the raw pointer.
   (`simulate_hardware` is provided: it plays the chip after each write.)

**Question**: why does the set/reset register exist, when software could
read `output`, change a bit and write it back?

---

## Exercise 5: Buttons and Interrupts

**Difficulty**: Medium
**Time**: 1 hour

**Learning Objectives**:
- Debounce a noisy input
- Move events from an interrupt to the main loop without locks or a heap

1. `Debouncer::update(raw)`: flip after `threshold` consecutive disagreeing
   samples; return the `Edge`.
2. `Button::poll` (active-low), `release`.
3. `on_tick` (the "interrupt": debounce, enqueue, count drops when full --
   never block), `drain` (the main loop), `press_durations`.

---

## Bonus: A Fixed-Point PID Controller

**Difficulty**: Medium
**Time**: 45 minutes

`Pid::update(setpoint, measurement)` with gains in thousandths, the
integral clamped (anti-windup), the derivative taken on the measurement (no
kick when the setpoint changes), the output clamped. The tests close the
loop around a toy `Heater`.
