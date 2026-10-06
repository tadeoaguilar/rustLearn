//! Bonus: a PID controller in fixed-point arithmetic.
//!
//! Keep a heater at a temperature, a motor at a speed: a PID controller
//! looks at the error (setpoint - measurement) and outputs a correction
//! from three terms -- **P**roportional to the error now, **I**ntegral of
//! past errors (removes steady offset), **D**erivative (damps overshoot).
//!
//! On a microcontroller without an FPU, floats are slow, so this uses
//! fixed point: gains are in thousandths (`kp = 1500` means 1.5), and
//! values are plain integers in the sensor's unit. The integral is clamped
//! ("anti-windup"), or a long saturation would store up a huge correction.

#[derive(Debug, Clone)]
pub struct Pid {
    /// Gains in thousandths.
    pub kp: i32,
    pub ki: i32,
    pub kd: i32,
    /// Output limits (also bound the integral's contribution).
    pub out_min: i32,
    pub out_max: i32,
    integral: i64,
    previous: Option<i32>,
}

impl Pid {
    pub fn new(kp: i32, ki: i32, kd: i32, out_min: i32, out_max: i32) -> Self {
        Pid {
            kp,
            ki,
            kd,
            out_min,
            out_max,
            integral: 0,
            previous: None,
        }
    }

    /// One control step: the output for this error, clamped to the limits.
    /// The derivative is of the *measurement* (not the error), so a
    /// setpoint change doesn't cause a kick.
    pub fn update(&mut self, setpoint: i32, measurement: i32) -> i32 {
        let error = setpoint as i64 - measurement as i64;
        self.integral += error * self.ki as i64;
        // Anti-windup: the integral term alone may not exceed the output range.
        self.integral = self
            .integral
            .clamp(self.out_min as i64 * 1000, self.out_max as i64 * 1000);
        let derivative = match self.previous {
            Some(prev) => -(measurement as i64 - prev as i64) * self.kd as i64,
            None => 0,
        };
        self.previous = Some(measurement);
        let output = (error * self.kp as i64 + self.integral + derivative) / 1000;
        output.clamp(self.out_min as i64, self.out_max as i64) as i32
    }

    pub fn reset(&mut self) {
        self.integral = 0;
        self.previous = None;
    }
}

/// A toy plant for testing: a heater. Each step the temperature moves
/// toward ambient by 1/`loss` of the difference and up by `power / gain`.
#[derive(Debug, Clone)]
pub struct Heater {
    pub temperature: i32,
    pub ambient: i32,
    pub loss: i32,
    pub gain: i32,
}

impl Heater {
    pub fn step(&mut self, power: i32) {
        self.temperature += (self.ambient - self.temperature) / self.loss + power / self.gain;
    }
}
