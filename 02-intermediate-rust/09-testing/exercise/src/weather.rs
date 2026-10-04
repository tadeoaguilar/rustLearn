//! A clothing advisor that depends on a weather API.
//!
//! `Advisor` doesn't know about HTTP. It depends on the `WeatherApi` *trait*,
//! so tests can hand it a fake or a mock instead of the network.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WeatherError {
    UnknownCity(String),
    Unavailable,
}

impl fmt::Display for WeatherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WeatherError::UnknownCity(c) => write!(f, "unknown city: {c}"),
            WeatherError::Unavailable => write!(f, "weather service unavailable"),
        }
    }
}

impl std::error::Error for WeatherError {}

/// `automock` generates `MockWeatherApi` -- only when compiling tests, so the
/// mock never ends up in the real library.
#[cfg_attr(test, mockall::automock)]
pub trait WeatherApi {
    fn temperature_c(&self, city: &str) -> Result<f64, WeatherError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Advice {
    /// below 0°C
    Freezing,
    /// 0°C up to (not including) 10°C
    Cold,
    /// 10°C up to (not including) 25°C
    Mild,
    /// 25°C and above
    Hot,
}

impl Advice {
    pub fn for_temperature(t: f64) -> Advice {
        if t > 25.0 {
            Advice::Hot
        } else if t >= 10.0 {
            Advice::Mild
        } else if t >= 0.0 {
            Advice::Cold
        } else {
            Advice::Freezing
        }
    }

    pub fn what_to_wear(self) -> &'static str {
        match self {
            Advice::Freezing => "a heavy coat",
            Advice::Cold => "a jacket",
            Advice::Mild => "a sweater",
            Advice::Hot => "a t-shirt",
        }
    }
}

pub struct Advisor<A: WeatherApi> {
    api: A,
}

impl<A: WeatherApi> Advisor<A> {
    pub fn new(api: A) -> Self {
        Advisor { api }
    }

    pub fn advise(&self, city: &str) -> Result<Advice, WeatherError> {
        let t = self.api.temperature_c(city)?;
        Ok(Advice::for_temperature(t))
    }

    pub fn sentence(&self, city: &str) -> String {
        match self.advise(city) {
            Ok(advice) => format!("In {city}, wear {}.", advice.what_to_wear()),
            Err(e) => format!("No advice for {city}: {e}."),
        }
    }
}

/// A canned API for the demo binary.
pub struct DemoWeather;

impl WeatherApi for DemoWeather {
    fn temperature_c(&self, city: &str) -> Result<f64, WeatherError> {
        match city {
            "Oslo" => Ok(-5.0),
            "London" => Ok(9.5),
            "Madrid" => Ok(31.0),
            "Atlantis" => Err(WeatherError::Unavailable),
            other => Err(WeatherError::UnknownCity(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Exercise 5: your tests here.
}
