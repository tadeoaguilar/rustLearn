//! Exercise 4: memory-mapped I/O -- talking to hardware registers.
//!
//! On a microcontroller, peripherals are controlled through *registers* at
//! fixed memory addresses: writing a bit in a GPIO's output register drives
//! a pin high. Two rules make that work in Rust:
//!
//! - every access is **volatile** (`read_volatile` / `write_volatile`), or
//!   the compiler may merge, reorder or delete accesses it thinks are
//!   pointless ("why read this twice? nothing changed it")
//! - registers are often **fields** packed into a 32-bit word: read, clear
//!   the field with a mask, set the new bits, write back
//!
//! Here the "hardware" is ordinary memory (a `#[repr(C)]` struct), so the
//! code runs on any machine, but it is written exactly as it would be for a
//! real GPIO block: a register-layout struct, accessed through a pointer.

use core::ptr::{addr_of, addr_of_mut};

/// A GPIO port's registers, in memory order (like an STM32's, simplified).
#[repr(C)]
#[derive(Debug, Default)]
pub struct GpioRegisters {
    /// 2 bits per pin: 00 input, 01 output, 10 alternate function, 11 analog.
    pub mode: u32,
    /// Bit n: the output level of pin n.
    pub output: u32,
    /// Bit n: the input level of pin n (read-only on real hardware).
    pub input: u32,
    /// Write 1 to bit n to set pin n; to bit n+16 to reset it. Writes of 0 do
    /// nothing, so no read-modify-write is needed (atomic from software's view).
    pub bit_set_reset: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Input = 0b00,
    Output = 0b01,
    Alternate = 0b10,
    Analog = 0b11,
}

/// Replace the `width`-bit field at `offset` in `value` with `field`.
pub fn set_field(value: u32, offset: u32, width: u32, field: u32) -> u32 {
    let mask = if width >= 32 {
        u32::MAX
    } else {
        ((1u32 << width) - 1) << offset
    };
    (value & !mask) | ((field << offset) & mask)
}

/// Read the `width`-bit field at `offset`.
pub fn get_field(value: u32, offset: u32, width: u32) -> u32 {
    let mask = if width >= 32 {
        u32::MAX
    } else {
        (1u32 << width) - 1
    };
    (value >> offset) & mask
}

/// A driver for one GPIO port, holding a pointer to its registers.
pub struct Gpio {
    regs: *mut GpioRegisters,
}

impl Gpio {
    /// # Safety
    ///
    /// `regs` must point to a valid `GpioRegisters` (real hardware or memory)
    /// that nothing else accesses while this `Gpio` exists.
    pub unsafe fn new(regs: *mut GpioRegisters) -> Self {
        Gpio { regs }
    }

    fn read_mode(&self) -> u32 {
        // SAFETY: `new`'s contract: `regs` is valid and ours alone.
        unsafe { addr_of!((*self.regs).mode).read_volatile() }
    }

    pub fn set_mode(&mut self, pin: u32, mode: Mode) {
        assert!(pin < 16, "a port has 16 pins");
        let value = set_field(self.read_mode(), pin * 2, 2, mode as u32);
        // SAFETY: as above.
        unsafe { addr_of_mut!((*self.regs).mode).write_volatile(value) }
    }

    pub fn mode(&self, pin: u32) -> Mode {
        match get_field(self.read_mode(), pin * 2, 2) {
            0b00 => Mode::Input,
            0b01 => Mode::Output,
            0b10 => Mode::Alternate,
            _ => Mode::Analog,
        }
    }

    /// Drive `pin` high or low through the set/reset register.
    pub fn write(&mut self, pin: u32, high: bool) {
        assert!(pin < 16, "a port has 16 pins");
        let bit = if high { 1 << pin } else { 1 << (pin + 16) };
        // SAFETY: as above.
        unsafe { addr_of_mut!((*self.regs).bit_set_reset).write_volatile(bit) }
    }

    /// The level of input `pin`.
    pub fn read(&self, pin: u32) -> bool {
        // SAFETY: as above.
        let input = unsafe { addr_of!((*self.regs).input).read_volatile() };
        input & (1 << pin) != 0
    }
}

/// What the hardware does the instant software writes the set/reset
/// register: apply it to the output register and clear it (if a pin is both
/// set and reset, set wins, as on STM32). Tests and the demo call this after
/// each write, to play the chip.
///
/// # Safety
///
/// `regs` must point to a valid `GpioRegisters`, and no reference to it may
/// be live (a `Gpio` holding the same pointer is fine: it only uses raw
/// pointers too).
pub unsafe fn simulate_hardware(regs: *mut GpioRegisters) {
    // SAFETY: the caller guarantees `regs` is valid; volatile accesses through
    // raw pointers don't create references that could alias the driver's.
    unsafe {
        let bsrr = addr_of!((*regs).bit_set_reset).read_volatile();
        let output = addr_of!((*regs).output).read_volatile();
        addr_of_mut!((*regs).output).write_volatile((output & !(bsrr >> 16)) | (bsrr & 0xFFFF));
        addr_of_mut!((*regs).bit_set_reset).write_volatile(0);
    }
}
