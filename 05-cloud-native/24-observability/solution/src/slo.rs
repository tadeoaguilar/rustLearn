//! Exercise 6: SLOs and burn-rate alerts.
//!
//! An SLO -- "99.9% of requests succeed, over 30 days" -- turns "is it
//! reliable enough?" into arithmetic. The remaining 0.1% is the **error
//! budget**. The **burn rate** says how fast it's being spent: burn rate 1
//! uses exactly the budget over the window; 14.4 would use 2% of a 30-day
//! budget in one hour.
//!
//! Alerting on burn rate (from the Google SRE workbook) pages only for
//! problems that matter, and quickly for big ones. Each alert checks a long
//! window (is it significant?) and a short one (is it still happening?).

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Slo {
    /// e.g. 0.999
    pub objective: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Counts {
    pub good: u64,
    pub total: u64,
}

impl Counts {
    pub fn new(good: u64, total: u64) -> Counts {
        Counts { good, total }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alert {
    None,
    /// Look at it during working hours.
    Ticket,
    /// Wake someone up.
    Page,
}

impl Slo {
    pub fn error_budget(&self) -> f64 {
        1.0 - self.objective
    }

    /// Fraction of good requests; no traffic counts as fully available.
    pub fn availability(c: Counts) -> f64 {
        if c.total == 0 {
            1.0
        } else {
            c.good as f64 / c.total as f64
        }
    }

    /// Error rate divided by the error budget.
    pub fn burn_rate(&self, c: Counts) -> f64 {
        (1.0 - Slo::availability(c)) / self.error_budget()
    }

    /// For counts over the whole SLO window: the fraction of the budget left
    /// (negative once it's overspent).
    pub fn budget_remaining(&self, c: Counts) -> f64 {
        1.0 - self.burn_rate(c)
    }

    /// Multiwindow, multi-burn-rate alerting:
    /// page if burn > 14.4 over 1 h and 5 m; ticket if burn > 6 over 6 h and 30 m.
    pub fn evaluate(
        &self,
        last_5m: Counts,
        last_30m: Counts,
        last_1h: Counts,
        last_6h: Counts,
    ) -> Alert {
        let over = |c: Counts, threshold: f64| self.burn_rate(c) > threshold;
        if over(last_1h, 14.4) && over(last_5m, 14.4) {
            Alert::Page
        } else if over(last_6h, 6.0) && over(last_30m, 6.0) {
            Alert::Ticket
        } else {
            Alert::None
        }
    }
}
