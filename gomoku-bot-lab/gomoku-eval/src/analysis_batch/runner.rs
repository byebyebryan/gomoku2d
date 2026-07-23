use super::*;

pub fn run_analysis_batch(
    replay_dir: &Path,
    options: AnalysisOptions,
) -> Result<AnalysisBatchReport, String> {
    run_analysis_batch_with_options(replay_dir, options.into())
}

pub fn run_analysis_batch_with_options(
    replay_dir: &Path,
    options: AnalysisBatchRunOptions,
) -> Result<AnalysisBatchReport, String> {
    run_analysis_batch_with_progress(replay_dir, options, None)
}

pub fn run_analysis_batch_with_progress(
    replay_dir: &Path,
    options: AnalysisBatchRunOptions,
    progress_interval: Option<usize>,
) -> Result<AnalysisBatchReport, String> {
    let batch_started = Instant::now();
    let model = model_from_options(&options);
    let mut paths = replay_paths(replay_dir)?;
    paths.sort();
    let progress = AnalysisBatchProgress::new(paths.len(), progress_interval, "replay analyses");

    let mut summary = AnalysisBatchSummary::default();
    let mut analyzed = 0;
    let mut failed = 0;

    let entries = paths
        .par_iter()
        .map(|path| {
            let entry_started = Instant::now();
            let relative_path = path
                .strip_prefix(replay_dir)
                .unwrap_or(path)
                .display()
                .to_string();
            let entry = match analyze_replay_file(path, options.analysis.clone()) {
                Ok((replay, analysis)) => entry_from_analysis(
                    relative_path,
                    analysis,
                    replay.moves.len(),
                    elapsed_millis(entry_started.elapsed()),
                    options.include_proof_details.then_some(&replay),
                ),
                Err(error) => error_entry(
                    relative_path,
                    error,
                    elapsed_millis(entry_started.elapsed()),
                ),
            };
            progress.record_completion();
            entry
        })
        .collect::<Vec<_>>();

    for entry in &entries {
        if entry.status == AnalysisBatchEntryStatus::Analyzed {
            analyzed += 1;
            increment_summary_from_entry(&mut summary, entry);
        } else {
            failed += 1;
            summary.analysis_error += 1;
        }
    }
    let total_elapsed_ms = entries.iter().map(|entry| entry.elapsed_ms).sum();
    let limit_cause_counts = limit_cause_counts(&entries);

    Ok(AnalysisBatchReport {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        source_kind: "replay_dir".to_string(),
        source: replay_dir.display().to_string(),
        replay_dir: replay_dir.display().to_string(),
        total: entries.len(),
        analyzed,
        failed,
        elapsed_ms: elapsed_millis(batch_started.elapsed()),
        total_elapsed_ms,
        model,
        summary,
        limit_cause_counts,
        entries,
    })
}

pub fn run_analysis_batch_replays(
    source: String,
    inputs: Vec<ReplayAnalysisInput>,
    options: AnalysisOptions,
) -> AnalysisBatchReport {
    run_analysis_batch_replays_with_options(source, inputs, options.into())
}

pub fn run_analysis_batch_replays_with_options(
    source: String,
    inputs: Vec<ReplayAnalysisInput>,
    options: AnalysisBatchRunOptions,
) -> AnalysisBatchReport {
    run_analysis_batch_replays_with_progress(source, inputs, options, None)
}

pub fn run_analysis_batch_replays_with_progress(
    source: String,
    inputs: Vec<ReplayAnalysisInput>,
    options: AnalysisBatchRunOptions,
    progress_interval: Option<usize>,
) -> AnalysisBatchReport {
    let batch_started = Instant::now();
    let model = model_from_options(&options);
    let progress = AnalysisBatchProgress::new(inputs.len(), progress_interval, "replay analyses");

    let entries = inputs
        .par_iter()
        .map(|input| {
            let entry_started = Instant::now();
            let entry = match analyze_replay(&input.replay, options.analysis.clone()) {
                Ok(analysis) => entry_from_analysis(
                    input.label.clone(),
                    analysis,
                    input.replay.moves.len(),
                    elapsed_millis(entry_started.elapsed()),
                    options.include_proof_details.then_some(&input.replay),
                ),
                Err(error) => error_entry(
                    input.label.clone(),
                    error.to_string(),
                    elapsed_millis(entry_started.elapsed()),
                ),
            };
            progress.record_completion();
            entry
        })
        .collect::<Vec<_>>();

    let mut summary = AnalysisBatchSummary::default();
    let mut analyzed = 0;
    let mut failed = 0;
    for entry in &entries {
        if entry.status == AnalysisBatchEntryStatus::Analyzed {
            analyzed += 1;
            increment_summary_from_entry(&mut summary, entry);
        } else {
            failed += 1;
            summary.analysis_error += 1;
        }
    }
    let total_elapsed_ms = entries.iter().map(|entry| entry.elapsed_ms).sum();
    let limit_cause_counts = limit_cause_counts(&entries);

    AnalysisBatchReport {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        source_kind: "report_replays".to_string(),
        replay_dir: source.clone(),
        source,
        total: entries.len(),
        analyzed,
        failed,
        elapsed_ms: elapsed_millis(batch_started.elapsed()),
        total_elapsed_ms,
        model,
        summary,
        limit_cause_counts,
        entries,
    }
}

struct AnalysisBatchProgress {
    completed: AtomicUsize,
    interval: Option<usize>,
    label: &'static str,
    started: Instant,
    total: usize,
}

impl AnalysisBatchProgress {
    fn new(total: usize, interval: Option<usize>, label: &'static str) -> Self {
        let interval = interval.filter(|interval| total > 0 && *interval > 0);
        if let Some(interval) = interval {
            eprintln!("Progress: every {interval} completed {label}");
            eprintln!("Progress: 0/{total} {label} complete");
        }
        Self {
            completed: AtomicUsize::new(0),
            interval,
            label,
            started: Instant::now(),
            total,
        }
    }

    fn record_completion(&self) {
        let Some(interval) = self.interval else {
            return;
        };
        let done = self.completed.fetch_add(1, Ordering::Relaxed) + 1;
        if done != self.total && !done.is_multiple_of(interval) {
            return;
        }

        let elapsed = self.started.elapsed().as_secs_f64();
        let progress = done as f64 * 100.0 / self.total as f64;
        let eta_secs = if done > 0 && done < self.total && elapsed > 0.0 {
            let entries_per_sec = done as f64 / elapsed;
            Some((self.total - done) as f64 / entries_per_sec)
        } else {
            None
        };
        match eta_secs {
            Some(eta_secs) => eprintln!(
                "Progress: {done}/{} {} complete ({progress:.1}%, elapsed {:.0}s, ETA {:.0}s)",
                self.total, self.label, elapsed, eta_secs
            ),
            None => eprintln!(
                "Progress: {done}/{} {} complete ({progress:.1}%, elapsed {:.0}s)",
                self.total, self.label, elapsed
            ),
        }
    }
}

fn model_from_options(options: &AnalysisBatchRunOptions) -> AnalysisBatchModel {
    AnalysisBatchModel {
        max_depth: options.analysis.max_depth,
        max_scan_plies: options.analysis.max_scan_plies,
    }
}

fn elapsed_millis(duration: std::time::Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}

fn error_entry(path: String, error: String, elapsed_ms: u64) -> AnalysisBatchEntry {
    AnalysisBatchEntry {
        path,
        status: AnalysisBatchEntryStatus::Error,
        winner: None,
        move_count: None,
        root_cause: None,
        unclear_reason: None,
        final_move: None,
        lethal_onset: None,
        setup_corridor: None,
        final_forced_interval_found: false,
        final_forced_interval: None,
        proof_intervals: Vec::new(),
        last_chance_ply: None,
        critical_loser_ply: None,
        tactical_notes: Vec::new(),
        failure: None,
        principal_line: Vec::new(),
        unknown_gaps: Vec::new(),
        unknown_gap_count: 0,
        unclear_context: None,
        proof_details: None,
        proof_detail_diagnostics: None,
        limit_causes: Vec::new(),
        elapsed_ms,
        prefixes_analyzed: 0,
        forced_prefix_count: 0,
        unknown_prefix_count: 0,
        escape_prefix_count: 0,
        error: Some(error),
    }
}

fn replay_paths(replay_dir: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let entries = std::fs::read_dir(replay_dir)
        .map_err(|err| format!("failed to read replay directory: {err}"))?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| format!("failed to read replay directory entry: {err}"))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some("json") {
            paths.push(path);
        }
    }
    Ok(paths)
}

fn analyze_replay_file(
    path: &Path,
    options: AnalysisOptions,
) -> Result<(Replay, GameAnalysis), String> {
    let json = std::fs::read_to_string(path)
        .map_err(|err| format!("failed to read replay JSON: {err}"))?;
    let replay =
        Replay::from_json(&json).map_err(|err| format!("failed to parse replay: {err}"))?;
    let analysis = analyze_replay(&replay, options)
        .map_err(|err| format!("failed to analyze replay: {err}"))?;
    Ok((replay, analysis))
}
