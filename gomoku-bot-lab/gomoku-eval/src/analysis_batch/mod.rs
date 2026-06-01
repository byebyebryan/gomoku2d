use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use gomoku_bot::tactical::{corridor_active_threats, LocalThreatKind};
use gomoku_core::{Board, Color, Move, Replay};
use rayon::prelude::*;

use crate::analysis::{
    analyze_alternate_defender_reply_options, analyze_replay, defender_reply_roles_for_move,
    replay_frame_annotations_for_analysis, visible_defender_reply_candidates,
    AnalysisBoardSnapshot, AnalysisOptions, DefenderReplyAnalysis, DefenderReplyCandidate,
    DefenderReplyOutcome, DefenderReplyRole, ForcedInterval, GameAnalysis, ProofLimitCause,
    ProofResult, ProofStatus, ReplayFrameAnnotations, ReplayFrameHighlightRole,
    ReplayFrameMarkerRole, RootCause, SearchDiagnostics, ANALYSIS_SCHEMA_VERSION,
};
use crate::report::ReportProvenance;

mod types;

pub use types::{
    AnalysisBatchEntry, AnalysisBatchEntryStatus, AnalysisBatchModel, AnalysisBatchProofDetails,
    AnalysisBatchProofFrame, AnalysisBatchProofMarker, AnalysisBatchProofMarkerKind,
    AnalysisBatchProofSnapshot, AnalysisBatchReport, AnalysisBatchRunOptions, AnalysisBatchSummary,
    ProofLimitCauseCount, PublishedAnalysisEntry, PublishedAnalysisMatchSummary,
    PublishedAnalysisProofDetails, PublishedAnalysisProofFrame, PublishedAnalysisProofMarker,
    PublishedAnalysisProvenance, PublishedAnalysisReplyOutcome, PublishedAnalysisReport,
    PublishedAnalysisSearchDetails, PublishedAnalysisSection, PublishedAnalysisSectionInput,
    ReplayAnalysisInput, PUBLISHED_ANALYSIS_REPORT_SCHEMA_VERSION,
};
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

pub fn published_analysis_report_from_batch(
    source_report: String,
    source_report_provenance: Option<&ReportProvenance>,
    selector: String,
    batch: &AnalysisBatchReport,
    sections: &[PublishedAnalysisSectionInput],
) -> Result<PublishedAnalysisReport, String> {
    let expected_entries = sections
        .iter()
        .map(|section| section.matches.len())
        .sum::<usize>();
    if expected_entries != batch.entries.len() {
        return Err(format!(
            "published analysis section inputs cover {expected_entries} entries, but batch contains {}",
            batch.entries.len()
        ));
    }

    let mut cursor = 0;
    let sections = sections
        .iter()
        .map(|section| {
            let start = cursor;
            let end = start + section.matches.len();
            cursor = end;
            published_analysis_section(section, &batch.entries[start..end])
        })
        .collect::<Vec<_>>();

    Ok(PublishedAnalysisReport {
        schema_version: PUBLISHED_ANALYSIS_REPORT_SCHEMA_VERSION,
        report_kind: "published_analysis".to_string(),
        source_kind: batch.source_kind.clone(),
        source_report,
        provenance: PublishedAnalysisProvenance::from(&ReportProvenance::capture()),
        source_report_provenance: source_report_provenance.map(PublishedAnalysisProvenance::from),
        selector,
        total: batch.total,
        analyzed: batch.analyzed,
        failed: batch.failed,
        elapsed_ms: batch.elapsed_ms,
        total_elapsed_ms: batch.total_elapsed_ms,
        model: batch.model.clone(),
        summary: batch.summary.clone(),
        sections,
    })
}

impl PublishedAnalysisReport {
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

fn published_analysis_section(
    input: &PublishedAnalysisSectionInput,
    entries: &[AnalysisBatchEntry],
) -> PublishedAnalysisSection {
    let mut summary = AnalysisBatchSummary::default();
    let mut analyzed = 0;
    let mut failed = 0;
    for entry in entries {
        if entry.status == AnalysisBatchEntryStatus::Analyzed {
            analyzed += 1;
            increment_summary_from_entry(&mut summary, entry);
        } else {
            failed += 1;
            summary.analysis_error += 1;
        }
    }

    PublishedAnalysisSection {
        label: input.label.clone(),
        entrant_a: input.entrant_a.clone(),
        entrant_b: input.entrant_b.clone(),
        total: entries.len(),
        analyzed,
        failed,
        summary,
        entries: entries
            .iter()
            .zip(input.matches.iter())
            .map(|(entry, match_report)| published_analysis_entry(entry, match_report.clone()))
            .collect(),
    }
}

fn published_analysis_entry(
    entry: &AnalysisBatchEntry,
    match_report: PublishedAnalysisMatchSummary,
) -> PublishedAnalysisEntry {
    PublishedAnalysisEntry {
        path: entry.path.clone(),
        match_report,
        status: entry.status,
        root_cause: entry.root_cause,
        unclear_reason: entry.unclear_reason,
        lethal_onset: entry.lethal_onset.clone(),
        setup_corridor: entry.setup_corridor.clone(),
        last_chance_ply: entry.last_chance_ply,
        critical_loser_ply: entry.critical_loser_ply,
        failure: entry.failure.clone(),
        proof_details: entry
            .proof_details
            .as_ref()
            .map(published_analysis_proof_details),
        search_details: entry
            .proof_detail_diagnostics
            .as_ref()
            .map(published_analysis_search_details),
        elapsed_ms: entry.elapsed_ms,
        error: entry.error.clone(),
    }
}

fn published_analysis_search_details(
    diagnostics: &SearchDiagnostics,
) -> PublishedAnalysisSearchDetails {
    PublishedAnalysisSearchDetails {
        search_nodes: diagnostics.search_nodes,
        branch_probes: diagnostics.branch_probes,
        max_depth_reached: diagnostics.max_depth_reached,
    }
}

fn published_analysis_proof_details(
    details: &AnalysisBatchProofDetails,
) -> PublishedAnalysisProofDetails {
    PublishedAnalysisProofDetails {
        proof_frames: details
            .proof_frames
            .iter()
            .map(published_analysis_proof_frame)
            .collect(),
    }
}

fn published_analysis_proof_frame(frame: &AnalysisBatchProofFrame) -> PublishedAnalysisProofFrame {
    PublishedAnalysisProofFrame {
        label: frame.label.clone(),
        ply: frame.ply,
        side_to_move: frame.side_to_move,
        status: frame.status,
        move_played_notation: frame.move_played_notation.clone(),
        lethal_onset_reached: frame.lethal_onset_reached,
        markers: frame
            .markers
            .iter()
            .map(|marker| PublishedAnalysisProofMarker {
                notation: marker.notation.clone(),
                kinds: marker.kinds.clone(),
            })
            .collect(),
        reply_outcomes: frame
            .reply_outcomes
            .iter()
            .map(|reply| PublishedAnalysisReplyOutcome {
                notation: reply.notation.clone(),
                roles: reply.roles.clone(),
                outcome: reply.outcome,
            })
            .collect(),
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

fn entry_from_analysis(
    path: String,
    analysis: GameAnalysis,
    move_count: usize,
    elapsed_ms: u64,
    replay: Option<&Replay>,
) -> AnalysisBatchEntry {
    let prefixes_analyzed = analysis.proof_summary.len();
    let forced_prefix_count = count_proof_status(&analysis, ProofStatus::ForcedWin);
    let unknown_prefix_count = count_proof_status(&analysis, ProofStatus::Unknown);
    let escape_prefix_count = count_proof_status(&analysis, ProofStatus::EscapeFound);
    let proof_details = replay.and_then(|replay| proof_details_from_analysis(replay, &analysis));
    let proof_detail_diagnostics = proof_details.as_ref().map(proof_details_diagnostics);
    let limit_causes = analysis
        .unclear_context
        .as_ref()
        .map(|context| context.previous_limit_causes.clone())
        .unwrap_or_default();

    AnalysisBatchEntry {
        path,
        status: AnalysisBatchEntryStatus::Analyzed,
        winner: analysis.winner,
        move_count: Some(move_count),
        root_cause: Some(analysis.root_cause),
        unclear_reason: analysis.unclear_reason,
        final_move: analysis.final_move,
        lethal_onset: analysis.lethal_onset,
        setup_corridor: analysis.setup_corridor,
        final_forced_interval_found: analysis.final_forced_interval_found,
        final_forced_interval: Some(analysis.final_forced_interval),
        proof_intervals: analysis.proof_intervals,
        last_chance_ply: analysis.last_chance_ply,
        critical_loser_ply: analysis.critical_loser_ply,
        tactical_notes: analysis.tactical_notes,
        failure: analysis.failure,
        principal_line: analysis.principal_line,
        unknown_gaps: analysis.unknown_gaps.clone(),
        unknown_gap_count: analysis.unknown_gaps.len(),
        unclear_context: analysis.unclear_context,
        proof_details,
        proof_detail_diagnostics,
        limit_causes,
        elapsed_ms,
        prefixes_analyzed,
        forced_prefix_count,
        unknown_prefix_count,
        escape_prefix_count,
        error: None,
    }
}

fn proof_details_from_analysis(
    replay: &Replay,
    analysis: &GameAnalysis,
) -> Option<AnalysisBatchProofDetails> {
    analysis.winner?;
    if analysis.proof_summary.is_empty() {
        return None;
    }

    let boards = replay_prefix_boards(replay).ok()?;
    let scan_start = boards.len().checked_sub(analysis.proof_summary.len())?;
    let final_forced_start_ply = analysis.final_forced_interval.start_ply;
    let previous_prefix_ply = final_forced_start_ply.checked_sub(1);
    let previous_proof_result = previous_prefix_ply
        .and_then(|ply| proof_result_at(&analysis.proof_summary, scan_start, ply));
    let final_start_proof_result =
        proof_result_at(&analysis.proof_summary, scan_start, final_forced_start_ply);
    let previous_proof = previous_prefix_ply
        .zip(previous_proof_result)
        .map(|(ply, proof)| proof_snapshot(ply, proof));
    let final_start_proof =
        final_start_proof_result.map(|proof| proof_snapshot(final_forced_start_ply, proof));
    let mut snapshots = Vec::new();
    if let Some(previous_prefix_ply) = previous_prefix_ply {
        if let Some(board) = boards.get(previous_prefix_ply) {
            snapshots.push(board_snapshot(
                "escape_boundary",
                previous_prefix_ply,
                board,
            ));
        }
    }
    if snapshots
        .iter()
        .all(|snapshot| snapshot.ply != final_forced_start_ply)
    {
        if let Some(board) = boards.get(final_forced_start_ply) {
            snapshots.push(board_snapshot(
                "forced_entry",
                final_forced_start_ply,
                board,
            ));
        }
    }
    let proof_frames = proof_frames_for_actual_interval(replay, &boards, analysis, scan_start);

    Some(AnalysisBatchProofDetails {
        previous_prefix_ply,
        final_forced_start_ply,
        previous_proof,
        final_start_proof,
        snapshots,
        proof_frames,
    })
}

fn proof_details_diagnostics(details: &AnalysisBatchProofDetails) -> SearchDiagnostics {
    let mut diagnostics = SearchDiagnostics::default();
    for reply in details
        .proof_frames
        .iter()
        .flat_map(|frame| frame.reply_outcomes.iter())
    {
        diagnostics.search_nodes += reply.diagnostics.search_nodes;
        diagnostics.branch_probes += reply.diagnostics.branch_probes;
        diagnostics.max_depth_reached = diagnostics
            .max_depth_reached
            .max(reply.diagnostics.max_depth_reached);
    }
    diagnostics
}

fn proof_result_at(
    proofs: &[ProofResult],
    scan_start: usize,
    prefix_ply: usize,
) -> Option<&ProofResult> {
    proofs.get(prefix_ply.checked_sub(scan_start)?)
}

fn proof_snapshot(prefix_ply: usize, proof: &ProofResult) -> AnalysisBatchProofSnapshot {
    let mut escape_replies = proof.escape_moves.clone();
    extend_unique_moves(
        &mut escape_replies,
        proof
            .threat_evidence
            .iter()
            .flat_map(|evidence| evidence.escape_replies.iter().copied()),
    );

    AnalysisBatchProofSnapshot {
        prefix_ply,
        attacker: proof.attacker,
        side_to_move: proof.side_to_move,
        status: proof.status,
        reply_classification: proof
            .threat_evidence
            .first()
            .map(|evidence| evidence.reply_classification),
        winning_squares: collect_evidence_moves(proof, |evidence| &evidence.winning_squares),
        legal_cost_squares: collect_evidence_moves(proof, |evidence| &evidence.legal_cost_squares),
        illegal_cost_squares: collect_evidence_moves(proof, |evidence| {
            &evidence.illegal_cost_squares
        }),
        defender_immediate_wins: collect_evidence_moves(proof, |evidence| {
            &evidence.defender_immediate_wins
        }),
        escape_replies,
        forced_replies: collect_evidence_moves(proof, |evidence| &evidence.forced_replies),
        principal_line: proof.principal_line.clone(),
        principal_line_notation: proof
            .principal_line
            .iter()
            .map(|mv| mv.to_notation())
            .collect(),
        limit_hit: proof.limit_hit,
        limit_causes: proof.limit_causes.clone(),
    }
}

fn proof_frames_for_actual_interval(
    replay: &Replay,
    boards: &[Board],
    analysis: &GameAnalysis,
    scan_start: usize,
) -> Vec<AnalysisBatchProofFrame> {
    let first_ply = proof_frame_start_ply(boards, analysis);
    let replay_annotations = replay_frame_annotations_for_analysis(replay, analysis)
        .unwrap_or_default()
        .into_iter()
        .map(|frame| (frame.ply, frame))
        .collect::<BTreeMap<_, _>>();
    let plys = (first_ply..=analysis.final_forced_interval.end_ply)
        .rev()
        .collect::<Vec<_>>();

    plys.into_iter()
        .filter_map(|ply| {
            let board_ply = ply.checked_sub(1)?;
            let board = boards.get(board_ply)?;
            let proof = proof_result_at(&analysis.proof_summary, scan_start, board_ply);
            let label = actual_frame_label(ply, &analysis.final_forced_interval);
            let mut markers = Vec::new();
            let actual_move = actual_move_at_ply(replay, ply);
            let lethal_onset_reached = lethal_onset_reached_for_frame(analysis, board_ply);
            let reply_candidates =
                defender_reply_candidates_for_frame(board, analysis, actual_move);
            let reply_outcomes = defender_reply_outcomes_for_frame(board, analysis, actual_move);
            if reply_candidates.is_empty() {
                add_loser_tactical_hint_markers(&mut markers, board, analysis.winner);
            } else {
                add_loser_candidate_markers(
                    &mut markers,
                    board,
                    analysis.winner,
                    &reply_candidates,
                );
            }
            add_reply_outcome_markers(&mut markers, &reply_outcomes);
            add_pre_corridor_escape_marker(
                &mut markers,
                PreCorridorEscapeMarkerInput {
                    replay,
                    analysis,
                    ply,
                    board,
                    proof,
                    previous_proof: board_ply.checked_sub(1).and_then(|previous| {
                        proof_result_at(&analysis.proof_summary, scan_start, previous)
                    }),
                    reply_outcomes: &reply_outcomes,
                },
            );
            if let Some(actual_move) = actual_move {
                add_actual_marker(&mut markers, board, analysis.winner, actual_move);
            }
            if let Some(annotation) = replay_annotations.get(&board_ply) {
                add_replay_annotation_markers(&mut markers, annotation);
            }
            markers.sort_by_key(|marker| (marker.mv.row, marker.mv.col));
            Some(proof_frame(ProofFrameInput {
                label: &label,
                ply,
                board,
                status: proof
                    .map(|proof| proof.status)
                    .unwrap_or(ProofStatus::Unknown),
                move_played: actual_move,
                lethal_onset_reached,
                markers,
                reply_outcomes,
            }))
        })
        .collect::<Vec<_>>()
}

fn proof_frame_start_ply(boards: &[Board], analysis: &GameAnalysis) -> usize {
    let start_ply = analysis.final_forced_interval.start_ply;
    let Some(winner) = analysis.winner else {
        return start_ply;
    };
    if start_ply <= 1 {
        return start_ply;
    }

    let start_board_ply = start_ply.saturating_sub(1);
    if boards
        .get(start_board_ply)
        .is_some_and(|board| board.current_player == winner.opponent())
    {
        start_ply - 1
    } else {
        start_ply
    }
}

fn defender_reply_outcomes_for_frame(
    board: &Board,
    analysis: &GameAnalysis,
    actual_move: Option<Move>,
) -> Vec<DefenderReplyAnalysis> {
    let Some(attacker) = analysis.winner else {
        return Vec::new();
    };
    if board.current_player != attacker.opponent() {
        return Vec::new();
    }

    analyze_alternate_defender_reply_options(
        board,
        attacker,
        actual_move,
        &AnalysisOptions {
            max_depth: analysis.model.max_depth,
            max_scan_plies: analysis.model.max_scan_plies,
        },
    )
}

fn defender_reply_candidates_for_frame(
    board: &Board,
    analysis: &GameAnalysis,
    actual_move: Option<Move>,
) -> Vec<DefenderReplyCandidate> {
    let Some(attacker) = analysis.winner else {
        return Vec::new();
    };
    if board.current_player != attacker.opponent() {
        return Vec::new();
    }

    visible_defender_reply_candidates(board, attacker, actual_move)
}

fn actual_frame_label(ply: usize, interval: &ForcedInterval) -> String {
    if ply == interval.end_ply {
        "winning_ply".to_string()
    } else {
        format!("actual_ply_{ply}")
    }
}

fn actual_move_at_ply(replay: &Replay, ply: usize) -> Option<Move> {
    let replay_move = replay.moves.get(ply.checked_sub(1)?)?;
    Move::from_notation(&replay_move.mv).ok()
}

fn add_loser_tactical_hint_markers(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    board: &Board,
    winner: Option<Color>,
) {
    let Some(winner) = winner else {
        return;
    };
    if board.current_player != winner.opponent() {
        return;
    }

    add_marker_kind(
        markers,
        board.immediate_winning_moves_for(board.current_player),
        AnalysisBatchProofMarkerKind::Winning,
    );
    add_marker_kind(
        markers,
        board.immediate_winning_moves_for(board.current_player.opponent()),
        AnalysisBatchProofMarkerKind::Threat,
    );
    if has_immediate_tactical_hint(markers) {
        return;
    }
    add_current_imminent_response_markers(markers, board, winner);
}

fn add_loser_candidate_markers(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    board: &Board,
    winner: Option<Color>,
    candidates: &[DefenderReplyCandidate],
) {
    let Some(winner) = winner else {
        return;
    };
    if board.current_player != winner.opponent() {
        return;
    }

    let defender = winner.opponent();
    for candidate in candidates {
        for role in &candidate.roles {
            if let Some(kind) = marker_kind_for_defender_reply_role(*role, None) {
                add_marker_kind(markers, [candidate.mv], kind);
            }
        }
        if !board.is_legal_for_color(candidate.mv, defender) {
            add_marker_kind(
                markers,
                [candidate.mv],
                AnalysisBatchProofMarkerKind::Forbidden,
            );
        }
    }
}

fn has_immediate_tactical_hint(markers: &[AnalysisBatchProofMarker]) -> bool {
    markers.iter().any(|marker| {
        marker.kinds.iter().any(|kind| {
            matches!(
                kind,
                AnalysisBatchProofMarkerKind::Winning | AnalysisBatchProofMarkerKind::Threat
            )
        })
    })
}

fn add_current_imminent_response_markers(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    board: &Board,
    attacker: Color,
) {
    let defender = attacker.opponent();
    for fact in corridor_active_threats(board, attacker)
        .into_iter()
        .filter(|fact| {
            matches!(
                fact.kind,
                LocalThreatKind::OpenThree | LocalThreatKind::BrokenThree
            )
        })
    {
        for mv in fact.defense_squares.iter().copied() {
            if !board.is_empty(mv.row, mv.col) {
                continue;
            }
            add_marker_kind(markers, [mv], AnalysisBatchProofMarkerKind::ImminentDefense);
            if !board.is_legal_for_color(mv, defender) {
                add_marker_kind(markers, [mv], AnalysisBatchProofMarkerKind::Forbidden);
            }
        }
    }
}

fn add_replay_annotation_markers(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    annotation: &ReplayFrameAnnotations,
) {
    for highlight in &annotation.evidence {
        if let Some(kind) = evidence_marker_kind_for_replay_highlight(highlight.role) {
            add_marker_kind(markers, [highlight.mv], kind);
        }
    }
    for highlight in &annotation.highlights {
        add_marker_kind(
            markers,
            [highlight.mv],
            marker_kind_for_replay_highlight(highlight.role, highlight.side),
        );
    }
    for marker in &annotation.markers {
        add_marker_kind(
            markers,
            [marker.mv],
            marker_kind_for_replay_marker(marker.role),
        );
    }
}

fn marker_kind_for_replay_highlight(
    role: ReplayFrameHighlightRole,
    side: Color,
) -> AnalysisBatchProofMarkerKind {
    match role {
        ReplayFrameHighlightRole::ImmediateWin => AnalysisBatchProofMarkerKind::Winning,
        ReplayFrameHighlightRole::ImmediateThreat => AnalysisBatchProofMarkerKind::Threat,
        ReplayFrameHighlightRole::ImminentThreat => AnalysisBatchProofMarkerKind::ImminentDefense,
        ReplayFrameHighlightRole::CounterThreat => AnalysisBatchProofMarkerKind::OffensiveCounter,
        ReplayFrameHighlightRole::CorridorEntry => corridor_entry_marker_kind(side),
    }
}

fn evidence_marker_kind_for_replay_highlight(
    role: ReplayFrameHighlightRole,
) -> Option<AnalysisBatchProofMarkerKind> {
    match role {
        ReplayFrameHighlightRole::ImmediateWin => {
            Some(AnalysisBatchProofMarkerKind::WinningEvidence)
        }
        ReplayFrameHighlightRole::ImmediateThreat => {
            Some(AnalysisBatchProofMarkerKind::ThreatEvidence)
        }
        ReplayFrameHighlightRole::ImminentThreat => {
            Some(AnalysisBatchProofMarkerKind::ImminentEvidence)
        }
        ReplayFrameHighlightRole::CounterThreat => {
            Some(AnalysisBatchProofMarkerKind::OffensiveEvidence)
        }
        ReplayFrameHighlightRole::CorridorEntry => None,
    }
}

fn marker_kind_for_replay_marker(role: ReplayFrameMarkerRole) -> AnalysisBatchProofMarkerKind {
    match role {
        ReplayFrameMarkerRole::ConfirmedEscape => AnalysisBatchProofMarkerKind::ConfirmedEscape,
        ReplayFrameMarkerRole::PossibleEscape => AnalysisBatchProofMarkerKind::PossibleEscape,
        ReplayFrameMarkerRole::ForcedLoss => AnalysisBatchProofMarkerKind::ForcedLoss,
        ReplayFrameMarkerRole::ImmediateLoss => AnalysisBatchProofMarkerKind::ImmediateLoss,
        ReplayFrameMarkerRole::Forbidden => AnalysisBatchProofMarkerKind::Forbidden,
        ReplayFrameMarkerRole::Unknown => AnalysisBatchProofMarkerKind::UnknownOutcome,
    }
}

struct ProofFrameInput<'a> {
    label: &'a str,
    ply: usize,
    board: &'a Board,
    status: ProofStatus,
    move_played: Option<Move>,
    lethal_onset_reached: bool,
    markers: Vec<AnalysisBatchProofMarker>,
    reply_outcomes: Vec<DefenderReplyAnalysis>,
}

fn proof_frame(input: ProofFrameInput<'_>) -> AnalysisBatchProofFrame {
    AnalysisBatchProofFrame {
        label: input.label.to_string(),
        ply: input.ply,
        side_to_move: input.board.current_player,
        status: input.status,
        move_played: input.move_played,
        move_played_notation: input.move_played.map(Move::to_notation),
        lethal_onset_reached: input.lethal_onset_reached,
        rows: board_rows(input.board),
        markers: input.markers,
        reply_outcomes: input.reply_outcomes,
    }
}

fn lethal_onset_reached_for_frame(analysis: &GameAnalysis, board_ply: usize) -> bool {
    analysis
        .lethal_onset
        .as_ref()
        .is_some_and(|onset| board_ply >= onset.prefix_ply)
}

fn add_reply_outcome_markers(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    replies: &[DefenderReplyAnalysis],
) {
    for reply in replies {
        for role in &reply.roles {
            if let Some(kind) = marker_kind_for_defender_reply_role(
                *role,
                Some(AnalysisBatchProofMarkerKind::Actual),
            ) {
                add_marker_kind(markers, [reply.mv], kind);
            }
        }
        add_marker_kind(
            markers,
            [reply.mv],
            marker_kind_for_defender_reply_outcome(reply.outcome),
        );
    }
}

struct PreCorridorEscapeMarkerInput<'a> {
    replay: &'a Replay,
    analysis: &'a GameAnalysis,
    ply: usize,
    board: &'a Board,
    proof: Option<&'a ProofResult>,
    previous_proof: Option<&'a ProofResult>,
    reply_outcomes: &'a [DefenderReplyAnalysis],
}

fn add_pre_corridor_escape_marker(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    input: PreCorridorEscapeMarkerInput<'_>,
) {
    let Some(winner) = input.analysis.winner else {
        return;
    };
    if input.board.current_player != winner.opponent()
        || !input.reply_outcomes.is_empty()
        || has_visible_tactical_hint(markers)
    {
        return;
    }

    let Some(entry_move) = pre_corridor_escape_entry_move(
        input.replay,
        input.analysis,
        input.ply,
        input.proof,
        input.previous_proof,
    ) else {
        return;
    };
    if !input.board.is_legal(entry_move) {
        return;
    }

    add_marker_kind(markers, [entry_move], corridor_entry_marker_kind(winner));
    add_marker_kind(
        markers,
        [entry_move],
        AnalysisBatchProofMarkerKind::ConfirmedEscape,
    );
}

fn pre_corridor_escape_entry_move(
    replay: &Replay,
    analysis: &GameAnalysis,
    ply: usize,
    proof: Option<&ProofResult>,
    previous_proof: Option<&ProofResult>,
) -> Option<Move> {
    if ply == analysis.final_forced_interval.start_ply
        && proof.map(|proof| proof.status) == Some(ProofStatus::EscapeFound)
    {
        return actual_move_at_ply(replay, ply + 1);
    }

    if ply == analysis.final_forced_interval.start_ply + 1
        && previous_proof.map(|proof| proof.status) == Some(ProofStatus::EscapeFound)
        && proof.map(|proof| proof.status) == Some(ProofStatus::ForcedWin)
    {
        return proof.and_then(|proof| proof.principal_line.first().copied());
    }

    None
}

fn corridor_entry_marker_kind(winner: Color) -> AnalysisBatchProofMarkerKind {
    match winner {
        Color::Black => AnalysisBatchProofMarkerKind::CorridorEntryBlack,
        Color::White => AnalysisBatchProofMarkerKind::CorridorEntryWhite,
    }
}

fn has_visible_tactical_hint(markers: &[AnalysisBatchProofMarker]) -> bool {
    markers
        .iter()
        .flat_map(|marker| marker.kinds.iter().copied())
        .any(is_hint_marker_kind)
}

fn add_actual_marker(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    board: &Board,
    winner: Option<Color>,
    mv: Move,
) {
    add_actual_hint_markers(markers, board, winner, mv);
    let marker = upsert_marker(markers, mv);
    marker.kinds.retain(|kind| is_hint_marker_kind(*kind));
    if !marker.kinds.contains(&AnalysisBatchProofMarkerKind::Actual) {
        marker.kinds.push(AnalysisBatchProofMarkerKind::Actual);
    }
}

fn add_actual_hint_markers(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    board: &Board,
    winner: Option<Color>,
    mv: Move,
) {
    let Some(winner) = winner else {
        return;
    };
    if board.current_player != winner.opponent() {
        return;
    }

    for role in defender_reply_roles_for_move(board, winner, mv) {
        if let Some(kind) = marker_kind_for_defender_reply_role(role, None) {
            add_marker_kind(markers, [mv], kind);
        }
    }
}

fn marker_kind_for_defender_reply_role(
    role: DefenderReplyRole,
    actual_kind: Option<AnalysisBatchProofMarkerKind>,
) -> Option<AnalysisBatchProofMarkerKind> {
    match role {
        DefenderReplyRole::Actual => actual_kind,
        DefenderReplyRole::ImmediateDefense => Some(AnalysisBatchProofMarkerKind::Threat),
        DefenderReplyRole::ImminentDefense => Some(AnalysisBatchProofMarkerKind::ImminentDefense),
        DefenderReplyRole::OffensiveCounter => Some(AnalysisBatchProofMarkerKind::OffensiveCounter),
    }
}

fn marker_kind_for_defender_reply_outcome(
    outcome: DefenderReplyOutcome,
) -> AnalysisBatchProofMarkerKind {
    match outcome {
        DefenderReplyOutcome::ForcedLoss => AnalysisBatchProofMarkerKind::ForcedLoss,
        DefenderReplyOutcome::ConfirmedEscape => AnalysisBatchProofMarkerKind::ConfirmedEscape,
        DefenderReplyOutcome::PossibleEscape => AnalysisBatchProofMarkerKind::PossibleEscape,
        DefenderReplyOutcome::ImmediateLoss => AnalysisBatchProofMarkerKind::ImmediateLoss,
        DefenderReplyOutcome::Unknown => AnalysisBatchProofMarkerKind::UnknownOutcome,
    }
}

fn is_hint_marker_kind(kind: AnalysisBatchProofMarkerKind) -> bool {
    matches!(
        kind,
        AnalysisBatchProofMarkerKind::Winning
            | AnalysisBatchProofMarkerKind::Threat
            | AnalysisBatchProofMarkerKind::ImminentDefense
            | AnalysisBatchProofMarkerKind::OffensiveCounter
            | AnalysisBatchProofMarkerKind::CorridorEntryBlack
            | AnalysisBatchProofMarkerKind::CorridorEntryWhite
    )
}

fn add_marker_kind(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    moves: impl IntoIterator<Item = Move>,
    kind: AnalysisBatchProofMarkerKind,
) {
    for mv in moves {
        let marker = upsert_marker(markers, mv);
        if !marker.kinds.contains(&kind) {
            marker.kinds.push(kind);
        }
    }
}

fn upsert_marker(
    markers: &mut Vec<AnalysisBatchProofMarker>,
    mv: Move,
) -> &mut AnalysisBatchProofMarker {
    if let Some(index) = markers.iter().position(|marker| marker.mv == mv) {
        return &mut markers[index];
    }
    markers.push(AnalysisBatchProofMarker {
        mv,
        notation: mv.to_notation(),
        kinds: Vec::new(),
    });
    markers
        .last_mut()
        .expect("marker was just pushed and must exist")
}

fn collect_evidence_moves(
    proof: &ProofResult,
    selector: fn(&crate::analysis::ThreatSequenceEvidence) -> &[Move],
) -> Vec<Move> {
    let mut moves = Vec::new();
    extend_unique_moves(
        &mut moves,
        proof
            .threat_evidence
            .iter()
            .flat_map(|evidence| selector(evidence).iter().copied()),
    );
    moves
}

fn extend_unique_moves(target: &mut Vec<Move>, moves: impl IntoIterator<Item = Move>) {
    for mv in moves {
        if !target.contains(&mv) {
            target.push(mv);
        }
    }
}

fn replay_prefix_boards(replay: &Replay) -> Result<Vec<Board>, String> {
    let mut board = Board::new(replay.rules.clone());
    let mut boards = vec![board.clone()];
    for (idx, replay_move) in replay.moves.iter().enumerate() {
        let ply = idx + 1;
        let mv = Move::from_notation(&replay_move.mv)
            .map_err(|message| format!("invalid replay move at ply {ply}: {message}"))?;
        board
            .apply_move(mv)
            .map_err(|err| format!("invalid replay move at ply {ply}: {err}"))?;
        boards.push(board.clone());
    }
    Ok(boards)
}

fn board_snapshot(label: &str, ply: usize, board: &Board) -> AnalysisBoardSnapshot {
    AnalysisBoardSnapshot {
        label: label.to_string(),
        ply,
        side_to_move: board.current_player,
        rows: board_rows(board),
    }
}

fn board_rows(board: &Board) -> Vec<String> {
    let size = board.config.board_size;
    (0..size)
        .map(|row| {
            (0..size)
                .map(|col| board.cell(row, col).map_or('.', Color::to_char))
                .collect()
        })
        .collect()
}

fn limit_cause_counts(entries: &[AnalysisBatchEntry]) -> Vec<ProofLimitCauseCount> {
    let mut counts = BTreeMap::<ProofLimitCause, usize>::new();
    for entry in entries {
        for cause in &entry.limit_causes {
            *counts.entry(*cause).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .map(|(cause, count)| ProofLimitCauseCount { cause, count })
        .collect()
}

fn count_proof_status(analysis: &GameAnalysis, status: ProofStatus) -> usize {
    analysis
        .proof_summary
        .iter()
        .filter(|proof| proof.status == status)
        .count()
}

fn increment_summary_from_entry(summary: &mut AnalysisBatchSummary, entry: &AnalysisBatchEntry) {
    if entry.winner.is_none() {
        summary.ongoing_or_draw += 1;
        return;
    }

    if matches!(entry.root_cause, Some(RootCause::Unclear) | None) {
        summary.unclear += 1;
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use gomoku_bot::tactical::LethalThreatKind;
    use gomoku_core::{Board, Color, Move, Replay, RuleConfig, Variant};

    use super::{
        add_actual_marker, add_loser_candidate_markers, add_reply_outcome_markers,
        defender_reply_candidates_for_frame, defender_reply_outcomes_for_frame,
        published_analysis_report_from_batch, run_analysis_batch, run_analysis_batch_replays,
        run_analysis_batch_replays_with_options, AnalysisBatchEntry, AnalysisBatchEntryStatus,
        AnalysisBatchModel, AnalysisBatchProofDetails, AnalysisBatchProofFrame,
        AnalysisBatchProofMarker, AnalysisBatchProofMarkerKind, AnalysisBatchReport,
        AnalysisBatchRunOptions, AnalysisBatchSummary, PublishedAnalysisMatchSummary,
        PublishedAnalysisSectionInput, ReplayAnalysisInput,
        PUBLISHED_ANALYSIS_REPORT_SCHEMA_VERSION,
    };
    use crate::analysis::{
        analyze_replay, replay_frame_annotations_for_analysis, AnalysisModel, AnalysisOptions,
        DefenderReplyAnalysis, DefenderReplyCandidate, DefenderReplyOutcome, DefenderReplyRole,
        FailureMode, ForcedInterval, GameAnalysis, ProofLimitCause, ProofStatus,
        ReplayFrameHighlightRole, ReplayFrameMarkerRole, ReplyClassification, RootCause,
        SearchDiagnostics, UnclearReason, ANALYSIS_SCHEMA_VERSION,
    };

    fn replay_from_moves(variant: Variant, moves: &[&str]) -> Replay {
        let rules = RuleConfig {
            variant,
            ..RuleConfig::default()
        };
        let mut board = Board::new(rules.clone());
        let mut replay = Replay::new(rules, "Black", "White");

        for notation in moves {
            let parsed = Move::from_notation(notation).expect("test move notation should parse");
            board
                .apply_move(parsed)
                .expect("test replay move should be legal");
            replay.push_move(parsed, 0, board.hash(), None);
        }
        replay.finish(&board.result, Some(0));
        replay
    }

    fn mv(notation: &str) -> Move {
        Move::from_notation(notation).expect("test move notation should parse")
    }

    fn board_from_moves(variant: Variant, moves: &[&str]) -> Board {
        let mut board = Board::new(RuleConfig {
            variant,
            ..RuleConfig::default()
        });
        for notation in moves {
            board
                .apply_move(mv(notation))
                .expect("test board move should be legal");
        }
        board
    }

    fn analysis_for_winner(winner: Color, rule_set: &str, max_depth: usize) -> GameAnalysis {
        GameAnalysis {
            schema_version: ANALYSIS_SCHEMA_VERSION,
            rule_set: rule_set.to_string(),
            winner: Some(winner),
            loser: Some(winner.opponent()),
            final_move: None,
            final_winning_line: Vec::new(),
            model: AnalysisModel {
                rule_set: rule_set.to_string(),
                max_depth,
                max_scan_plies: Some(64),
            },
            lethal_onset: None,
            setup_corridor: None,
            final_forced_interval_found: false,
            final_forced_interval: ForcedInterval {
                start_ply: 0,
                end_ply: 0,
            },
            proof_intervals: Vec::new(),
            unknown_gaps: Vec::new(),
            unclear_reason: None,
            unclear_context: None,
            last_chance_ply: None,
            decisive_attack_ply: None,
            critical_loser_ply: None,
            root_cause: RootCause::Unclear,
            failure: None,
            tactical_notes: Vec::new(),
            principal_line: Vec::new(),
            proof_summary: Vec::new(),
        }
    }

    fn reply_candidate(notation: &str, roles: Vec<DefenderReplyRole>) -> DefenderReplyCandidate {
        let mv = mv(notation);
        DefenderReplyCandidate {
            mv,
            notation: mv.to_notation(),
            roles,
        }
    }

    fn reply_analysis(
        notation: &str,
        roles: Vec<DefenderReplyRole>,
        outcome: DefenderReplyOutcome,
    ) -> DefenderReplyAnalysis {
        let mv = mv(notation);
        DefenderReplyAnalysis {
            mv,
            notation: mv.to_notation(),
            roles,
            outcome,
            principal_line: Vec::new(),
            principal_line_notation: Vec::new(),
            limit_causes: Vec::new(),
            diagnostics: SearchDiagnostics::default(),
        }
    }

    fn proof_frame_with_markers(
        side_to_move: Color,
        markers: Vec<AnalysisBatchProofMarker>,
        reply_outcomes: Vec<DefenderReplyAnalysis>,
    ) -> AnalysisBatchProofFrame {
        AnalysisBatchProofFrame {
            label: "test_frame".to_string(),
            ply: 0,
            side_to_move,
            status: ProofStatus::ForcedWin,
            move_played: None,
            move_played_notation: None,
            lethal_onset_reached: false,
            rows: Vec::new(),
            markers,
            reply_outcomes,
        }
    }

    fn temp_report_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gomoku-analysis-batch-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp report dir should be created");
        dir
    }

    #[test]
    fn published_analysis_report_keeps_ui_frames_and_drops_debug_details() {
        let mut frame = proof_frame_with_markers(
            Color::White,
            vec![AnalysisBatchProofMarker {
                mv: mv("H8"),
                notation: "H8".to_string(),
                kinds: vec![AnalysisBatchProofMarkerKind::Threat],
            }],
            vec![reply_analysis(
                "H8",
                vec![DefenderReplyRole::ImmediateDefense],
                DefenderReplyOutcome::ForcedLoss,
            )],
        );
        frame.rows = vec!["debug board row".to_string()];
        let interval = ForcedInterval {
            start_ply: 1,
            end_ply: 2,
        };
        let entry = AnalysisBatchEntry {
            path: "match_0001__bot_a__vs__bot_b".to_string(),
            status: AnalysisBatchEntryStatus::Analyzed,
            winner: Some(Color::Black),
            move_count: Some(2),
            root_cause: Some(RootCause::CorridorEntry),
            unclear_reason: None,
            final_move: Some(mv("H8")),
            lethal_onset: None,
            setup_corridor: Some(interval.clone()),
            final_forced_interval_found: true,
            final_forced_interval: Some(interval.clone()),
            proof_intervals: vec![interval],
            last_chance_ply: Some(1),
            critical_loser_ply: Some(2),
            tactical_notes: Vec::new(),
            failure: None,
            principal_line: Vec::new(),
            unknown_gaps: Vec::new(),
            unknown_gap_count: 0,
            unclear_context: None,
            proof_details: Some(AnalysisBatchProofDetails {
                previous_prefix_ply: Some(1),
                final_forced_start_ply: 2,
                previous_proof: None,
                final_start_proof: None,
                snapshots: Vec::new(),
                proof_frames: vec![frame],
            }),
            proof_detail_diagnostics: Some(SearchDiagnostics {
                search_nodes: 99,
                branch_probes: 3,
                max_depth_reached: 4,
            }),
            limit_causes: Vec::new(),
            elapsed_ms: 7,
            prefixes_analyzed: 1,
            forced_prefix_count: 1,
            unknown_prefix_count: 0,
            escape_prefix_count: 0,
            error: None,
        };
        let batch = AnalysisBatchReport {
            schema_version: ANALYSIS_SCHEMA_VERSION,
            source_kind: "report_replays".to_string(),
            source: "outputs/full-report.json:Preset triangle".to_string(),
            replay_dir: "outputs/full-report.json:Preset triangle".to_string(),
            total: 1,
            analyzed: 1,
            failed: 0,
            elapsed_ms: 7,
            total_elapsed_ms: 7,
            model: AnalysisBatchModel {
                max_depth: 4,
                max_scan_plies: Some(64),
            },
            summary: AnalysisBatchSummary::default(),
            limit_cause_counts: Vec::new(),
            entries: vec![entry],
        };
        let published = published_analysis_report_from_batch(
            "outputs/full-report.json".to_string(),
            None,
            "Preset triangle".to_string(),
            &batch,
            &[PublishedAnalysisSectionInput {
                label: "Easy vs Normal".to_string(),
                entrant_a: "search-d1".to_string(),
                entrant_b: "search-d3+pattern-eval".to_string(),
                matches: vec![PublishedAnalysisMatchSummary {
                    match_index: 1,
                    black: "bot-a".to_string(),
                    white: "bot-b".to_string(),
                    result: "black_won".to_string(),
                    winner: Some("bot-a".to_string()),
                    end_reason: "win".to_string(),
                    move_cells: vec![112, 113],
                    move_count: 2,
                }],
            }],
        )
        .expect("published analysis report should build");
        let json = published
            .to_json()
            .expect("published analysis report should serialize");

        assert_eq!(published.report_kind, "published_analysis");
        assert_eq!(
            published.schema_version,
            PUBLISHED_ANALYSIS_REPORT_SCHEMA_VERSION
        );
        assert!(json.contains("\"provenance\""));
        assert_eq!(published.sections[0].entries[0].match_report.match_index, 1);
        assert!(json.contains("\"proof_frames\""));
        assert!(json.contains("\"markers\""));
        assert!(json.contains("\"reply_outcomes\""));
        assert!(!json.contains("proof_detail_diagnostics"));
        assert!(json.contains("\"search_details\""));
        assert!(json.contains("\"search_nodes\""));
        assert!(json.contains("\"branch_probes\""));
        assert!(json.contains("\"max_depth_reached\""));
        assert!(!json.contains("debug board row"));
    }

    #[test]
    fn analysis_batch_groups_replay_directory_by_root_cause() {
        let dir = temp_report_dir("root-cause");
        let missed_defense = replay_from_moves(
            Variant::Freestyle,
            &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
        );
        fs::write(
            dir.join("missed_defense.json"),
            missed_defense
                .to_json()
                .expect("test replay should serialize"),
        )
        .expect("test replay should write");

        let report = run_analysis_batch(&dir, AnalysisOptions::default())
            .expect("batch analysis should run");

        assert_eq!(report.total, 1);
        assert_eq!(report.analyzed, 1);
        assert_eq!(report.failed, 0);
        assert_eq!(report.model.max_depth, AnalysisOptions::default().max_depth);
        assert_eq!(report.summary.unclear, 0);
        assert_eq!(report.entries[0].root_cause, Some(RootCause::MissedDefense));
        assert_eq!(
            report.entries[0]
                .failure
                .as_ref()
                .map(|failure| failure.mode),
            Some(FailureMode::MissedImmediateResponse)
        );
        assert_eq!(report.entries[0].path, "missed_defense.json");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn analysis_batch_replays_preserves_input_order_and_records_work_metrics() {
        let first = replay_from_moves(
            Variant::Freestyle,
            &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
        );
        let second = replay_from_moves(
            Variant::Freestyle,
            &["H8", "A1", "I8", "A2", "J8", "A3", "K8", "A4", "L8"],
        );

        let report = run_analysis_batch_replays(
            "report.json:bot-a vs bot-b".to_string(),
            vec![
                ReplayAnalysisInput {
                    label: "match_0002".to_string(),
                    replay: second,
                },
                ReplayAnalysisInput {
                    label: "match_0001".to_string(),
                    replay: first,
                },
            ],
            AnalysisOptions::default(),
        );

        assert_eq!(report.source_kind, "report_replays");
        assert_eq!(report.source, "report.json:bot-a vs bot-b");
        assert_eq!(report.entries[0].path, "match_0002");
        assert_eq!(report.entries[1].path, "match_0001");
        assert!(report.entries[0].proof_details.is_none());
        assert!(report.entries[0].final_forced_interval_found);
        assert!(
            report.entries[0].unclear_reason.is_some()
                || report.entries[0].root_cause != Some(RootCause::Unclear)
        );
        assert_eq!(
            report.entries[0].prefixes_analyzed,
            report.entries[0].forced_prefix_count
                + report.entries[0].unknown_prefix_count
                + report.entries[0].escape_prefix_count
        );
        assert_eq!(
            report.total_elapsed_ms,
            report
                .entries
                .iter()
                .map(|entry| entry.elapsed_ms)
                .sum::<u64>()
        );
    }

    #[test]
    fn analysis_batch_replays_records_scan_cap_drilldown_context() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &["H8", "A1", "I8", "A2", "J8", "A3", "K8", "B1", "L8"],
        );

        let report = run_analysis_batch_replays(
            "report.json:bot-a vs bot-b".to_string(),
            vec![ReplayAnalysisInput {
                label: "proof_limit_case".to_string(),
                replay,
            }],
            AnalysisOptions {
                max_scan_plies: Some(0),
                ..AnalysisOptions::default()
            },
        );

        let entry = &report.entries[0];
        let context = entry
            .unclear_context
            .as_ref()
            .expect("scan-cap-limited entries should expose drilldown context");

        assert_eq!(entry.unclear_reason, Some(UnclearReason::ScanWindowCutoff));
        assert_eq!(context.previous_prefix_ply, Some(8));
        assert_eq!(context.previous_proof_status, None);
        assert_eq!(context.previous_proof_limit_hit, None);
        assert!(context
            .previous_limit_causes
            .contains(&ProofLimitCause::OutsideScanWindow));
        assert!(entry
            .limit_causes
            .contains(&ProofLimitCause::OutsideScanWindow));
        assert!(report
            .limit_cause_counts
            .iter()
            .any(|count| count.cause == ProofLimitCause::OutsideScanWindow && count.count == 1));
        assert_eq!(context.move_count, 9);
        assert!(context.principal_line.is_empty());
        assert!(context.principal_line_notation.is_empty());
        assert!(context
            .snapshots
            .iter()
            .any(|snapshot| snapshot.label == "previous_prefix" && snapshot.ply == 8));
    }

    #[test]
    fn analysis_batch_replays_can_include_decisive_proof_details() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &["H8", "G8", "I8", "A1", "J8", "A2", "K8", "B1", "L8"],
        );

        let report = run_analysis_batch_replays_with_options(
            "report.json:bot-a vs bot-b".to_string(),
            vec![ReplayAnalysisInput {
                label: "missed_defense".to_string(),
                replay,
            }],
            AnalysisBatchRunOptions {
                analysis: AnalysisOptions::default(),
                include_proof_details: true,
            },
        );

        let entry = &report.entries[0];
        assert_eq!(entry.root_cause, Some(RootCause::MissedDefense));
        let details = entry
            .proof_details
            .as_ref()
            .expect("opt-in proof details should be recorded for decisive entries");

        assert_eq!(details.previous_prefix_ply, Some(7));
        assert_eq!(details.final_forced_start_ply, 8);

        let previous = details
            .previous_proof
            .as_ref()
            .expect("previous prefix proof should be available");
        assert_eq!(previous.prefix_ply, 7);
        assert_eq!(previous.status, ProofStatus::EscapeFound);
        assert_eq!(
            previous.reply_classification,
            Some(ReplyClassification::ConfirmedEscape)
        );
        assert_eq!(
            previous.escape_replies,
            vec![Move::from_notation("L8").unwrap()]
        );
        assert_eq!(
            previous.winning_squares,
            vec![Move::from_notation("L8").unwrap()]
        );

        let final_start = details
            .final_start_proof
            .as_ref()
            .expect("final forced start proof should be available");
        assert_eq!(final_start.prefix_ply, 8);
        assert_eq!(final_start.status, ProofStatus::ForcedWin);
        assert_eq!(
            final_start.principal_line,
            vec![Move::from_notation("L8").unwrap()]
        );
        assert_eq!(final_start.principal_line_notation, vec!["L8".to_string()]);

        assert!(details
            .snapshots
            .iter()
            .any(|snapshot| snapshot.label == "escape_boundary" && snapshot.ply == 7));
        assert!(details
            .snapshots
            .iter()
            .any(|snapshot| snapshot.label == "forced_entry" && snapshot.ply == 8));

        assert_eq!(
            details
                .proof_frames
                .iter()
                .map(|frame| (frame.label.as_str(), frame.ply))
                .collect::<Vec<_>>(),
            vec![("winning_ply", 9), ("actual_ply_8", 8), ("actual_ply_7", 7)]
        );
        assert!(details
            .proof_frames
            .iter()
            .all(
                |frame| frame.markers.iter().all(|marker| marker.kinds.iter().all(
                    |kind| matches!(
                        kind,
                        AnalysisBatchProofMarkerKind::Winning
                            | AnalysisBatchProofMarkerKind::Threat
                            | AnalysisBatchProofMarkerKind::ImminentDefense
                            | AnalysisBatchProofMarkerKind::OffensiveCounter
                            | AnalysisBatchProofMarkerKind::WinningEvidence
                            | AnalysisBatchProofMarkerKind::ThreatEvidence
                            | AnalysisBatchProofMarkerKind::ImminentEvidence
                            | AnalysisBatchProofMarkerKind::OffensiveEvidence
                            | AnalysisBatchProofMarkerKind::CorridorEntryBlack
                            | AnalysisBatchProofMarkerKind::CorridorEntryWhite
                            | AnalysisBatchProofMarkerKind::Forbidden
                            | AnalysisBatchProofMarkerKind::ForcedLoss
                            | AnalysisBatchProofMarkerKind::ConfirmedEscape
                            | AnalysisBatchProofMarkerKind::PossibleEscape
                            | AnalysisBatchProofMarkerKind::ImmediateLoss
                            | AnalysisBatchProofMarkerKind::UnknownOutcome
                            | AnalysisBatchProofMarkerKind::Actual
                    )
                ))
            ));
        let winning_frame = details
            .proof_frames
            .first()
            .expect("winning-ply frame should be first");
        assert_eq!(winning_frame.side_to_move, Color::Black);
        let actual_l8 = winning_frame
            .markers
            .iter()
            .find(|marker| marker.notation == "L8")
            .expect("winning frame should mark the actual winning move");
        assert!(actual_l8
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::Actual));
        assert_eq!(actual_l8.kinds, vec![AnalysisBatchProofMarkerKind::Actual]);

        let attacker_frame = details
            .proof_frames
            .iter()
            .find(|frame| frame.label == "actual_ply_7" && frame.ply == 7)
            .expect("winner-side setup frame should be recorded");
        assert_eq!(attacker_frame.side_to_move, Color::Black);
        assert!(attacker_frame.markers.iter().all(|marker| {
            marker
                .kinds
                .iter()
                .all(|kind| matches!(kind, AnalysisBatchProofMarkerKind::Actual))
        }));

        let final_frame = details
            .proof_frames
            .iter()
            .find(|frame| frame.label == "actual_ply_8" && frame.ply == 8)
            .expect("forced-interval decision frame should be recorded");
        assert_eq!(final_frame.side_to_move, Color::White);
        assert_eq!(final_frame.move_played_notation.as_deref(), Some("B1"));
        let final_actual = final_frame
            .markers
            .iter()
            .find(|marker| marker.notation == "B1")
            .expect("final frame should mark the actual replay move");
        assert!(final_actual
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::Actual));
        assert_eq!(
            final_actual.kinds,
            vec![AnalysisBatchProofMarkerKind::Actual]
        );
        let final_l8 = final_frame
            .markers
            .iter()
            .find(|marker| marker.notation == "L8")
            .expect("final frame should mark the L8 losing square");
        assert!(final_l8
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::Threat));
    }

    #[test]
    fn analysis_batch_report_uses_perspective_status_after_lethal_onset() {
        let replay = replay_from_moves(
            Variant::Freestyle,
            &[
                "H8", "G8", "I8", "A1", "J8", "O1", "K8", "A15", "I7", "O15", "I9", "L8", "I6",
                "A14", "I10",
            ],
        );

        let report = run_analysis_batch_replays_with_options(
            "report.json:bot-a vs bot-b".to_string(),
            vec![ReplayAnalysisInput {
                label: "lethal_onset".to_string(),
                replay,
            }],
            AnalysisBatchRunOptions {
                analysis: AnalysisOptions::default(),
                include_proof_details: true,
            },
        );

        let entry = &report.entries[0];
        let onset = entry
            .lethal_onset
            .as_ref()
            .expect("analysis entry should carry lethal onset evidence");
        assert_eq!(onset.prefix_ply, 11);
        assert_eq!(onset.kind, LethalThreatKind::OneStepCoverage);
        assert_eq!(onset.shape.label, "4x3");
        let setup_corridor = entry
            .setup_corridor
            .as_ref()
            .expect("analysis entry should carry setup corridor evidence");
        assert_eq!(setup_corridor.end_ply, onset.prefix_ply);

        let details = entry
            .proof_details
            .as_ref()
            .expect("proof details should be recorded");
        let onset_frame = details
            .proof_frames
            .iter()
            .find(|frame| frame.ply == 12)
            .expect("onset frame should be recorded");
        assert_eq!(onset_frame.ply, 12);
        assert!(onset_frame.lethal_onset_reached);
        let l8 = marker_for(onset_frame, "L8");
        assert!(l8.kinds.contains(&AnalysisBatchProofMarkerKind::Threat));
        let i6 = marker_for(onset_frame, "I6");
        assert!(i6.kinds.contains(&AnalysisBatchProofMarkerKind::Threat));
        assert!(i6
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ImmediateLoss));
        let i10 = marker_for(onset_frame, "I10");
        assert!(i10.kinds.contains(&AnalysisBatchProofMarkerKind::Threat));
        assert!(i10
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ImmediateLoss));
        assert!(marker_for(onset_frame, "H8")
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ThreatEvidence));
        assert!(marker_for(onset_frame, "I7")
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ThreatEvidence));
    }

    #[test]
    fn analysis_batch_marks_pre_corridor_entry_as_escape_target() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "H9", "I8", "I9", "G8", "J8", "G6", "H10", "K7", "G11", "F12", "F10", "J9",
                "H7", "E8", "F8", "F7", "H5", "C10", "D9", "G5", "G7", "H6", "E9", "D8", "G9",
                "F9", "E7", "D6", "I11",
            ],
        );

        let report = run_analysis_batch_replays_with_options(
            "report.json:bot-a vs bot-b".to_string(),
            vec![ReplayAnalysisInput {
                label: "pre_corridor_escape".to_string(),
                replay,
            }],
            AnalysisBatchRunOptions {
                analysis: AnalysisOptions {
                    max_depth: 4,
                    max_scan_plies: Some(8),
                },
                include_proof_details: true,
            },
        );

        let details = report.entries[0]
            .proof_details
            .as_ref()
            .expect("proof details should be recorded");
        assert_eq!(details.final_forced_start_ply, 23);
        let frame = details
            .proof_frames
            .iter()
            .find(|frame| frame.label == "actual_ply_23")
            .expect("pre-corridor escape frame should be present");
        assert_eq!(frame.side_to_move, Color::Black);
        assert_eq!(frame.status, ProofStatus::EscapeFound);
        assert!(frame.reply_outcomes.is_empty());

        let e9 = frame
            .markers
            .iter()
            .find(|marker| marker.notation == "E9")
            .expect("winner corridor entry should be shown as an escape target");
        assert!(e9
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::CorridorEntryWhite));
        assert!(e9
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ConfirmedEscape));
    }

    #[test]
    fn analysis_batch_marks_attacker_started_corridor_entry_as_escape_target() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "H9", "I8", "I9", "G8", "J8", "J9", "K7", "H10", "H7", "G9", "L6", "M5",
                "I7", "G7", "G6", "F8", "E8", "E7", "D6", "I11",
            ],
        );

        let report = run_analysis_batch_replays_with_options(
            "report.json:bot-a vs bot-b".to_string(),
            vec![ReplayAnalysisInput {
                label: "attacker_started_corridor_entry".to_string(),
                replay,
            }],
            AnalysisBatchRunOptions {
                analysis: AnalysisOptions {
                    max_depth: 4,
                    max_scan_plies: Some(64),
                },
                include_proof_details: true,
            },
        );

        let details = report.entries[0]
            .proof_details
            .as_ref()
            .expect("proof details should be recorded");
        assert_eq!(details.final_forced_start_ply, 13);
        let frame = details
            .proof_frames
            .iter()
            .find(|frame| frame.label == "actual_ply_14")
            .expect("defender frame after attacker corridor entry should be present");
        assert_eq!(frame.side_to_move, Color::White);
        assert_eq!(frame.status, ProofStatus::ForcedWin);
        assert!(frame.reply_outcomes.is_empty());

        let g7 = frame
            .markers
            .iter()
            .find(|marker| marker.notation == "G7")
            .expect("winner corridor entry should be shown as an escape target");
        assert!(g7
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::CorridorEntryBlack));
        assert!(g7
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ConfirmedEscape));
    }

    #[test]
    fn shared_replay_annotations_match_report_corridor_entry_boundary() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "H9", "I8", "I9", "G8", "J8", "J9", "K7", "H10", "H7", "G9", "L6", "M5",
                "I7", "G7", "G6", "F8", "E8", "E7", "D6", "I11",
            ],
        );
        let options = AnalysisOptions {
            max_depth: 4,
            max_scan_plies: Some(64),
        };
        let analysis = analyze_replay(&replay, options).expect("analysis should run");
        let annotations = replay_frame_annotations_for_analysis(&replay, &analysis)
            .expect("shared replay annotations should build");

        let boundary = annotations
            .iter()
            .find(|frame| frame.ply == 13)
            .expect("shared annotation should include the report corridor-entry boundary");
        assert!(boundary.highlights.iter().any(|highlight| {
            highlight.role == ReplayFrameHighlightRole::CorridorEntry
                && highlight.notation == "G7"
                && highlight.side == Color::Black
        }));
        assert!(boundary.markers.iter().any(|marker| {
            marker.role == ReplayFrameMarkerRole::ConfirmedEscape
                && marker.notation == "G7"
                && marker.side == Color::White
        }));
    }

    #[test]
    fn analysis_batch_actual_marker_keeps_counter_hint() {
        let board = board_from_moves(
            Variant::Renju,
            &[
                "H8", "G7", "H6", "H7", "I7", "F7", "G5", "F4", "J8", "K9", "I8", "G8", "I6", "I9",
                "H9", "F6", "F5", "G9", "G10",
            ],
        );
        assert_eq!(board.current_player, Color::White);

        let mut markers = Vec::new();
        add_actual_marker(&mut markers, &board, Some(Color::Black), mv("E7"));

        let marker = proof_marker_for(&markers, "E7");
        assert!(marker
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::OffensiveCounter));
        assert!(marker.kinds.contains(&AnalysisBatchProofMarkerKind::Actual));
    }

    #[test]
    fn analysis_batch_reply_markers_combine_roles_and_outcomes() {
        let board = board_from_moves(Variant::Renju, &["H8"]);
        let mut markers = Vec::new();
        add_loser_candidate_markers(
            &mut markers,
            &board,
            Some(Color::Black),
            &[reply_candidate(
                "G7",
                vec![
                    DefenderReplyRole::ImminentDefense,
                    DefenderReplyRole::Actual,
                ],
            )],
        );
        add_actual_marker(&mut markers, &board, Some(Color::Black), mv("G7"));

        let replies = vec![
            reply_analysis(
                "G4",
                vec![DefenderReplyRole::ImminentDefense],
                DefenderReplyOutcome::ForcedLoss,
            ),
            reply_analysis(
                "G9",
                vec![DefenderReplyRole::ImminentDefense],
                DefenderReplyOutcome::ForcedLoss,
            ),
            reply_analysis(
                "I10",
                vec![DefenderReplyRole::OffensiveCounter],
                DefenderReplyOutcome::PossibleEscape,
            ),
            reply_analysis(
                "I11",
                vec![DefenderReplyRole::OffensiveCounter],
                DefenderReplyOutcome::ForcedLoss,
            ),
        ];
        add_reply_outcome_markers(&mut markers, &replies);
        let frame = proof_frame_with_markers(Color::White, markers, replies);
        let frame = &frame;

        let g4 = marker_for(frame, "G4");
        assert!(g4
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ImminentDefense));
        assert!(g4.kinds.contains(&AnalysisBatchProofMarkerKind::ForcedLoss));

        let g9 = marker_for(frame, "G9");
        assert!(g9
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ImminentDefense));
        assert!(g9.kinds.contains(&AnalysisBatchProofMarkerKind::ForcedLoss));

        let g7 = marker_for(frame, "G7");
        assert_eq!(
            g7.kinds,
            vec![
                AnalysisBatchProofMarkerKind::ImminentDefense,
                AnalysisBatchProofMarkerKind::Actual,
            ]
        );
        assert!(!frame
            .reply_outcomes
            .iter()
            .any(|reply| reply.notation == "G7"));

        let i10 = marker_for(frame, "I10");
        assert!(i10
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::OffensiveCounter));
        assert!(i10
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::PossibleEscape));

        let i11 = marker_for(frame, "I11");
        assert!(i11
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::OffensiveCounter));
        assert!(i11
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ForcedLoss));
    }

    #[test]
    fn analysis_batch_visual_frames_filter_forbidden_costs_to_current_prefix() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "G7", "H9", "J8", "H7", "H6", "I7", "F8", "E9", "H10", "I5", "G9", "E7",
                "I9", "K7", "G10", "G11", "I11", "J12", "G6", "G8", "F6", "L7", "J7", "J6", "E6",
                "D6", "I6",
            ],
        );

        let report = run_analysis_batch_replays_with_options(
            "report.json:bot-a vs bot-b".to_string(),
            vec![ReplayAnalysisInput {
                label: "renju_forbidden_cost_prefix_scope".to_string(),
                replay,
            }],
            AnalysisBatchRunOptions {
                analysis: AnalysisOptions {
                    max_depth: 4,
                    max_scan_plies: Some(64),
                },
                include_proof_details: true,
            },
        );

        let frames = &report.entries[0]
            .proof_details
            .as_ref()
            .expect("proof details should be present")
            .proof_frames;
        for label in ["actual_ply_24", "actual_ply_26"] {
            let frame = frames
                .iter()
                .find(|frame| frame.label == label)
                .unwrap_or_else(|| panic!("{label} frame should be present"));
            if let Some(i6) = frame.markers.iter().find(|marker| marker.notation == "I6") {
                assert!(
                    !i6.kinds.contains(&AnalysisBatchProofMarkerKind::Forbidden),
                    "{label} must not mark I6 using future forbidden-cost evidence: {:?}",
                    i6.kinds
                );
            }
        }
    }

    #[test]
    fn analysis_batch_actual_marker_keeps_imminent_hint() {
        let board = board_from_moves(
            Variant::Renju,
            &[
                "H8", "I8", "H6", "G7", "H7", "H9", "J7", "G4", "G5", "I7", "I5", "H5", "I6", "I9",
                "J6", "K6", "J4",
            ],
        );
        let actual = mv("J5");
        let analysis = analysis_for_winner(Color::Black, "renju", 0);
        let candidates = defender_reply_candidates_for_frame(&board, &analysis, Some(actual));
        let mut markers = Vec::new();
        add_loser_candidate_markers(&mut markers, &board, analysis.winner, &candidates);
        add_actual_marker(&mut markers, &board, analysis.winner, actual);

        let marker = proof_marker_for(&markers, "J5");
        assert_eq!(
            marker.kinds,
            vec![
                AnalysisBatchProofMarkerKind::ImminentDefense,
                AnalysisBatchProofMarkerKind::Actual,
            ]
        );
    }

    #[test]
    fn analysis_batch_actual_marker_keeps_far_open_three_defense_hint() {
        let board = board_from_moves(
            Variant::Renju,
            &[
                "H8", "H9", "J8", "I7", "I8", "G8", "I10", "F7", "K8", "L8", "J9", "L7", "J7",
                "J10", "L9", "I6", "H7", "G6", "M10", "N11", "J6", "J5", "K9",
            ],
        );
        let actual = mv("N9");
        let mut markers = Vec::new();
        add_actual_marker(&mut markers, &board, Some(Color::Black), actual);

        let marker = proof_marker_for(&markers, "N9");
        assert_eq!(
            marker.kinds,
            vec![
                AnalysisBatchProofMarkerKind::ImminentDefense,
                AnalysisBatchProofMarkerKind::Actual,
            ]
        );
    }

    #[test]
    fn analysis_batch_visual_frames_mark_both_forbidden_open_three_responses() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "G7", "H9", "J8", "H7", "H6", "I7", "F8", "E9", "H10", "I5", "G9", "E7",
                "I9", "K7", "G10", "G11", "I11", "J12", "G6", "G8", "F6", "L7", "J7", "J6", "E6",
                "D6", "I6",
            ],
        );

        let report = run_analysis_batch_replays_with_options(
            "report.json:bot-a vs bot-b".to_string(),
            vec![ReplayAnalysisInput {
                label: "renju_forbidden_open_three_responses".to_string(),
                replay,
            }],
            AnalysisBatchRunOptions {
                analysis: AnalysisOptions {
                    max_depth: 4,
                    max_scan_plies: Some(64),
                },
                include_proof_details: true,
            },
        );

        let frame = report.entries[0]
            .proof_details
            .as_ref()
            .expect("proof details should be present")
            .proof_frames
            .iter()
            .find(|frame| frame.label == "actual_ply_25")
            .expect("ply 25 decision frame should be present");

        for notation in ["E6", "I6"] {
            let marker = marker_for(frame, notation);
            assert!(
                marker
                    .kinds
                    .contains(&AnalysisBatchProofMarkerKind::ImminentDefense),
                "{notation} should be marked as an imminent open-three response: {:?}",
                marker.kinds
            );
            assert!(
                marker
                    .kinds
                    .contains(&AnalysisBatchProofMarkerKind::Forbidden),
                "{notation} should be marked forbidden for Black under Renju: {:?}",
                marker.kinds
            );
        }
    }

    #[test]
    fn analysis_batch_candidate_markers_prioritize_immediate_threats_over_imminent_responses() {
        let board = board_from_moves(
            Variant::Renju,
            &[
                "H8", "I8", "H7", "I7", "H6", "H5", "I6", "I9", "G6", "J6", "G8", "J5", "E6", "F6",
                "H9",
            ],
        );
        let actual = mv("H10");
        let analysis = analysis_for_winner(Color::Black, "renju", 0);
        let candidates = defender_reply_candidates_for_frame(&board, &analysis, Some(actual));
        let mut markers = Vec::new();
        add_loser_candidate_markers(&mut markers, &board, analysis.winner, &candidates);
        add_actual_marker(&mut markers, &board, analysis.winner, actual);

        let h10 = proof_marker_for(&markers, "H10");
        assert!(h10.kinds.contains(&AnalysisBatchProofMarkerKind::Threat));
        assert!(h10.kinds.contains(&AnalysisBatchProofMarkerKind::Actual));
        assert!(
            !markers.iter().any(|marker| marker
                .kinds
                .contains(&AnalysisBatchProofMarkerKind::ImminentDefense)),
            "imminent responses should be suppressed while an immediate threat response exists: {:?}",
            markers
        );
    }

    #[test]
    fn analysis_batch_visual_frames_probe_all_imminent_combo_replies() {
        let board = board_from_moves(
            Variant::Renju,
            &[
                "H8", "H7", "F8", "G9", "G8", "I8", "G6", "D9", "F9", "F10", "D7", "G10", "F7",
                "E10", "E8", "D8", "C6", "B5", "D10", "F11", "F6", "F5", "D6", "E6", "H5", "I4",
            ],
        );
        assert_eq!(board.current_player, Color::Black);

        let analysis = analysis_for_winner(Color::White, "renju", 0);

        let reply_outcomes = defender_reply_outcomes_for_frame(&board, &analysis, Some(mv("C8")));
        for notation in ["J7", "H9", "E12", "G12"] {
            assert!(
                reply_outcomes
                    .iter()
                    .any(|reply| reply.notation == notation),
                "{notation} should be probed as a non-actual 3+3 reply: {:?}",
                reply_outcomes
            );
        }
        assert!(
            !reply_outcomes.iter().any(|reply| reply.notation == "C8"),
            "the actual replay move is inherited from replay context, not re-probed: {:?}",
            reply_outcomes
        );

        let mut markers = Vec::new();
        add_reply_outcome_markers(&mut markers, &reply_outcomes);
        for notation in ["J7", "H9", "E12"] {
            let marker = proof_marker_for(&markers, notation);
            assert!(
                marker
                    .kinds
                    .contains(&AnalysisBatchProofMarkerKind::ImminentDefense),
                "{notation} should keep its imminent-response hint box: {:?}",
                marker.kinds
            );
            assert!(
                marker.kinds.iter().any(|kind| matches!(
                    kind,
                    AnalysisBatchProofMarkerKind::ForcedLoss
                        | AnalysisBatchProofMarkerKind::ConfirmedEscape
                        | AnalysisBatchProofMarkerKind::PossibleEscape
                        | AnalysisBatchProofMarkerKind::ImmediateLoss
                        | AnalysisBatchProofMarkerKind::UnknownOutcome
                )),
                "{notation} should carry a proof outcome marker: {:?}",
                marker.kinds
            );
        }
        assert!(
            markers.iter().all(|marker| marker.notation != "C8"),
            "actual replay move should not be re-probed: {markers:?}"
        );
    }

    #[test]
    fn analysis_batch_visual_frames_do_not_show_lower_tier_forbidden_replies_during_immediate_threats(
    ) {
        let board = board_from_moves(
            Variant::Renju,
            &[
                "H8", "I8", "H10", "G9", "H9", "H7", "J9", "G12", "G10", "I10", "H11", "H12",
                "I12", "F9", "I6", "E10", "J11", "C12", "D11", "D9", "H13", "E9", "C9", "F11",
                "C8", "K10", "C7", "C10", "J10", "J8", "I9", "K11", "K9", "L9", "L8", "M7", "F6",
                "G7", "H6", "G6", "F7", "E12", "C5", "C6", "J12", "J13", "F15", "G14", "E8", "F12",
                "D12", "F10",
            ],
        );
        assert_eq!(board.current_player, Color::Black);
        let analysis = analysis_for_winner(Color::White, "renju", 0);
        let reply_outcomes = defender_reply_outcomes_for_frame(&board, &analysis, Some(mv("F8")));
        assert!(
            reply_outcomes
                .iter()
                .any(|reply| reply.notation == "F13"
                    && reply.roles.contains(&DefenderReplyRole::ImmediateDefense)),
            "the active proof candidate should be the immediate-threat response: {reply_outcomes:?}"
        );
        assert!(
            reply_outcomes.iter().all(|reply| reply.notation != "D8"),
            "lower-tier forbidden imminent replies should not be probed: {reply_outcomes:?}"
        );

        let reply_candidates =
            defender_reply_candidates_for_frame(&board, &analysis, Some(mv("F8")));
        let mut markers = Vec::new();
        add_loser_candidate_markers(&mut markers, &board, analysis.winner, &reply_candidates);
        add_reply_outcome_markers(&mut markers, &reply_outcomes);

        assert!(
            markers.iter().all(|marker| marker.notation != "D8"),
            "lower-tier forbidden imminent replies should not be marked while immediate threats are active: {markers:?}"
        );
        let f13 = proof_marker_for(&markers, "F13");
        assert!(f13.kinds.contains(&AnalysisBatchProofMarkerKind::Threat));
        assert!(f13
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ImmediateLoss));
    }

    #[test]
    fn analysis_batch_visual_frames_mark_renju_forbidden_blocks() {
        let replay = replay_from_moves(
            Variant::Renju,
            &[
                "H8", "H7", "J8", "I9", "I8", "G8", "F9", "F7", "H9", "G7", "I7", "E7", "D7", "G9",
                "G6", "G11", "K8", "G10",
            ],
        );

        let report = run_analysis_batch_replays_with_options(
            "report.json:bot-a vs bot-b".to_string(),
            vec![ReplayAnalysisInput {
                label: "renju_forbidden_block".to_string(),
                replay,
            }],
            AnalysisBatchRunOptions {
                analysis: AnalysisOptions {
                    max_depth: 4,
                    max_scan_plies: Some(8),
                },
                include_proof_details: true,
            },
        );

        let frames = &report.entries[0]
            .proof_details
            .as_ref()
            .expect("proof details should be present")
            .proof_frames;
        let turn_16_17 = frames
            .iter()
            .find(|frame| frame.label == "actual_ply_17")
            .expect("ply 17 decision frame should be present");
        let g10 = marker_for(turn_16_17, "G10");
        assert!(g10.kinds.contains(&AnalysisBatchProofMarkerKind::Threat));
        assert!(g10.kinds.contains(&AnalysisBatchProofMarkerKind::Forbidden));

        let turn_14_15 = frames
            .iter()
            .find(|frame| frame.label == "actual_ply_15")
            .expect("ply 15 decision frame should be present");
        let future_g10 = marker_for(turn_14_15, "G10");
        assert!(future_g10
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::ImminentDefense));
        assert!(future_g10
            .kinds
            .contains(&AnalysisBatchProofMarkerKind::Forbidden));
    }

    fn marker_for<'a>(
        frame: &'a AnalysisBatchProofFrame,
        notation: &str,
    ) -> &'a AnalysisBatchProofMarker {
        frame
            .markers
            .iter()
            .find(|marker| marker.notation == notation)
            .unwrap_or_else(|| panic!("expected marker {notation}"))
    }

    fn proof_marker_for<'a>(
        markers: &'a [AnalysisBatchProofMarker],
        notation: &str,
    ) -> &'a AnalysisBatchProofMarker {
        markers
            .iter()
            .find(|marker| marker.notation == notation)
            .unwrap_or_else(|| panic!("expected marker {notation}"))
    }
}
