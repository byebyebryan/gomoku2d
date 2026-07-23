use super::*;

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
