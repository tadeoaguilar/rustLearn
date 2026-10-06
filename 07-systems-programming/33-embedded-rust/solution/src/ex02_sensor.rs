//! Exercise 2: an I2C sensor driver -- the TMP102 temperature sensor.
//!
//! A driver turns register reads and writes into meaningful values. The
//! TMP102 (Texas Instruments) is typical: a pointer register selects which
//! register an access hits; temperature is a 12-bit two's-complement value
//! left-justified in two bytes, 0.0625 °C per bit; the configuration
//! register has a shutdown bit (to save power) and a one-shot bit (measure
//! once while shut down).
//!
//! | Pointer | Register | Bytes |
//! |---|---|---|
//! | `0x00` | temperature | 2, read-only |
//! | `0x01` | configuration | 2: byte 0 bit 0 = SD (shutdown), bit 7 = OS (one-shot) |
//!
//! Integer maths only: many microcontrollers have no floating-point unit.

use embedded_hal::i2c::I2c;

/// The four addresses the TMP102 can have, set by wiring its ADD0 pin. An
/// enum instead of a `u8`: an invalid address doesn't compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Address {
    /// ADD0 to ground (the usual breakout default).
    Gnd = 0x48,
    Vcc = 0x49,
    Sda = 0x4A,
    Scl = 0x4B,
}
pub const REG_TEMPERATURE: u8 = 0x00;
pub const REG_CONFIG: u8 = 0x01;
pub const CONFIG_SHUTDOWN: u8 = 0b0000_0001;
pub const CONFIG_ONE_SHOT: u8 = 0b1000_0000;

#[derive(Debug, PartialEq, Eq)]
pub enum Error<E> {
    /// The bus failed.
    I2c(E),
}

impl<E> From<E> for Error<E> {
    fn from(e: E) -> Self {
        Error::I2c(e)
    }
}

pub struct Tmp102<I> {
    i2c: I,
    address: u8,
}

/// Raw register bytes -> milli-degrees Celsius. The 12-bit reading sits in
/// the top bits; an arithmetic shift keeps the sign. 1 bit = 62.5 m°C.
pub fn raw_to_millicelsius(bytes: [u8; 2]) -> i32 {
    let raw = i16::from_be_bytes(bytes) >> 4;
    raw as i32 * 625 / 10
}

impl<I: I2c> Tmp102<I> {
    pub fn new(i2c: I, address: Address) -> Self {
        Tmp102 {
            i2c,
            address: address as u8,
        }
    }

    /// One read: write the pointer, then read two bytes (a repeated start).
    fn read_register(&mut self, register: u8) -> Result<[u8; 2], Error<I::Error>> {
        let mut buf = [0u8; 2];
        self.i2c.write_read(self.address, &[register], &mut buf)?;
        Ok(buf)
    }

    pub fn temperature_millicelsius(&mut self) -> Result<i32, Error<I::Error>> {
        Ok(raw_to_millicelsius(self.read_register(REG_TEMPERATURE)?))
    }

    /// Set or clear the shutdown bit, keeping the other configuration bits
    /// (read-modify-write).
    pub fn set_shutdown(&mut self, shutdown: bool) -> Result<(), Error<I::Error>> {
        let [mut high, low] = self.read_register(REG_CONFIG)?;
        if shutdown {
            high |= CONFIG_SHUTDOWN;
        } else {
            high &= !CONFIG_SHUTDOWN;
        }
        self.i2c.write(self.address, &[REG_CONFIG, high, low])?;
        Ok(())
    }

    /// Start a single conversion while shut down.
    pub fn trigger_one_shot(&mut self) -> Result<(), Error<I::Error>> {
        let [high, low] = self.read_register(REG_CONFIG)?;
        self.i2c
            .write(self.address, &[REG_CONFIG, high | CONFIG_ONE_SHOT, low])?;
        Ok(())
    }

    /// Give the bus back.
    pub fn release(self) -> I {
        self.i2c
    }
}
