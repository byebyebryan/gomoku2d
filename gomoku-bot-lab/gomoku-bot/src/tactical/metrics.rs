use std::cell::Cell as MetricCell;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TacticalMetrics {
    pub renju_effective_filter_calls: u64,
    pub renju_effective_filter_ns: u64,
    pub renju_effective_filter_continuation_checks: u64,
    pub renju_effective_filter_continuation_ns: u64,
    pub compound_imminent_queries: u64,
    pub compound_imminent_ns: u64,
    pub compound_imminent_prefilter_candidates: u64,
    pub compound_imminent_confirmed_entries: u64,
    pub compound_imminent_hits: u64,
}

thread_local! {
    static TACTICAL_METRICS: MetricCell<TacticalMetrics> = const { MetricCell::new(TacticalMetrics {
        renju_effective_filter_calls: 0,
        renju_effective_filter_ns: 0,
        renju_effective_filter_continuation_checks: 0,
        renju_effective_filter_continuation_ns: 0,
        compound_imminent_queries: 0,
        compound_imminent_ns: 0,
        compound_imminent_prefilter_candidates: 0,
        compound_imminent_confirmed_entries: 0,
        compound_imminent_hits: 0,
    }) };
}

pub fn tactical_metrics_snapshot() -> TacticalMetrics {
    TACTICAL_METRICS.with(MetricCell::get)
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn record_renju_effective_filter(elapsed: std::time::Duration) {
    TACTICAL_METRICS.with(|metrics| {
        let mut current = metrics.get();
        current.renju_effective_filter_calls =
            current.renju_effective_filter_calls.saturating_add(1);
        current.renju_effective_filter_ns = current
            .renju_effective_filter_ns
            .saturating_add(u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX).max(1));
        metrics.set(current);
    });
}

#[cfg(target_arch = "wasm32")]
pub(super) fn record_renju_effective_filter() {
    TACTICAL_METRICS.with(|metrics| {
        let mut current = metrics.get();
        current.renju_effective_filter_calls =
            current.renju_effective_filter_calls.saturating_add(1);
        metrics.set(current);
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn record_renju_effective_filter_continuation(elapsed: std::time::Duration) {
    TACTICAL_METRICS.with(|metrics| {
        let mut current = metrics.get();
        current.renju_effective_filter_continuation_checks = current
            .renju_effective_filter_continuation_checks
            .saturating_add(1);
        current.renju_effective_filter_continuation_ns = current
            .renju_effective_filter_continuation_ns
            .saturating_add(u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX).max(1));
        metrics.set(current);
    });
}

#[cfg(target_arch = "wasm32")]
pub(super) fn record_renju_effective_filter_continuation() {
    TACTICAL_METRICS.with(|metrics| {
        let mut current = metrics.get();
        current.renju_effective_filter_continuation_checks = current
            .renju_effective_filter_continuation_checks
            .saturating_add(1);
        metrics.set(current);
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn record_compound_imminent_query(
    elapsed: std::time::Duration,
    prefilter_candidates: usize,
    confirmed_entries: usize,
) {
    TACTICAL_METRICS.with(|metrics| {
        let mut current = metrics.get();
        current.compound_imminent_queries = current.compound_imminent_queries.saturating_add(1);
        current.compound_imminent_ns = current
            .compound_imminent_ns
            .saturating_add(u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX).max(1));
        current.compound_imminent_prefilter_candidates = current
            .compound_imminent_prefilter_candidates
            .saturating_add(prefilter_candidates as u64);
        current.compound_imminent_confirmed_entries = current
            .compound_imminent_confirmed_entries
            .saturating_add(confirmed_entries as u64);
        if confirmed_entries > 0 {
            current.compound_imminent_hits = current.compound_imminent_hits.saturating_add(1);
        }
        metrics.set(current);
    });
}

#[cfg(target_arch = "wasm32")]
pub(super) fn record_compound_imminent_query(
    prefilter_candidates: usize,
    confirmed_entries: usize,
) {
    TACTICAL_METRICS.with(|metrics| {
        let mut current = metrics.get();
        current.compound_imminent_queries = current.compound_imminent_queries.saturating_add(1);
        current.compound_imminent_prefilter_candidates = current
            .compound_imminent_prefilter_candidates
            .saturating_add(prefilter_candidates as u64);
        current.compound_imminent_confirmed_entries = current
            .compound_imminent_confirmed_entries
            .saturating_add(confirmed_entries as u64);
        if confirmed_entries > 0 {
            current.compound_imminent_hits = current.compound_imminent_hits.saturating_add(1);
        }
        metrics.set(current);
    });
}
