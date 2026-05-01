use crate::arena::MatchEndReason;
use crate::elo::{expected_score, DEFAULT_INITIAL_RATING, DEFAULT_K_FACTOR};
use crate::tournament::TournamentResults;
use gomoku_core::{Color, GameResult, Move, RuleConfig};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub const TOURNAMENT_REPORT_SCHEMA_VERSION: u32 = 1;
pub const MOVE_CODEC: &str = "cell_index_v1";
const SHUFFLED_ELO_SAMPLES: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentRunReport {
    pub bots: Vec<String>,
    pub rules: RuleConfig,
    pub games_per_pair: u32,
    pub seed: u64,
    pub opening_plies: usize,
    pub threads: usize,
    pub search_time_ms: Option<u64>,
    pub search_cpu_time_ms: Option<u64>,
    pub max_moves: Option<usize>,
    pub max_game_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentReport {
    pub schema_version: u32,
    pub report_kind: String,
    pub board_size: usize,
    pub move_codec: String,
    pub shuffled_elo_samples: usize,
    pub run: TournamentRunReport,
    pub standings: Vec<StandingReport>,
    pub pairwise: Vec<PairwiseReport>,
    pub color_splits: Vec<ColorSplitReport>,
    pub end_reasons: Vec<CountReport>,
    pub matches: Vec<MatchReport>,
}

impl TournamentReport {
    pub fn from_results(
        run: TournamentRunReport,
        results: &TournamentResults,
    ) -> Result<Self, String> {
        let board_size = run.rules.board_size;
        let matches = results
            .matches
            .iter()
            .map(|record| MatchReport::from_record(record, board_size))
            .collect::<Result<Vec<_>, _>>()?;
        let shuffled_elo = shuffled_elo_stats(&run.bots, &matches, SHUFFLED_ELO_SAMPLES);

        Ok(Self {
            schema_version: TOURNAMENT_REPORT_SCHEMA_VERSION,
            report_kind: "tournament".to_string(),
            board_size,
            move_codec: MOVE_CODEC.to_string(),
            shuffled_elo_samples: SHUFFLED_ELO_SAMPLES,
            standings: standings(&run.bots, results, &matches, &shuffled_elo),
            pairwise: pairwise(&run.bots, &matches),
            color_splits: color_splits(&matches),
            end_reasons: end_reasons(results),
            matches,
            run,
        })
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(input: &str) -> Result<Self, String> {
        let report: Self = serde_json::from_str(input).map_err(|err| err.to_string())?;
        report.validate()?;
        Ok(report)
    }

    fn validate(&self) -> Result<(), String> {
        if self.schema_version != TOURNAMENT_REPORT_SCHEMA_VERSION {
            return Err(format!(
                "unsupported tournament report schema version: {}",
                self.schema_version
            ));
        }
        if self.report_kind != "tournament" {
            return Err(format!("unsupported report kind: {}", self.report_kind));
        }
        if self.move_codec != MOVE_CODEC {
            return Err(format!("unsupported move codec: {}", self.move_codec));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandingReport {
    pub bot: String,
    pub wins: u32,
    pub draws: u32,
    pub losses: u32,
    pub sequential_elo: f64,
    pub shuffled_elo_avg: f64,
    pub shuffled_elo_stddev: f64,
    pub match_count: u32,
    pub move_count: u32,
    pub search_move_count: u32,
    pub total_time_ms: u64,
    pub avg_search_time_ms: f64,
    pub total_nodes: u64,
    pub avg_nodes: f64,
    pub avg_depth: f64,
    pub max_depth: u32,
    pub budget_exhausted_count: u32,
    pub budget_exhausted_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairwiseReport {
    pub bot_a: String,
    pub bot_b: String,
    pub wins_a: u32,
    pub wins_b: u32,
    pub draws: u32,
    pub total: u32,
    pub score_a: f64,
    pub score_b: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorSplitReport {
    pub black: String,
    pub white: String,
    pub black_wins: u32,
    pub white_wins: u32,
    pub draws: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountReport {
    pub key: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReport {
    pub match_index: usize,
    pub black: String,
    pub white: String,
    pub result: String,
    pub winner: Option<String>,
    pub end_reason: String,
    pub duration_ms: Option<u64>,
    pub move_cells: Vec<usize>,
    pub move_count: usize,
    pub black_stats: SideStatsReport,
    pub white_stats: SideStatsReport,
}

impl MatchReport {
    fn from_record(
        record: &crate::tournament::TournamentMatchRecord,
        board_size: usize,
    ) -> Result<Self, String> {
        let mut black_stats = SideStatsAccumulator::default();
        let mut white_stats = SideStatsAccumulator::default();
        let mut move_cells = Vec::with_capacity(record.replay.moves.len());

        for (idx, replay_move) in record.replay.moves.iter().enumerate() {
            let mv = Move::from_notation(&replay_move.mv)?;
            move_cells.push(encode_move_cell(mv, board_size)?);
            let target = if idx % 2 == 0 {
                &mut black_stats
            } else {
                &mut white_stats
            };
            target.record_move(replay_move.time_ms, replay_move.trace.as_ref());
        }

        Ok(Self {
            match_index: record.match_idx,
            black: record.black_name.clone(),
            white: record.white_name.clone(),
            result: result_code(&record.result).to_string(),
            winner: winner_name(&record.result, &record.black_name, &record.white_name),
            end_reason: end_reason_code(record.end_reason).to_string(),
            duration_ms: record.replay.duration_ms,
            move_count: move_cells.len(),
            move_cells,
            black_stats: black_stats.finish(),
            white_stats: white_stats.finish(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SideStatsReport {
    pub move_count: u32,
    pub search_move_count: u32,
    pub total_time_ms: u64,
    pub avg_search_time_ms: f64,
    pub search_nodes: u64,
    pub prefilter_nodes: u64,
    pub total_nodes: u64,
    pub avg_nodes: f64,
    pub depth_sum: u64,
    pub avg_depth: f64,
    pub max_depth: u32,
    pub budget_exhausted_count: u32,
    pub budget_exhausted_rate: f64,
}

#[derive(Debug, Clone, Default)]
struct SideStatsAccumulator {
    move_count: u32,
    search_move_count: u32,
    total_time_ms: u64,
    search_nodes: u64,
    prefilter_nodes: u64,
    total_nodes: u64,
    depth_sum: u64,
    max_depth: u32,
    budget_exhausted_count: u32,
}

impl SideStatsAccumulator {
    fn record_move(&mut self, time_ms: u64, trace: Option<&Value>) {
        self.move_count += 1;
        self.total_time_ms += time_ms;

        let Some(trace) = trace else {
            return;
        };

        self.search_move_count += 1;
        self.search_nodes += trace_value_u64(trace, "nodes");
        self.prefilter_nodes += trace_value_u64(trace, "prefilter_nodes");
        self.total_nodes += trace_value_u64(trace, "total_nodes");
        if let Some(depth) = trace.get("depth").and_then(Value::as_u64) {
            self.depth_sum += depth;
            self.max_depth = self.max_depth.max(depth as u32);
        }
        if trace
            .get("budget_exhausted")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            self.budget_exhausted_count += 1;
        }
    }

    fn add_report(&mut self, stats: &SideStatsReport) {
        self.move_count += stats.move_count;
        self.search_move_count += stats.search_move_count;
        self.total_time_ms += stats.total_time_ms;
        self.search_nodes += stats.search_nodes;
        self.prefilter_nodes += stats.prefilter_nodes;
        self.total_nodes += stats.total_nodes;
        self.depth_sum += stats.depth_sum;
        self.max_depth = self.max_depth.max(stats.max_depth);
        self.budget_exhausted_count += stats.budget_exhausted_count;
    }

    fn finish(self) -> SideStatsReport {
        let avg_search_time_ms = avg(self.total_time_ms as f64, self.search_move_count);
        let avg_nodes = avg(self.total_nodes as f64, self.search_move_count);
        let avg_depth = avg(self.depth_sum as f64, self.search_move_count);
        let budget_exhausted_rate = avg(self.budget_exhausted_count as f64, self.search_move_count);

        SideStatsReport {
            move_count: self.move_count,
            search_move_count: self.search_move_count,
            total_time_ms: self.total_time_ms,
            avg_search_time_ms,
            search_nodes: self.search_nodes,
            prefilter_nodes: self.prefilter_nodes,
            total_nodes: self.total_nodes,
            avg_nodes,
            depth_sum: self.depth_sum,
            avg_depth,
            max_depth: self.max_depth,
            budget_exhausted_count: self.budget_exhausted_count,
            budget_exhausted_rate,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct EloAggregate {
    sum: f64,
    sum_sq: f64,
}

impl EloAggregate {
    fn add(&mut self, rating: f64) {
        self.sum += rating;
        self.sum_sq += rating * rating;
    }

    fn finish(&self, samples: usize) -> (f64, f64) {
        if samples == 0 {
            return (DEFAULT_INITIAL_RATING, 0.0);
        }
        let mean = self.sum / samples as f64;
        let variance = (self.sum_sq / samples as f64) - mean * mean;
        (mean, variance.max(0.0).sqrt())
    }
}

fn standings(
    bots: &[String],
    results: &TournamentResults,
    matches: &[MatchReport],
    shuffled_elo: &HashMap<String, (f64, f64)>,
) -> Vec<StandingReport> {
    let mut stats: HashMap<String, SideStatsAccumulator> = bots
        .iter()
        .map(|bot| (bot.clone(), SideStatsAccumulator::default()))
        .collect();

    for report_match in matches {
        stats
            .entry(report_match.black.clone())
            .or_default()
            .add_report(&report_match.black_stats);
        stats
            .entry(report_match.white.clone())
            .or_default()
            .add_report(&report_match.white_stats);
    }

    let mut standings = bots
        .iter()
        .map(|bot| {
            let side_stats = stats.remove(bot).unwrap_or_default().finish();
            let wins = *results.wins.get(bot).unwrap_or(&0);
            let draws = *results.draws.get(bot).unwrap_or(&0);
            let losses = *results.losses.get(bot).unwrap_or(&0);
            let (shuffled_elo_avg, shuffled_elo_stddev) = shuffled_elo
                .get(bot)
                .copied()
                .unwrap_or((DEFAULT_INITIAL_RATING, 0.0));

            StandingReport {
                bot: bot.clone(),
                wins,
                draws,
                losses,
                sequential_elo: results.elo_tracker.get_rating(bot),
                shuffled_elo_avg,
                shuffled_elo_stddev,
                match_count: wins + draws + losses,
                move_count: side_stats.move_count,
                search_move_count: side_stats.search_move_count,
                total_time_ms: side_stats.total_time_ms,
                avg_search_time_ms: side_stats.avg_search_time_ms,
                total_nodes: side_stats.total_nodes,
                avg_nodes: side_stats.avg_nodes,
                avg_depth: side_stats.avg_depth,
                max_depth: side_stats.max_depth,
                budget_exhausted_count: side_stats.budget_exhausted_count,
                budget_exhausted_rate: side_stats.budget_exhausted_rate,
            }
        })
        .collect::<Vec<_>>();

    standings.sort_by(|a, b| {
        b.shuffled_elo_avg
            .partial_cmp(&a.shuffled_elo_avg)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    standings
}

fn pairwise(bots: &[String], matches: &[MatchReport]) -> Vec<PairwiseReport> {
    let order = bot_order(bots);
    let mut map: HashMap<(String, String), PairwiseReport> = HashMap::new();

    for report_match in matches {
        let (bot_a, bot_b) = ordered_pair(&report_match.black, &report_match.white, &order);
        let entry = map
            .entry((bot_a.clone(), bot_b.clone()))
            .or_insert(PairwiseReport {
                bot_a,
                bot_b,
                wins_a: 0,
                wins_b: 0,
                draws: 0,
                total: 0,
                score_a: 0.0,
                score_b: 0.0,
            });
        entry.total += 1;

        match report_match.winner.as_deref() {
            Some(winner) if winner == entry.bot_a => {
                entry.wins_a += 1;
                entry.score_a += 1.0;
            }
            Some(winner) if winner == entry.bot_b => {
                entry.wins_b += 1;
                entry.score_b += 1.0;
            }
            None => {
                entry.draws += 1;
                entry.score_a += 0.5;
                entry.score_b += 0.5;
            }
            _ => {}
        }
    }

    let mut values = map.into_values().collect::<Vec<_>>();
    values.sort_by_key(|entry| {
        (
            order.get(&entry.bot_a).copied().unwrap_or(usize::MAX),
            order.get(&entry.bot_b).copied().unwrap_or(usize::MAX),
        )
    });
    values
}

fn color_splits(matches: &[MatchReport]) -> Vec<ColorSplitReport> {
    let mut map: HashMap<(String, String), ColorSplitReport> = HashMap::new();

    for report_match in matches {
        let entry = map
            .entry((report_match.black.clone(), report_match.white.clone()))
            .or_insert(ColorSplitReport {
                black: report_match.black.clone(),
                white: report_match.white.clone(),
                black_wins: 0,
                white_wins: 0,
                draws: 0,
                total: 0,
            });
        entry.total += 1;

        match report_match.winner.as_deref() {
            Some(winner) if winner == entry.black => entry.black_wins += 1,
            Some(winner) if winner == entry.white => entry.white_wins += 1,
            None => entry.draws += 1,
            _ => {}
        }
    }

    let mut values = map.into_values().collect::<Vec<_>>();
    values.sort_by(|a, b| a.black.cmp(&b.black).then(a.white.cmp(&b.white)));
    values
}

fn end_reasons(results: &TournamentResults) -> Vec<CountReport> {
    let mut values = results
        .end_reasons
        .iter()
        .map(|(reason, count)| CountReport {
            key: end_reason_code(*reason).to_string(),
            count: *count,
        })
        .collect::<Vec<_>>();
    values.sort_by(|a, b| a.key.cmp(&b.key));
    values
}

fn shuffled_elo_stats(
    bots: &[String],
    matches: &[MatchReport],
    samples: usize,
) -> HashMap<String, (f64, f64)> {
    let mut aggregate: HashMap<String, EloAggregate> = bots
        .iter()
        .map(|bot| (bot.clone(), EloAggregate::default()))
        .collect();

    for sample in 0..samples {
        let mut indices = (0..matches.len()).collect::<Vec<_>>();
        shuffle_indices(&mut indices, sample as u64);
        let ratings = elo_for_order(bots, matches, &indices);
        for (bot, rating) in ratings {
            aggregate.entry(bot).or_default().add(rating);
        }
    }

    aggregate
        .into_iter()
        .map(|(bot, aggregate)| (bot, aggregate.finish(samples)))
        .collect()
}

fn elo_for_order(
    bots: &[String],
    matches: &[MatchReport],
    indices: &[usize],
) -> HashMap<String, f64> {
    let mut ratings: HashMap<String, f64> = bots
        .iter()
        .map(|bot| (bot.clone(), DEFAULT_INITIAL_RATING))
        .collect();

    for &idx in indices {
        let report_match = &matches[idx];
        let black_rating = *ratings
            .get(&report_match.black)
            .unwrap_or(&DEFAULT_INITIAL_RATING);
        let white_rating = *ratings
            .get(&report_match.white)
            .unwrap_or(&DEFAULT_INITIAL_RATING);
        let expected_black = expected_score(black_rating, white_rating);
        let expected_white = expected_score(white_rating, black_rating);
        let (score_black, score_white) = match report_match.winner.as_deref() {
            Some(winner) if winner == report_match.black => (1.0, 0.0),
            Some(winner) if winner == report_match.white => (0.0, 1.0),
            None => (0.5, 0.5),
            _ => (0.5, 0.5),
        };

        ratings.insert(
            report_match.black.clone(),
            black_rating + DEFAULT_K_FACTOR * (score_black - expected_black),
        );
        ratings.insert(
            report_match.white.clone(),
            white_rating + DEFAULT_K_FACTOR * (score_white - expected_white),
        );
    }

    ratings
}

fn shuffle_indices(indices: &mut [usize], sample: u64) {
    let mut state = 0x9e37_79b9_7f4a_7c15_u64 ^ sample.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    for i in (1..indices.len()).rev() {
        state = xorshift64(state);
        let j = (state as usize) % (i + 1);
        indices.swap(i, j);
    }
}

fn xorshift64(mut value: u64) -> u64 {
    value ^= value << 13;
    value ^= value >> 7;
    value ^= value << 17;
    value
}

fn bot_order(bots: &[String]) -> HashMap<String, usize> {
    bots.iter()
        .enumerate()
        .map(|(idx, bot)| (bot.clone(), idx))
        .collect()
}

fn ordered_pair(first: &str, second: &str, order: &HashMap<String, usize>) -> (String, String) {
    let first_order = order.get(first).copied().unwrap_or(usize::MAX);
    let second_order = order.get(second).copied().unwrap_or(usize::MAX);
    if first_order < second_order || (first_order == second_order && first <= second) {
        (first.to_string(), second.to_string())
    } else {
        (second.to_string(), first.to_string())
    }
}

fn encode_move_cell(mv: Move, board_size: usize) -> Result<usize, String> {
    if mv.row >= board_size || mv.col >= board_size {
        return Err(format!(
            "move outside board: {} for board size {}",
            mv.to_notation(),
            board_size
        ));
    }
    Ok(mv.row * board_size + mv.col)
}

fn result_code(result: &GameResult) -> &'static str {
    match result {
        GameResult::Winner(Color::Black) => "black_won",
        GameResult::Winner(Color::White) => "white_won",
        GameResult::Draw => "draw",
        GameResult::Ongoing => "ongoing",
    }
}

fn winner_name(result: &GameResult, black: &str, white: &str) -> Option<String> {
    match result {
        GameResult::Winner(Color::Black) => Some(black.to_string()),
        GameResult::Winner(Color::White) => Some(white.to_string()),
        GameResult::Draw | GameResult::Ongoing => None,
    }
}

fn end_reason_code(reason: MatchEndReason) -> &'static str {
    match reason {
        MatchEndReason::Natural => "natural",
        MatchEndReason::MaxMoves => "max_moves",
        MatchEndReason::MaxGameTime => "max_game_time",
    }
}

fn trace_value_u64(trace: &Value, key: &str) -> u64 {
    trace.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn avg(total: f64, count: u32) -> f64 {
    if count == 0 {
        0.0
    } else {
        total / count as f64
    }
}

pub fn render_tournament_report_html(report: &TournamentReport) -> String {
    let mut html = String::new();
    html.push_str("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">");
    html.push_str("<title>Gomoku2D Tournament Report</title>");
    html.push_str(STYLE);
    html.push_str("</head><body><main>");
    html.push_str("<header><p class=\"eyebrow\">Gomoku2D Bot Lab</p><h1>Tournament Report</h1>");
    html.push_str(&format!(
        "<p class=\"lede\">{} bots, {} games per pair, {} rule, seed {}.</p></header>",
        report.run.bots.len(),
        report.run.games_per_pair,
        html_escape(&variant_label(&report.run.rules)),
        report.run.seed
    ));

    html.push_str("<section class=\"cards\">");
    metric_card(&mut html, "Matches", report.matches.len().to_string());
    metric_card(
        &mut html,
        "Opening Plies",
        report.run.opening_plies.to_string(),
    );
    metric_card(&mut html, "Threads", report.run.threads.to_string());
    metric_card(
        &mut html,
        "CPU Budget",
        report
            .run
            .search_cpu_time_ms
            .map(|ms| format!("{ms} ms"))
            .unwrap_or_else(|| "none".to_string()),
    );
    html.push_str("</section>");

    html.push_str("<section><h2>Standings</h2><table><thead><tr>");
    for head in [
        "Bot",
        "W",
        "D",
        "L",
        "Seq Elo",
        "Avg Elo",
        "Avg ms",
        "Avg nodes",
        "Avg depth",
        "Budget hit",
    ] {
        html.push_str(&format!("<th>{head}</th>"));
    }
    html.push_str("</tr></thead><tbody>");
    for row in &report.standings {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{:.1}</td><td>{:.1} +/- {:.1}</td><td>{:.1}</td><td>{:.0}</td><td>{:.2}</td><td>{:.0}%</td></tr>",
            html_escape(&row.bot),
            row.wins,
            row.draws,
            row.losses,
            row.sequential_elo,
            row.shuffled_elo_avg,
            row.shuffled_elo_stddev,
            row.avg_search_time_ms,
            row.avg_nodes,
            row.avg_depth,
            row.budget_exhausted_rate * 100.0,
        ));
    }
    html.push_str("</tbody></table></section>");

    html.push_str("<section><h2>Pairwise</h2><table><thead><tr>");
    for head in ["Pair", "A wins", "B wins", "Draws", "Score"] {
        html.push_str(&format!("<th>{head}</th>"));
    }
    html.push_str("</tr></thead><tbody>");
    for row in &report.pairwise {
        html.push_str(&format!(
            "<tr><td>{} / {}</td><td>{}</td><td>{}</td><td>{}</td><td>{:.1} - {:.1}</td></tr>",
            html_escape(&row.bot_a),
            html_escape(&row.bot_b),
            row.wins_a,
            row.wins_b,
            row.draws,
            row.score_a,
            row.score_b,
        ));
    }
    html.push_str("</tbody></table></section>");

    html.push_str("<section><h2>Color Splits</h2><table><thead><tr>");
    for head in ["Black", "White", "Black wins", "White wins", "Draws"] {
        html.push_str(&format!("<th>{head}</th>"));
    }
    html.push_str("</tr></thead><tbody>");
    for row in &report.color_splits {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            html_escape(&row.black),
            html_escape(&row.white),
            row.black_wins,
            row.white_wins,
            row.draws,
        ));
    }
    html.push_str("</tbody></table></section>");

    html.push_str("<section><h2>Matches</h2><div class=\"match-list\">");
    for row in &report.matches {
        html.push_str("<details class=\"match\"><summary>");
        html.push_str(&format!(
            "<span>#{:03}</span><strong>{} vs {}</strong><span>{}</span></summary>",
            row.match_index,
            html_escape(&row.black),
            html_escape(&row.white),
            html_escape(&result_label(row)),
        ));
        html.push_str("<div class=\"match-grid\">");
        html.push_str(&format!(
            "<p><b>Moves</b><br>{}</p>",
            html_escape(&move_notations(&row.move_cells, report.board_size).join(" "))
        ));
        html.push_str(&format!(
            "<p><b>Move cells</b><br>{}</p>",
            row.move_cells
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        ));
        html.push_str(&format!(
            "<p><b>Black stats</b><br>{:.1} ms, {:.0} nodes, depth {:.2}, budget {:.0}%</p>",
            row.black_stats.avg_search_time_ms,
            row.black_stats.avg_nodes,
            row.black_stats.avg_depth,
            row.black_stats.budget_exhausted_rate * 100.0,
        ));
        html.push_str(&format!(
            "<p><b>White stats</b><br>{:.1} ms, {:.0} nodes, depth {:.2}, budget {:.0}%</p>",
            row.white_stats.avg_search_time_ms,
            row.white_stats.avg_nodes,
            row.white_stats.avg_depth,
            row.white_stats.budget_exhausted_rate * 100.0,
        ));
        html.push_str("</div></details>");
    }
    html.push_str("</div></section></main></body></html>");
    html
}

fn metric_card(html: &mut String, label: &str, value: String) {
    html.push_str(&format!(
        "<article><span>{}</span><strong>{}</strong></article>",
        html_escape(label),
        html_escape(&value)
    ));
}

fn variant_label(rules: &RuleConfig) -> String {
    match rules.variant {
        gomoku_core::Variant::Freestyle => "freestyle".to_string(),
        gomoku_core::Variant::Renju => "renju".to_string(),
    }
}

fn result_label(report_match: &MatchReport) -> String {
    match report_match.winner.as_deref() {
        Some(winner) => format!("{winner} wins"),
        None => "draw".to_string(),
    }
}

fn move_notations(move_cells: &[usize], board_size: usize) -> Vec<String> {
    move_cells
        .iter()
        .map(|cell| {
            let row = cell / board_size;
            let col = cell % board_size;
            Move { row, col }.to_notation()
        })
        .collect()
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

const STYLE: &str = r#"
<style>
:root{color-scheme:dark;--bg:#111619;--panel:#1b2427;--text:#f2f0dc;--muted:#9ca99e;--line:#324044;--accent:#e2b84b;--green:#5ad17a}
*{box-sizing:border-box}body{margin:0;background:radial-gradient(circle at top,#253034,#111619 48rem);color:var(--text);font-family:ui-monospace,SFMono-Regular,Menlo,monospace;line-height:1.45}
main{width:min(1180px,calc(100% - 32px));margin:0 auto;padding:40px 0 56px}header{margin-bottom:28px}.eyebrow{margin:0 0 8px;color:var(--accent);letter-spacing:.14em;text-transform:uppercase;font-size:12px}h1{margin:0;font-size:clamp(32px,6vw,64px)}.lede{max-width:760px;color:var(--muted)}
.cards{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:12px;margin:24px 0}article,section,.match{background:color-mix(in srgb,var(--panel) 88%,transparent);border:1px solid var(--line);box-shadow:0 8px 0 rgba(0,0,0,.18)}
article{padding:14px 16px}article span{display:block;color:var(--muted);font-size:12px;text-transform:uppercase;letter-spacing:.1em}article strong{font-size:22px;color:var(--green)}
section{padding:18px;margin:16px 0;overflow:auto}h2{margin:0 0 14px;color:var(--accent);font-size:18px}table{width:100%;border-collapse:collapse;min-width:760px}th,td{padding:9px 10px;border-bottom:1px solid var(--line);text-align:right;white-space:nowrap}th:first-child,td:first-child{text-align:left}th{color:var(--muted);font-size:12px;text-transform:uppercase;letter-spacing:.08em}
.match-list{display:grid;gap:10px}.match{padding:0}.match summary{cursor:pointer;display:grid;grid-template-columns:72px 1fr auto;gap:12px;align-items:center;padding:12px 14px}.match summary span{color:var(--muted)}.match-grid{display:grid;grid-template-columns:1.4fr 1.4fr 1fr 1fr;gap:12px;padding:0 14px 14px;color:var(--muted)}.match-grid p{margin:0;word-break:break-word}.match-grid b{color:var(--text)}
@media (max-width:760px){main{width:min(100% - 20px,1180px);padding-top:24px}.cards{grid-template-columns:repeat(2,minmax(0,1fr))}.match summary{grid-template-columns:1fr}.match-grid{grid-template-columns:1fr}}
</style>
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_cells_match_saved_match_codec() {
        assert_eq!(encode_move_cell(Move { row: 0, col: 0 }, 15).unwrap(), 0);
        assert_eq!(encode_move_cell(Move { row: 7, col: 7 }, 15).unwrap(), 112);
        assert_eq!(
            encode_move_cell(Move { row: 14, col: 14 }, 15).unwrap(),
            224
        );
    }

    #[test]
    fn html_escape_handles_special_chars() {
        assert_eq!(html_escape("<bot & 'x'>"), "&lt;bot &amp; &#39;x&#39;&gt;");
    }

    #[test]
    fn from_json_rejects_unsupported_schema() {
        let input = r#"{
          "schema_version": 999,
          "report_kind": "tournament",
          "board_size": 15,
          "move_codec": "cell_index_v1",
          "shuffled_elo_samples": 256,
          "run": {
            "bots": [],
            "rules": {"board_size": 15, "win_length": 5, "variant": "renju"},
            "games_per_pair": 0,
            "seed": 0,
            "opening_plies": 0,
            "threads": 1,
            "search_time_ms": null,
            "search_cpu_time_ms": null,
            "max_moves": null,
            "max_game_ms": null
          },
          "standings": [],
          "pairwise": [],
          "color_splits": [],
          "end_reasons": [],
          "matches": []
        }"#;

        let err = TournamentReport::from_json(input).unwrap_err();
        assert!(err.contains("unsupported tournament report schema version"));
    }
}
