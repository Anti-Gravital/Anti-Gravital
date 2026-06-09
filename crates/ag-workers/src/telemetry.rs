//! Observability wiring.
//!
//! Emits `ag_workers_*` metrics through the `metrics` crate when the `metrics` feature
//! is enabled; otherwise every function is a no-op. Metric labels are bounded
//! (`queue`, `kind`, `status`, `outcome`, `reason`); `tenant_id` is never a label
//! (cardinality), per the observability rules.

/// Records the outcome of an enqueue attempt.
pub fn record_enqueue(queue: &str, kind: &str, outcome: &str) {
    emit_counter(
        "ag_workers_enqueue_total",
        &[("queue", queue), ("kind", kind), ("outcome", outcome)],
    );
}

/// Records a job reaching a terminal/processed status.
pub fn record_job(queue: &str, kind: &str, status: &str) {
    emit_counter(
        "ag_workers_jobs_total",
        &[("queue", queue), ("kind", kind), ("status", status)],
    );
}

/// Records a job execution duration in seconds.
pub fn record_duration(queue: &str, kind: &str, seconds: f64) {
    emit_histogram(
        "ag_workers_job_duration_seconds",
        &[("queue", queue), ("kind", kind)],
        seconds,
    );
}

/// Records a retry scheduling.
pub fn record_retry(queue: &str, kind: &str) {
    emit_counter(
        "ag_workers_retry_total",
        &[("queue", queue), ("kind", kind)],
    );
}

/// Records a dead-letter transition with its reason.
pub fn record_dead_letter(queue: &str, kind: &str, reason: &str) {
    emit_counter(
        "ag_workers_dead_letter_total",
        &[("queue", queue), ("kind", kind), ("reason", reason)],
    );
}

/// Records a poison-guard trip (§15.1).
pub fn record_poison_guard(queue: &str, kind: &str) {
    emit_counter(
        "ag_workers_poison_guard_total",
        &[("queue", queue), ("kind", kind)],
    );
}

#[cfg(feature = "metrics")]
fn emit_counter(name: &'static str, labels: &[(&'static str, &str)]) {
    let labels: Vec<metrics::Label> = labels
        .iter()
        .map(|(k, v)| metrics::Label::new(*k, v.to_string()))
        .collect();
    metrics::counter!(name, labels).increment(1);
}

#[cfg(feature = "metrics")]
fn emit_histogram(name: &'static str, labels: &[(&'static str, &str)], value: f64) {
    let labels: Vec<metrics::Label> = labels
        .iter()
        .map(|(k, v)| metrics::Label::new(*k, v.to_string()))
        .collect();
    metrics::histogram!(name, labels).record(value);
}

#[cfg(not(feature = "metrics"))]
fn emit_counter(_name: &'static str, _labels: &[(&'static str, &str)]) {}

#[cfg(not(feature = "metrics"))]
fn emit_histogram(_name: &'static str, _labels: &[(&'static str, &str)], _value: f64) {}
