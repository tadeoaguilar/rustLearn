//! Exercise 5: collecting spans with a `tracing` Layer.
//!
//! A `tracing` subscriber is built from layers; each sees every span open,
//! record and close. This one turns spans into finished trace spans -- name,
//! ids, parent, duration, attributes -- the way OpenTelemetry's tracing
//! bridge does before exporting them to Jaeger or Tempo.
//!
//! Ids come from span fields when the code sets them (`trace_id`, `span_id`,
//! `parent_id`, `service` -- the HTTP middleware does, from `traceparent`);
//! otherwise a span inherits the trace and service of its parent span and
//! gets a fresh span id.

use rand::Rng;
use std::collections::BTreeMap;
use std::fmt::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::Subscriber;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinishedSpan {
    pub name: String,
    pub service: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_id: Option<String>,
    pub duration: Duration,
    pub attributes: BTreeMap<String, String>,
}

/// Kept in the span's extensions while it's open.
struct SpanData {
    service: String,
    trace_id: String,
    span_id: String,
    parent_id: Option<String>,
    attributes: BTreeMap<String, String>,
    started: Instant,
}

#[derive(Default)]
struct Fields(BTreeMap<String, String>);

impl Visit for Fields {
    fn record_str(&mut self, field: &Field, value: &str) {
        todo!("Exercise 5")
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        todo!("Exercise 5")
    }
}

#[derive(Clone, Default)]
pub struct SpanCollector {
    finished: Arc<Mutex<Vec<FinishedSpan>>>,
}

impl SpanCollector {
    pub fn new() -> SpanCollector {
        SpanCollector::default()
    }

    /// Finished spans, in the order they closed.
    pub fn spans(&self) -> Vec<FinishedSpan> {
        self.finished.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.finished.lock().unwrap().clear();
    }
}

impl<S> Layer<S> for SpanCollector
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        todo!("Exercise 5")
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        todo!("Exercise 5")
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        todo!("Exercise 5")
    }
}

/// The spans of one trace as an indented tree, roots first, children in
/// start order:
///
/// ```text
/// frontend http.server http.route=/checkout/{n}
///   frontend http.client
///     backend http.server http.route=/price/{sku}
/// ```
pub fn render_tree(spans: &[FinishedSpan], trace_id: &str) -> String {
    todo!("Exercise 5")
}
