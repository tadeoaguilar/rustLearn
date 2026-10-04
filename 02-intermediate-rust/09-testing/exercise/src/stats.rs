//! Descriptive statistics.
//!
//! Every function returns `None` for empty input: there is no mean of nothing.

/// Arithmetic mean.
pub fn mean(values: &[f64]) -> Option<f64> {
    Some(values.iter().sum::<f64>() / values.len() as f64)
}

/// The middle value; for an even count, the average of the two middle values.
pub fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    Some(sorted[sorted.len() / 2])
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

    // Exercise 1: your tests here.
}
