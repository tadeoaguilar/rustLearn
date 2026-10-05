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
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.0
            .insert(field.name().to_string(), format!("{value:?}"));
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
        let mut fields = Fields::default();
        attrs.record(&mut fields);
        let mut f = fields.0;
        let span = ctx.span(id).expect("the span exists");
        // The parent span's trace data, if it has any.
        let inherited = span.parent().and_then(|p| {
            p.extensions()
                .get::<SpanData>()
                .map(|d| (d.trace_id.clone(), d.span_id.clone(), d.service.clone()))
        });
        let (trace_id, parent_from_span, service) = match (f.remove("trace_id"), inherited) {
            (Some(t), inh) => (
                t,
                inh.as_ref().map(|i| i.1.clone()),
                f.remove("service").or(inh.map(|i| i.2)).unwrap_or_default(),
            ),
            (None, Some((t, parent, svc))) => (t, Some(parent), f.remove("service").unwrap_or(svc)),
            (None, None) => return, // not part of a trace: don't collect
        };
        let span_id = f
            .remove("span_id")
            .unwrap_or_else(|| format!("{:016x}", rand::thread_rng().gen_range(1..=u64::MAX)));
        // An explicit parent_id (from an incoming traceparent) wins; "" means root.
        let parent_id = match f.remove("parent_id") {
            Some(p) if p.is_empty() => None,
            Some(p) => Some(p),
            None => parent_from_span,
        };
        f.retain(|_, v| !v.is_empty());
        span.extensions_mut().insert(SpanData {
            service,
            trace_id,
            span_id,
            parent_id,
            attributes: f,
            started: Instant::now(),
        });
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("the span exists");
        let mut fields = Fields::default();
        values.record(&mut fields);
        if let Some(data) = span.extensions_mut().get_mut::<SpanData>() {
            data.attributes.extend(fields.0);
        }
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        let span = ctx.span(&id).expect("the span exists");
        let Some(data) = span.extensions_mut().remove::<SpanData>() else {
            return;
        };
        self.finished.lock().unwrap().push(FinishedSpan {
            name: span.name().to_string(),
            service: data.service,
            trace_id: data.trace_id,
            span_id: data.span_id,
            parent_id: data.parent_id,
            duration: data.started.elapsed(),
            attributes: data.attributes,
        });
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
    let in_trace: Vec<&FinishedSpan> = spans.iter().filter(|s| s.trace_id == trace_id).collect();
    let ids: Vec<&str> = in_trace.iter().map(|s| s.span_id.as_str()).collect();
    let mut out = String::new();
    fn walk(
        out: &mut String,
        all: &[&FinishedSpan],
        parent: Option<&str>,
        ids: &[&str],
        depth: usize,
    ) {
        let mut children: Vec<&&FinishedSpan> = all
            .iter()
            .filter(|s| match (parent, s.parent_id.as_deref()) {
                (None, None) => true,
                (None, Some(p)) => !ids.contains(&p), // parent not collected here (another process): a local root
                (Some(want), Some(p)) => p == want,
                (Some(_), None) => false,
            })
            .collect();
        // Spans are collected as they *close*; siblings that set an `order`
        // attribute are shown in that order.
        children.sort_by_key(|s| {
            s.attributes
                .get("order")
                .and_then(|o| o.parse::<u64>().ok())
                .unwrap_or(u64::MAX)
        });
        for s in children {
            let route = s
                .attributes
                .get("http.route")
                .map(|r| format!(" http.route={r}"))
                .unwrap_or_default();
            let _ = writeln!(out, "{}{} {}{route}", "  ".repeat(depth), s.service, s.name);
            walk(out, all, Some(&s.span_id), ids, depth + 1);
        }
    }
    walk(&mut out, &in_trace, None, &ids, 0);
    out
}
