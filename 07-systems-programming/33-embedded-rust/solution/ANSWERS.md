# Answers · 33 Embedded Rust

## Exercise 1: `tick` versus `delay_ms`

`delay_ms` busy-waits: during an SOS pattern the CPU does nothing else for
3.4 seconds -- bytes arriving on the UART overflow its buffer, a sensor
deadline passes, a button press is missed. A state machine ticked from the
main loop (or a timer interrupt) does its tiny bit of work and returns, so
one loop serves the LED, the sensor and the serial port in turn, and the
CPU can sleep (`wfi`) between ticks to save power. This is "cooperative
multitasking by hand"; Embassy's async/await generates these state machines
for you.

## Exercise 3: Why at most one bad frame?

COBS guarantees `0x00` appears only as a delimiter. A receiver that starts
listening mid-frame collects the tail of that frame, sees the delimiter, and
reports one bad frame (bad COBS or bad CRC); from that delimiter on, it is
exactly aligned with the sender. Corruption is local in the same way: a
damaged byte spoils its frame, the CRC catches it, and the next delimiter
resynchronises. Formats without a reserved delimiter (a length prefix alone)
can stay misaligned forever after one corrupted length byte.

## Exercise 4: Why a set/reset register?

Read-modify-write is three steps. If an interrupt fires between the read and
the write and changes another pin of the same port, the main code's write
puts the stale value back and undoes the interrupt's change -- a race with
no data structure in sight. A write to the set/reset register changes only
the pins whose bits are 1, in a single bus write: no read, nothing to go
stale, so interrupt handlers and main code can drive different pins of one
port safely without disabling interrupts.
