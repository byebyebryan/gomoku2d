use super::*;

pub(super) fn resolve_report_replay_entrants(
    standing_bots: &[String],
    entrant_a: Option<String>,
    entrant_b: Option<String>,
) -> Result<(String, String), String> {
    match (entrant_a, entrant_b) {
        (Some(a), Some(b)) if a == b => Err("report replay entrants must be different".to_string()),
        (Some(a), Some(b)) => Ok((a, b)),
        (Some(a), None) => {
            let b = highest_different_standing(standing_bots, &a)
                .ok_or_else(|| format!("Tournament report has no standing different from {a}."))?;
            Ok((a, b))
        }
        (None, Some(b)) => {
            let a = highest_different_standing(standing_bots, &b)
                .ok_or_else(|| format!("Tournament report has no standing different from {b}."))?;
            Ok((a, b))
        }
        (None, None) => {
            let a = standing_bots
                .first()
                .cloned()
                .ok_or_else(|| "Tournament report has no standing #1.".to_string())?;
            let b = highest_different_standing(standing_bots, &a)
                .ok_or_else(|| "Tournament report has no standing #2.".to_string())?;
            Ok((a, b))
        }
    }
}

pub(super) struct ReportReplaySectionPlan<'a> {
    pub(super) label: String,
    pub(super) entrant_a: String,
    pub(super) entrant_b: String,
    pub(super) selections: Vec<ReportReplaySelection<'a>>,
}

pub(super) const PRESET_EASY_BOT: &str = "search-d1";
pub(super) const PRESET_NORMAL_BOT: &str = "search-d3+pattern-eval";
pub(super) const PRESET_HARD_BOT: &str =
    "search-d7+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w4";

pub(super) fn report_replay_section_plans<'a>(
    report_source: &'a ReportReplaySource,
    selector: CliReportReplaySelector,
    entrant_a: Option<String>,
    entrant_b: Option<String>,
    sample_size: usize,
) -> Result<Vec<ReportReplaySectionPlan<'a>>, String> {
    match selector {
        CliReportReplaySelector::HeadToHead => {
            let (entrant_a, entrant_b) =
                resolve_report_replay_entrants(&report_source.standings, entrant_a, entrant_b)?;
            let selections =
                select_report_matches(report_source, &entrant_a, &entrant_b, sample_size)?;
            Ok(vec![ReportReplaySectionPlan {
                label: if sample_size == usize::MAX {
                    format!("{entrant_a} vs {entrant_b}")
                } else {
                    report_replay_selector_label(&entrant_a, &entrant_b, false)
                },
                entrant_a,
                entrant_b,
                selections,
            }])
        }
        CliReportReplaySelector::PresetTriangle => {
            if entrant_a.is_some() || entrant_b.is_some() {
                return Err(
                    "preset-triangle selector does not accept --entrant-a or --entrant-b"
                        .to_string(),
                );
            }

            [
                ("Easy vs Normal", PRESET_EASY_BOT, PRESET_NORMAL_BOT),
                ("Easy vs Hard", PRESET_EASY_BOT, PRESET_HARD_BOT),
                ("Normal vs Hard", PRESET_NORMAL_BOT, PRESET_HARD_BOT),
            ]
            .into_iter()
            .map(|(label, entrant_a, entrant_b)| {
                let selections =
                    select_report_matches(report_source, entrant_a, entrant_b, usize::MAX)?;
                Ok(ReportReplaySectionPlan {
                    label: label.to_string(),
                    entrant_a: entrant_a.to_string(),
                    entrant_b: entrant_b.to_string(),
                    selections,
                })
            })
            .collect()
        }
    }
}

pub(super) fn flatten_report_replay_sections(
    report_source: &ReportReplaySource,
    sections: &[ReportReplaySectionPlan<'_>],
) -> Vec<ReplayAnalysisInput> {
    sections
        .iter()
        .flat_map(|section| {
            section.selections.iter().map(|selection| {
                let replay = report_match_to_replay(report_source, selection.match_report)
                    .unwrap_or_else(|err| {
                        exit_with_error(format!(
                            "Failed to convert match {} to replay: {err}",
                            selection.match_report.match_index
                        ))
                    });
                ReplayAnalysisInput {
                    label: report_replay_input_label(selection.match_report),
                    replay,
                }
            })
        })
        .collect()
}

pub(super) fn published_analysis_sections_from_plans(
    sections: &[ReportReplaySectionPlan<'_>],
) -> Vec<PublishedAnalysisSectionInput> {
    sections
        .iter()
        .map(|section| PublishedAnalysisSectionInput {
            label: section.label.clone(),
            entrant_a: section.entrant_a.clone(),
            entrant_b: section.entrant_b.clone(),
            matches: section
                .selections
                .iter()
                .map(|selection| published_analysis_match_summary(selection.match_report))
                .collect(),
        })
        .collect()
}

pub(super) fn published_analysis_match_summary(
    match_report: &ReportReplayMatch,
) -> PublishedAnalysisMatchSummary {
    PublishedAnalysisMatchSummary {
        match_index: match_report.match_index,
        black: match_report.black.clone(),
        white: match_report.white.clone(),
        result: match_report.result.clone(),
        winner: match_report.winner.clone(),
        end_reason: match_report.end_reason.clone(),
        move_cells: match_report.move_cells.clone(),
        move_count: match_report.move_count,
    }
}

pub(super) fn report_replay_input_label(match_report: &ReportReplayMatch) -> String {
    format!(
        "match_{:04}__{}__vs__{}",
        match_report.match_index,
        match_report.black.replace('+', "_"),
        match_report.white.replace('+', "_")
    )
}

pub(super) fn highest_different_standing(
    standing_bots: &[String],
    entrant: &str,
) -> Option<String> {
    standing_bots
        .iter()
        .find(|bot| bot.as_str() != entrant)
        .cloned()
}

pub(super) fn report_replay_source_label(
    report: &Path,
    entrant_a: &str,
    entrant_b: &str,
    default_top_two: bool,
) -> String {
    format!(
        "{}:{}",
        report.display(),
        report_replay_selector_label(entrant_a, entrant_b, default_top_two)
    )
}

pub(super) fn report_replay_selector_label(
    entrant_a: &str,
    entrant_b: &str,
    default_top_two: bool,
) -> String {
    if default_top_two {
        "Top 2 entrants".to_string()
    } else {
        format!("{entrant_a} vs {entrant_b}")
    }
}
