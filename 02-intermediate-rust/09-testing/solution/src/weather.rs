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
        // BUG FIXED: the last check was `t > 25.0`, so exactly 25°C was Mild.
        // Ordering the checks from the top makes each boundary appear once.
        if t >= 25.0 {
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
    use mockall::predicate::eq;

    /// A hand-rolled fake: always returns the same answer. Often all you need.
    struct FixedWeather(Result<f64, WeatherError>);

    impl WeatherApi for FixedWeather {
        fn temperature_c(&self, _city: &str) -> Result<f64, WeatherError> {
            self.0.clone()
        }
    }

    fn advice_at(t: f64) -> Advice {
        Advisor::new(FixedWeather(Ok(t)))
            .advise("anywhere")
            .unwrap()
    }

    #[test]
    fn every_boundary() {
        // Test *at* each boundary and just below it. Bug #4 lives at 25.0.
        assert_eq!(advice_at(-0.1), Advice::Freezing);
        assert_eq!(advice_at(0.0), Advice::Cold);
        assert_eq!(advice_at(9.9), Advice::Cold);
        assert_eq!(advice_at(10.0), Advice::Mild);
        assert_eq!(advice_at(24.9), Advice::Mild);
        assert_eq!(advice_at(25.0), Advice::Hot);
        assert_eq!(advice_at(40.0), Advice::Hot);
    }

    #[test]
    fn errors_are_passed_through() {
        let advisor = Advisor::new(FixedWeather(Err(WeatherError::Unavailable)));
        assert_eq!(advisor.advise("x"), Err(WeatherError::Unavailable));
        assert_eq!(
            advisor.sentence("x"),
            "No advice for x: weather service unavailable."
        );
    }

    #[test]
    fn mock_checks_the_city_and_the_number_of_calls() {
        let mut api = MockWeatherApi::new();
        api.expect_temperature_c()
            .with(eq("Oslo"))
            .times(1) // more or fewer calls fail the test when `api` is dropped
            .returning(|_| Ok(-5.0));
        let advisor = Advisor::new(api);
        assert_eq!(advisor.sentence("Oslo"), "In Oslo, wear a heavy coat.");
    }

    #[test]
    #[should_panic] // the mock panics on a call it wasn't told to expect
    fn mock_rejects_unexpected_arguments() {
        let mut api = MockWeatherApi::new();
        api.expect_temperature_c()
            .with(eq("Oslo"))
            .returning(|_| Ok(0.0));
        let _ = Advisor::new(api).advise("Paris");
    }
}
