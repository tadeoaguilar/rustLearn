//! Descriptive statistics.
//!
//! Every function returns `None` for empty input: there is no mean of nothing.

/// Arithmetic mean.
///
/// ```
/// use m09_testing_solution::stats::mean;
/// assert_eq!(mean(&[1.0, 2.0, 3.0, 4.0]), Some(2.5));
/// assert_eq!(mean(&[]), None);
/// ```
pub fn mean(values: &[f64]) -> Option<f64> {
    // BUG FIXED: without this check, 0.0 / 0.0 = NaN, returned as Some(NaN).
    if values.is_empty() {
        return None;
    }
    Some(values.iter().sum::<f64>() / values.len() as f64)
}

/// The middle value; for an even count, the average of the two middle values.
///
/// ```
/// use m09_testing_solution::stats::median;
/// assert_eq!(median(&[3.0, 1.0, 2.0]), Some(2.0));
/// assert_eq!(median(&[4.0, 1.0, 3.0, 2.0]), Some(2.5));
/// ```
pub fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mid = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        // BUG FIXED: used to return sorted[mid], the upper middle value.
        Some((sorted[mid - 1] + sorted[mid]) / 2.0)
    } else {
        Some(sorted[mid])
    }
}

/// The most frequent value. Ties go to the smallest value, so the result
/// doesn't depend on HashMap iteration order.
pub fn mode(values: &[i64]) -> Option<i64> {
    let mut counts = std::collections::HashMap::new();
    for &v in values {
        *counts.entry(v).or_insert(0usize) += 1;
    }
    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0)))
        .map(|(v, _)| v)
}

/// Population standard deviation.
pub fn std_dev(values: &[f64]) -> Option<f64> {
    let m = mean(values)?;
    let variance = values.iter().map(|v| (v - m).powi(2)).sum::<f64>() / values.len() as f64;
    Some(variance.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn mean_of_simple_values() {
        assert_eq!(mean(&[1.0, 2.0, 3.0]), Some(2.0));
    }

    #[test]
    fn mean_of_one_value_is_that_value() {
        assert_eq!(mean(&[42.5]), Some(42.5));
    }

    #[test]
    fn mean_of_nothing_is_none_not_nan() {
        // Bug #1. `Some(NaN)` would compare unequal to everything -- including
        // itself -- so assert_eq!(mean(&[]), None) is the precise check.
        assert_eq!(mean(&[]), None);
    }

    #[test]
    fn median_of_odd_length_is_the_middle_value() {
        assert_eq!(median(&[5.0, 1.0, 3.0]), Some(3.0));
    }

    #[test]
    fn median_of_even_length_averages_the_middle_two() {
        // Bug #2.
        assert_eq!(median(&[1.0, 2.0, 3.0, 4.0]), Some(2.5));
        assert_eq!(median(&[10.0, 20.0]), Some(15.0));
    }

    #[test]
    fn median_does_not_care_about_input_order() {
        assert_eq!(median(&[4.0, 1.0, 3.0, 2.0]), median(&[1.0, 2.0, 3.0, 4.0]));
    }

    #[test]
    fn median_of_nothing_is_none() {
        assert_eq!(median(&[]), None);
    }

    #[test]
    fn mode_picks_most_frequent_and_smallest_on_ties() {
        assert_eq!(mode(&[1, 2, 2, 3]), Some(2));
        assert_eq!(mode(&[3, 1, 3, 1]), Some(1), "tie between 1 and 3");
        assert_eq!(mode(&[]), None);
    }

    #[test]
    fn std_dev_of_known_values() {
        // The classic example: mean 5, population std dev exactly 2.
        let sd = std_dev(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]).unwrap();
        assert!(approx(sd, 2.0), "expected 2.0, got {sd}");
        assert_eq!(std_dev(&[3.0, 3.0]), Some(0.0));
        assert_eq!(std_dev(&[]), None);
    }
}
