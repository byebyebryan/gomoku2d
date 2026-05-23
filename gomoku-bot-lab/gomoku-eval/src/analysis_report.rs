use std::collections::HashSet;

use gomoku_core::{replay::ReplayResult, Move, Replay, RuleConfig};
use serde_json::Value;

use crate::report::{PublishedTournamentReport, TournamentReport};

#[derive(Debug, Clone)]
pub struct ReportReplaySelection<'a> {
    pub match_report: &'a ReportReplayMatch,
}

#[derive(Debug, Clone)]
pub struct ReportReplaySource {
    pub board_size: usize,
    pub move_codec: String,
    pub rules: RuleConfig,
    pub standings: Vec<String>,
    pub matches: Vec<ReportReplayMatch>,
}

#[derive(Debug, Clone)]
pub struct ReportReplayMatch {
    pub match_index: usize,
    pub black: String,
    pub white: String,
    pub result: String,
    pub winner: Option<String>,
    pub end_reason: String,
    pub duration_ms: Option<u64>,
    pub move_cells: Vec<usize>,
    pub move_count: usize,
}

impl ReportReplaySource {
    pub fn from_json(input: &str) -> Result<Self, String> {
        let value: Value = serde_json::from_str(input).map_err(|err| err.to_string())?;
        match value.get("report_kind").and_then(Value::as_str) {
            Some("tournament") => {
                let report = TournamentReport::from_json(input)?;
                Ok(Self::from_tournament_report(&report))
            }
            Some("published_tournament") => {
                let report = PublishedTournamentReport::from_json(input)?;
                Ok(Self::from_published_tournament_report(&report))
            }
            Some(other) => Err(format!("unsupported report kind: {other}")),
            None => Err("report is missing report_kind".to_string()),
        }
    }

    pub fn from_tournament_report(report: &TournamentReport) -> Self {
        Self {
            board_size: report.board_size,
            move_codec: report.move_codec.clone(),
            rules: report.run.rules.clone(),
            standings: report
                .standings
                .iter()
                .map(|standing| standing.bot.clone())
                .collect(),
            matches: report
                .matches
                .iter()
                .map(|match_report| ReportReplayMatch {
                    match_index: match_report.match_index,
                    black: match_report.black.clone(),
                    white: match_report.white.clone(),
                    result: match_report.result.clone(),
                    winner: match_report.winner.clone(),
                    end_reason: match_report.end_reason.clone(),
                    duration_ms: match_report.duration_ms,
                    move_cells: match_report.move_cells.clone(),
                    move_count: match_report.move_count,
                })
                .collect(),
        }
    }

    pub fn from_published_tournament_report(report: &PublishedTournamentReport) -> Self {
        Self {
            board_size: report.board_size,
            move_codec: report.move_codec.clone(),
            rules: report.run.rules.clone(),
            standings: report
                .standings
                .iter()
                .map(|standing| standing.bot.clone())
                .collect(),
            matches: report
                .matches
                .iter()
                .map(|match_report| ReportReplayMatch {
                    match_index: match_report.match_index,
                    black: match_report.black.clone(),
                    white: match_report.white.clone(),
                    result: match_report.result.clone(),
                    winner: match_report.winner.clone(),
                    end_reason: match_report.end_reason.clone(),
                    duration_ms: None,
                    move_cells: match_report.move_cells.clone(),
                    move_count: match_report.move_count,
                })
                .collect(),
        }
    }
}

pub fn select_report_matches<'a>(
    report: &'a ReportReplaySource,
    entrant_a: &str,
    entrant_b: &str,
    sample_size: usize,
) -> Result<Vec<ReportReplaySelection<'a>>, String> {
    if sample_size == 0 {
        return Err("sample size must be greater than zero".to_string());
    }

    let mut matches = report
        .matches
        .iter()
        .filter(|match_report| {
            (match_report.black == entrant_a && match_report.white == entrant_b)
                || (match_report.black == entrant_b && match_report.white == entrant_a)
        })
        .collect::<Vec<_>>();
    matches.sort_by_key(|match_report| match_report.match_index);

    if matches.is_empty() {
        return Err(format!(
            "report has no head-to-head matches for {entrant_a} vs {entrant_b}"
        ));
    }
    if sample_size >= matches.len() {
        return Ok(matches
            .into_iter()
            .map(|match_report| ReportReplaySelection { match_report })
            .collect());
    }

    let mut selected = Vec::new();
    let mut seen = HashSet::new();
    let mut push_match = |match_report: &'a ReportReplayMatch| {
        if selected.len() < sample_size && seen.insert(match_report.match_index) {
            selected.push(match_report);
        }
    };

    if let Some(match_report) = matches.iter().copied().find(|match_report| {
        match_report.result == "draw" || match_report.end_reason == "max_moves"
    }) {
        push_match(match_report);
    }

    for winner in [entrant_a, entrant_b] {
        for result in ["black_won", "white_won"] {
            if let Some(match_report) = matches.iter().copied().find(|match_report| {
                match_report.winner.as_deref() == Some(winner) && match_report.result == result
            }) {
                push_match(match_report);
            }
        }
    }

    if let Some(shortest) = matches.iter().copied().min_by_key(|m| m.move_count) {
        push_match(shortest);
    }
    if let Some(longest) = matches.iter().copied().max_by_key(|m| m.move_count) {
        push_match(longest);
    }
    if let Some(median) = median_by_move_count(&matches) {
        push_match(median);
    }

    for match_report in evenly_spaced_matches(&matches, sample_size) {
        push_match(match_report);
    }
    for match_report in matches {
        push_match(match_report);
    }

    Ok(selected
        .into_iter()
        .map(|match_report| ReportReplaySelection { match_report })
        .collect())
}

pub fn report_match_to_replay(
    report: &ReportReplaySource,
    match_report: &ReportReplayMatch,
) -> Result<Replay, String> {
    if report.move_codec != crate::report::MOVE_CODEC {
        return Err(format!("unsupported move codec: {}", report.move_codec));
    }

    let mut replay = Replay::new(
        report.rules.clone(),
        match_report.black.clone(),
        match_report.white.clone(),
    );
    for cell in match_report.move_cells.iter().copied() {
        replay.push_move(decode_move_cell(cell, report.board_size)?, 0, 0, None);
    }
    replay.result = match match_report.result.as_str() {
        "black_won" => ReplayResult::BlackWins,
        "white_won" => ReplayResult::WhiteWins,
        "draw" => ReplayResult::Draw,
        "ongoing" => ReplayResult::Ongoing,
        other => return Err(format!("unsupported match result: {other}")),
    };
    replay.duration_ms = match_report.duration_ms;
    Ok(replay)
}

fn decode_move_cell(cell: usize, board_size: usize) -> Result<Move, String> {
    if board_size == 0 {
        return Err("board size must be greater than zero".to_string());
    }
    let row = cell / board_size;
    let col = cell % board_size;
    if row >= board_size {
        return Err(format!(
            "move cell {cell} is outside {board_size}x{board_size} board"
        ));
    }
    Ok(Move { row, col })
}

fn median_by_move_count<'a>(matches: &[&'a ReportReplayMatch]) -> Option<&'a ReportReplayMatch> {
    let mut sorted = matches.to_vec();
    sorted.sort_by_key(|match_report| (match_report.move_count, match_report.match_index));
    sorted.get(sorted.len() / 2).copied()
}

fn evenly_spaced_matches<'a>(
    matches: &[&'a ReportReplayMatch],
    sample_size: usize,
) -> Vec<&'a ReportReplayMatch> {
    if sample_size <= 1 {
        return matches.first().copied().into_iter().collect();
    }
    let last = matches.len().saturating_sub(1);
    (0..sample_size)
        .filter_map(|idx| {
            let match_idx = idx * last / (sample_size - 1);
            matches.get(match_idx).copied()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use gomoku_core::{replay::ReplayResult, RuleConfig, Variant};

    use crate::analysis_report::{
        report_match_to_replay, select_report_matches, ReportReplaySource,
    };
    use crate::report::{
        CountReport, MatchReport, ReportProvenance, SideStatsReport, TournamentReport,
        TournamentRunReport, MOVE_CODEC, TOURNAMENT_REPORT_SCHEMA_VERSION,
    };

    fn sample_report(matches: Vec<MatchReport>) -> TournamentReport {
        TournamentReport {
            schema_version: TOURNAMENT_REPORT_SCHEMA_VERSION,
            report_kind: "tournament".to_string(),
            board_size: 15,
            move_codec: MOVE_CODEC.to_string(),
            shuffled_elo_samples: 0,
            provenance: ReportProvenance::default(),
            reference_anchors: None,
            run: TournamentRunReport {
                bots: vec!["bot-a".to_string(), "bot-b".to_string()],
                schedule: "round-robin".to_string(),
                rules: RuleConfig {
                    board_size: 15,
                    win_length: 5,
                    variant: Variant::Renju,
                },
                games_per_pair: 8,
                seed: 7,
                opening_plies: 4,
                opening_policy: "centered-suite".to_string(),
                threads: 1,
                search_time_ms: None,
                search_cpu_time_ms: Some(1000),
                search_budget_mode: "strict".to_string(),
                search_cpu_reserve_ms: None,
                search_cpu_max_move_ms: None,
                max_moves: Some(120),
                max_game_ms: None,
                total_wall_time_ms: None,
            },
            standings: Vec::new(),
            pairwise: Vec::new(),
            color_splits: Vec::new(),
            end_reasons: vec![CountReport {
                key: "win".to_string(),
                count: 1,
            }],
            matches,
        }
    }

    fn match_report(
        match_index: usize,
        black: &str,
        white: &str,
        result: &str,
        winner: Option<&str>,
        end_reason: &str,
        move_count: usize,
    ) -> MatchReport {
        MatchReport {
            match_index,
            black: black.to_string(),
            white: white.to_string(),
            result: result.to_string(),
            winner: winner.map(str::to_string),
            end_reason: end_reason.to_string(),
            duration_ms: Some(0),
            opening: None,
            move_cells: (0..move_count).collect(),
            move_count,
            black_stats: SideStatsReport::default(),
            white_stats: SideStatsReport::default(),
        }
    }

    #[test]
    fn stratified_report_sample_keeps_draws_both_winners_and_lengths() {
        let report = sample_report(vec![
            match_report(1, "bot-a", "bot-b", "black_won", Some("bot-a"), "win", 19),
            match_report(2, "bot-b", "bot-a", "white_won", Some("bot-a"), "win", 63),
            match_report(3, "bot-a", "bot-b", "white_won", Some("bot-b"), "win", 21),
            match_report(4, "bot-b", "bot-a", "black_won", Some("bot-b"), "win", 55),
            match_report(5, "bot-a", "bot-b", "draw", None, "max_moves", 120),
            match_report(6, "bot-a", "bot-b", "black_won", Some("bot-a"), "win", 15),
        ]);
        let replay_source = ReportReplaySource::from_tournament_report(&report);

        let selected = select_report_matches(&replay_source, "bot-a", "bot-b", 4)
            .expect("head-to-head matches should sample");
        let selected_indices = selected
            .iter()
            .map(|selection| selection.match_report.match_index)
            .collect::<Vec<_>>();

        assert!(
            selected_indices.contains(&5),
            "draw/max-move game is useful smoke coverage"
        );
        assert!(
            selected
                .iter()
                .any(|selection| selection.match_report.winner.as_deref() == Some("bot-a")),
            "sample should include a bot-a win"
        );
        assert!(
            selected
                .iter()
                .any(|selection| selection.match_report.winner.as_deref() == Some("bot-b")),
            "sample should include a bot-b win"
        );
        assert_eq!(selected.len(), 4);
    }

    #[test]
    fn stratified_report_sample_rejects_zero_size() {
        let report = sample_report(vec![match_report(
            1,
            "bot-a",
            "bot-b",
            "black_won",
            Some("bot-a"),
            "win",
            19,
        )]);
        let replay_source = ReportReplaySource::from_tournament_report(&report);

        let err = select_report_matches(&replay_source, "bot-a", "bot-b", 0)
            .expect_err("zero-sized analysis samples should be rejected");

        assert!(err.contains("sample size"));
    }

    #[test]
    fn report_match_to_replay_uses_cell_index_codec_and_result() {
        let report = sample_report(vec![MatchReport {
            move_cells: vec![112, 113],
            move_count: 2,
            ..match_report(42, "bot-a", "bot-b", "white_won", Some("bot-b"), "win", 2)
        }]);
        let replay_source = ReportReplaySource::from_tournament_report(&report);

        let replay = report_match_to_replay(&replay_source, &replay_source.matches[0])
            .expect("report match should convert to replay");

        assert_eq!(replay.black, "bot-a");
        assert_eq!(replay.white, "bot-b");
        assert_eq!(replay.rules.variant, Variant::Renju);
        assert_eq!(replay.moves[0].mv, "H8");
        assert_eq!(replay.moves[1].mv, "I8");
        assert_eq!(replay.result, ReplayResult::WhiteWins);
    }
}
