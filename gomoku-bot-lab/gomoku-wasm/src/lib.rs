//! WASM binding layer for `gomoku-core` and `gomoku-bot`.
//!
//! This crate is a **bridge only** — it translates between Rust types and JS
//! values. It contains no game logic, no rule semantics, and no bot strategy.
//! All authoritative behaviour lives in `gomoku-core` and `gomoku-bot`;
//! `gomoku-wasm` just exposes it across the Wasm boundary.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use gomoku_analysis::{
    analysis_options_from_json, replay_analysis_error, ReplayAnalysisSession,
    ReplayAnalysisStepEnvelope,
};
use gomoku_bot::{
    frontier::RollingThreatFrontier,
    tactical::{
        compound_imminent_evidence_stones, defender_hint_reply_candidates_from_view,
        local_threat_evidence_stones, DefenderReplyRole, LocalThreatFact, LocalThreatKind,
        SearchThreatPolicy, ThreatView,
    },
    Bot, CorridorProofConfig, MoveOrdering, SearchBot, SearchBotConfig, StaticEvaluation,
};
use gomoku_core::rules::Variant;
use gomoku_core::{Board, Color, GameResult, Move, Replay, RuleConfig};

#[cfg(test)]
mod replay_analysis_tests {
    use super::WasmReplayAnalyzer;
    use gomoku_analysis::analysis_options_from_json;
    use gomoku_analysis::DEFAULT_MAX_SCAN_PLIES;
    use gomoku_core::{Board, Move, Replay, RuleConfig};
    use serde_json::Value;

    fn mv(notation: &str) -> Move {
        Move::from_notation(notation).expect("test move notation should parse")
    }

    fn replay_json(moves: &[&str]) -> String {
        let rules = RuleConfig::default();
        let mut board = Board::new(rules.clone());
        let mut replay = Replay::new(rules, "Black", "White");

        for notation in moves {
            let parsed = mv(notation);
            let result = board.apply_move(parsed).expect("test move should apply");
            replay.push_move(parsed, 0, board.hash(), None);
            replay.finish(&result, None);
        }

        replay.to_json().expect("test replay should serialize")
    }

    #[test]
    fn analysis_options_json_uses_defaults_for_empty_object() {
        let options = analysis_options_from_json("{}").expect("options should parse");

        assert_eq!(options.max_depth, 4);
        assert_eq!(options.max_scan_plies, Some(DEFAULT_MAX_SCAN_PLIES));
    }

    #[test]
    fn analysis_options_json_accepts_depth_and_unbounded_scan() {
        let options = analysis_options_from_json(r#"{"max_depth":2,"max_scan_plies":null}"#)
            .expect("options should parse");

        assert_eq!(options.max_depth, 2);
        assert_eq!(options.max_scan_plies, None);
    }

    fn step_value(analyzer: &mut WasmReplayAnalyzer, max_work_units: usize) -> Value {
        serde_json::from_str(&analyzer.step(max_work_units))
            .expect("analysis step result should be valid JSON")
    }

    #[test]
    fn replay_analysis_step_json_reports_running_then_resolved_finished_game() {
        let mut analyzer = WasmReplayAnalyzer::create_from_replay_json(
            &replay_json(&["H8", "A1", "I8", "A2", "J8", "A3", "K8", "B1", "L8"]),
            "{}",
        );

        let first = step_value(&mut analyzer, 1);
        assert_eq!(first["status"], "running");
        assert_eq!(first["done"], false);
        assert!(first["analysis"].is_null());
        assert!(first["error"].is_null());
        assert_eq!(first["annotations"].as_array().unwrap().len(), 1);
        assert_eq!(first["counters"]["prefixes_analyzed"], 1);

        let mut final_step = first;
        for _ in 0..16 {
            if final_step["done"] == true {
                break;
            }
            final_step = step_value(&mut analyzer, 1);
        }

        assert_eq!(final_step["status"], "resolved");
        assert_eq!(final_step["done"], true);
        assert!(!final_step["analysis"].is_null());
        assert!(final_step["error"].is_null());
        assert!(final_step["current_ply"].is_null());
    }

    #[test]
    fn replay_analysis_step_json_reports_unsupported_ongoing_game() {
        let mut analyzer =
            WasmReplayAnalyzer::create_from_replay_json(&replay_json(&["H8", "A1", "I8"]), "{}");
        let result = step_value(&mut analyzer, 1);

        assert_eq!(result["status"], "unsupported");
        assert_eq!(result["done"], true);
        assert!(!result["analysis"].is_null());
        assert!(result["error"].is_null());
    }

    #[test]
    fn replay_analysis_step_json_reports_invalid_replay_error() {
        let mut analyzer = WasmReplayAnalyzer::create_from_replay_json("{not json", "{}");
        let result = step_value(&mut analyzer, 1);

        assert_eq!(result["status"], "error");
        assert_eq!(result["done"], true);
        assert!(result["analysis"].is_null());
        assert!(result["error"]
            .as_str()
            .expect("error should be a string")
            .contains("invalid replay json"));
    }
}

#[cfg(test)]
mod wasm_board_tests {
    use super::{parse_bot_spec, parse_variant_value, WasmBoard};
    use gomoku_bot::{MoveOrdering, StaticEvaluation};
    use gomoku_core::Move;
    use serde_json::Value;

    #[test]
    fn hash_string_exports_exact_unsigned_hash() {
        let mut board = WasmBoard::create_with_variant("freestyle").expect("variant should parse");

        board.inner.apply_move(Move { row: 7, col: 7 }).unwrap();

        let hash = board.hash_string();
        assert!(hash.parse::<u64>().is_ok());
        assert_eq!(hash, board.inner.hash().to_string());
    }

    #[test]
    fn parse_variant_rejects_unknown_values() {
        assert!(parse_variant_value("freestyle").is_ok());
        assert!(parse_variant_value("renju").is_ok());
        assert!(parse_variant_value("gomoku").is_err());
    }

    #[test]
    fn apply_move_exports_json_result() {
        let mut board = WasmBoard::create_with_variant("freestyle").expect("variant should parse");
        let result: Value = serde_json::from_str(&board.apply_move(7, 7))
            .expect("apply move result should be JSON");

        assert_eq!(result["result"], "ongoing");
        assert!(result["error"].is_null());
    }

    #[test]
    fn threat_snapshot_exports_json_payload() {
        let board = WasmBoard::create_with_variant("freestyle").expect("variant should parse");
        let snapshot: Value =
            serde_json::from_str(&board.threat_snapshot()).expect("threat snapshot should be JSON");

        assert!(snapshot["winningMoves"].is_array());
        assert!(snapshot["forbiddenMoves"].is_array());
    }

    #[test]
    fn bot_spec_json_configures_search_bot() {
        let config = parse_bot_spec(
            r#"{"kind":"search","depth":5,"childLimit":16,"maxTtEntries":500000,"patternEval":true,"corridorProof":{"candidateLimit":16,"depth":8,"width":4}}"#,
        )
        .expect("bot spec should parse");

        assert_eq!(config.max_depth, 5);
        assert_eq!(config.child_limit, Some(16));
        assert_eq!(config.max_tt_entries, Some(500_000));
        assert_eq!(config.move_ordering, MoveOrdering::Tactical);
        assert_eq!(config.static_eval, StaticEvaluation::PatternEval);
        assert!(config.corridor_proof.enabled);
        assert_eq!(config.corridor_proof.max_depth, 8);
        assert_eq!(config.corridor_proof.max_reply_width, 4);
        assert_eq!(config.corridor_proof.proof_candidate_limit, 16);
    }

    #[test]
    fn bot_spec_json_rejects_non_search_bot() {
        let err = parse_bot_spec(r#"{"kind":"human"}"#)
            .expect_err("human bot spec should not construct a wasm bot");

        assert!(err.contains("search bot"));
    }
}

fn push_unique_move(moves: &mut Vec<Move>, mv: Move) {
    if !moves.contains(&mv) {
        moves.push(mv);
    }
}

fn normalize_moves_for_snapshot(moves: &mut Vec<Move>) {
    moves.sort_by_key(|mv| (mv.row, mv.col));
    moves.dedup();
}

fn push_threat_fact_evidence(moves: &mut Vec<Move>, board: &Board, fact: &LocalThreatFact) {
    for mv in local_threat_evidence_stones(board, fact) {
        push_unique_move(moves, mv);
    }
}

fn candidate_evidence_stones(
    board: &Board,
    player: Color,
    mv: Move,
    keep: impl Fn(LocalThreatKind) -> bool,
) -> Vec<Move> {
    let annotation = SearchThreatPolicy.annotation_for_player(board, player, mv);
    let mut evidence = Vec::new();
    for fact in annotation
        .local_threats
        .iter()
        .filter(|fact| keep(fact.kind))
    {
        push_threat_fact_evidence(&mut evidence, board, fact);
    }
    normalize_moves_for_snapshot(&mut evidence);
    evidence
}

fn push_candidate_evidence(
    moves: &mut Vec<Move>,
    board: &Board,
    player: Color,
    mv: Move,
    keep: impl Fn(LocalThreatKind) -> bool,
) {
    for evidence in candidate_evidence_stones(board, player, mv, keep) {
        push_unique_move(moves, evidence);
    }
}

fn wasm_board_from_inner(inner: Board) -> WasmBoard {
    let threat_view = RollingThreatFrontier::from_board(&inner);
    WasmBoard { inner, threat_view }
}

fn parse_variant_value(variant: &str) -> Result<Variant, String> {
    match variant {
        "freestyle" => Ok(Variant::Freestyle),
        "renju" => Ok(Variant::Renju),
        _ => Err(format!("unknown game variant: {variant}")),
    }
}

fn to_bridge_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("wasm bridge payload should serialize")
}

#[derive(Debug, Clone, Copy, Serialize)]
struct BridgeMove {
    row: usize,
    col: usize,
}

impl From<Move> for BridgeMove {
    fn from(mv: Move) -> Self {
        Self {
            row: mv.row,
            col: mv.col,
        }
    }
}

fn bridge_moves(moves: Vec<Move>) -> Vec<BridgeMove> {
    moves.into_iter().map(BridgeMove::from).collect()
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApplyMoveResult {
    result: Option<&'static str>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ThreatSnapshot {
    winning_moves: Vec<BridgeMove>,
    winning_evidence_cells: Vec<BridgeMove>,
    immediate_threat_moves: Vec<BridgeMove>,
    immediate_threat_evidence_cells: Vec<BridgeMove>,
    imminent_threat_moves: Vec<BridgeMove>,
    imminent_threat_evidence_cells: Vec<BridgeMove>,
    counter_threat_moves: Vec<BridgeMove>,
    counter_threat_evidence_cells: Vec<BridgeMove>,
    forbidden_moves: Vec<BridgeMove>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WasmCorridorProofSpec {
    candidate_limit: i32,
    depth: i32,
    width: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
enum WasmBotSpec {
    Human,
    Search {
        child_limit: Option<i32>,
        corridor_proof: Option<WasmCorridorProofSpec>,
        depth: i32,
        max_tt_entries: Option<i32>,
        pattern_eval: bool,
    },
}

fn parse_bot_spec(spec_json: &str) -> Result<SearchBotConfig, String> {
    let spec = serde_json::from_str::<WasmBotSpec>(spec_json)
        .map_err(|err| format!("invalid bot spec json: {err}"))?;

    let WasmBotSpec::Search {
        child_limit,
        corridor_proof,
        depth,
        max_tt_entries,
        pattern_eval,
    } = spec
    else {
        return Err("wasm bot spec must be a search bot".to_string());
    };

    if depth < 0 {
        return Err("bot depth must be non-negative".to_string());
    }
    if let Some(max_tt_entries) = max_tt_entries {
        if max_tt_entries <= 0 {
            return Err("bot maxTtEntries must be null or a positive integer".to_string());
        }
    }

    let mut config = SearchBotConfig::custom_depth(depth);
    config.max_tt_entries = max_tt_entries.map(|value| value as usize);
    if let Some(child_limit) = child_limit {
        if child_limit <= 0 {
            return Err("bot childLimit must be null or a positive integer".to_string());
        }
        config.move_ordering = MoveOrdering::Tactical;
        config.child_limit = Some(child_limit as usize);
    }
    if pattern_eval {
        config.static_eval = StaticEvaluation::PatternEval;
    }
    if let Some(corridor_proof) = corridor_proof {
        if corridor_proof.depth <= 0
            || corridor_proof.width <= 0
            || corridor_proof.candidate_limit <= 0
        {
            return Err(
                "corridor proof depth, width, and candidateLimit must be positive integers"
                    .to_string(),
            );
        }
        config.corridor_proof = CorridorProofConfig {
            enabled: true,
            max_depth: corridor_proof.depth as usize,
            max_reply_width: corridor_proof.width as usize,
            proof_candidate_limit: corridor_proof.candidate_limit as usize,
        };
    }

    Ok(config)
}

#[wasm_bindgen]
pub struct WasmReplayAnalyzer {
    completed_json: Option<String>,
    init_error: Option<String>,
    session: Option<ReplayAnalysisSession>,
}

#[wasm_bindgen]
impl WasmReplayAnalyzer {
    #[wasm_bindgen(js_name = "createFromReplayJson")]
    pub fn create_from_replay_json(replay_json: &str, options_json: &str) -> WasmReplayAnalyzer {
        let session = Replay::from_json(replay_json)
            .map_err(|err| format!("invalid replay json: {err}"))
            .and_then(|replay| {
                analysis_options_from_json(options_json)
                    .map(|options| (replay, options))
                    .map_err(|err| err.to_string())
            })
            .and_then(|(replay, options)| {
                ReplayAnalysisSession::new(replay, options).map_err(|err| err.to_string())
            });

        let (session, init_error) = match session {
            Ok(session) => (Some(session), None),
            Err(err) => (None, Some(err)),
        };

        WasmReplayAnalyzer {
            completed_json: None,
            init_error,
            session,
        }
    }

    pub fn step(&mut self, max_work_units: usize) -> String {
        if let Some(completed_json) = &self.completed_json {
            return completed_json.clone();
        }

        let result = if let Some(init_error) = self.init_error.take() {
            replay_analysis_error(init_error)
        } else if let Some(session) = self.session.as_mut() {
            let step = session.step(max_work_units);
            ReplayAnalysisStepEnvelope::from_step(step)
        } else {
            replay_analysis_error("analysis session is not available")
        };

        let json = serde_json::to_string(&result).unwrap_or_else(|err| {
            serde_json::to_string(&replay_analysis_error(format!(
                "failed to serialize analysis result: {err}"
            )))
            .expect("error analysis result should serialize")
        });
        if result.done {
            self.completed_json = Some(json.clone());
            self.session = None;
        }
        json
    }

    pub fn dispose(&mut self) {
        self.completed_json = None;
        self.init_error = None;
        self.session = None;
    }
}

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct WasmBoard {
    inner: Board,
    threat_view: RollingThreatFrontier,
}

impl Default for WasmBoard {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WasmBoard {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmBoard {
        wasm_board_from_inner(Board::new(RuleConfig::default()))
    }

    #[wasm_bindgen(js_name = "createWithVariant")]
    pub fn create_with_variant(variant: &str) -> Result<WasmBoard, JsValue> {
        let parsed = parse_variant_value(variant).map_err(|err| JsValue::from_str(&err))?;
        Ok(wasm_board_from_inner(Board::new(RuleConfig {
            variant: parsed,
            ..RuleConfig::default()
        })))
    }

    #[wasm_bindgen(js_name = "applyMove")]
    pub fn apply_move(&mut self, row: usize, col: usize) -> String {
        let mv = Move { row, col };
        let result = self.inner.apply_move(mv);
        let result = match result {
            Ok(game_result) => {
                if self.threat_view.apply_move(mv).is_err() {
                    self.threat_view = RollingThreatFrontier::from_board(&self.inner);
                }
                let result_str = match game_result {
                    GameResult::Ongoing => "ongoing",
                    GameResult::Winner(Color::Black) => "black",
                    GameResult::Winner(Color::White) => "white",
                    GameResult::Draw => "draw",
                };
                ApplyMoveResult {
                    result: Some(result_str),
                    error: None,
                }
            }
            Err(err) => ApplyMoveResult {
                result: None,
                error: Some(err.to_string()),
            },
        };
        to_bridge_json(&result)
    }

    #[wasm_bindgen(js_name = "isLegal")]
    pub fn is_legal(&self, row: usize, col: usize) -> bool {
        self.inner.is_legal(Move { row, col })
    }

    pub fn cell(&self, row: usize, col: usize) -> u8 {
        match self.inner.cell(row, col) {
            None => 0,
            Some(Color::Black) => 1,
            Some(Color::White) => 2,
        }
    }

    #[wasm_bindgen(js_name = "currentPlayer")]
    pub fn current_player(&self) -> u8 {
        match self.inner.current_player {
            Color::Black => 1,
            Color::White => 2,
        }
    }

    pub fn result(&self) -> String {
        match self.inner.result {
            GameResult::Ongoing => "ongoing".into(),
            GameResult::Winner(Color::Black) => "black".into(),
            GameResult::Winner(Color::White) => "white".into(),
            GameResult::Draw => "draw".into(),
        }
    }

    #[wasm_bindgen(js_name = "moveCount")]
    pub fn move_count(&self) -> usize {
        self.inner.history.len()
    }

    #[wasm_bindgen(js_name = "hashString")]
    pub fn hash_string(&self) -> String {
        self.inner.hash().to_string()
    }

    #[wasm_bindgen(js_name = "legalMoves")]
    pub fn legal_moves(&self) -> String {
        to_bridge_json(&bridge_moves(self.inner.legal_moves()))
    }

    #[wasm_bindgen(js_name = "threatSnapshot")]
    pub fn threat_snapshot(&self) -> String {
        let current = self.inner.current_player;
        let opponent = current.opponent();

        let winning_moves = self.threat_view.immediate_winning_moves_for(current);
        let mut winning_evidence_cells = Vec::new();
        for mv in winning_moves.iter().copied() {
            push_candidate_evidence(
                &mut winning_evidence_cells,
                &self.inner,
                current,
                mv,
                |kind| kind == LocalThreatKind::Five,
            );
        }

        let mut blocked = winning_moves.clone();

        let immediate_threat_moves = self
            .threat_view
            .immediate_winning_moves_for(opponent)
            .into_iter()
            .filter(|mv| !blocked.contains(mv))
            .collect::<Vec<_>>();
        let mut immediate_threat_evidence_cells = Vec::new();
        for mv in immediate_threat_moves.iter().copied() {
            push_candidate_evidence(
                &mut immediate_threat_evidence_cells,
                &self.inner,
                opponent,
                mv,
                |kind| kind == LocalThreatKind::Five,
            );
        }
        blocked.extend(immediate_threat_moves.iter().copied());

        let reply_candidates =
            defender_hint_reply_candidates_from_view(&self.inner, &self.threat_view, opponent);
        let mut imminent_threat_moves = Vec::new();
        for candidate in &reply_candidates {
            if blocked.contains(&candidate.mv) {
                continue;
            }
            if candidate
                .roles
                .contains(&DefenderReplyRole::ImminentDefense)
            {
                push_unique_move(&mut imminent_threat_moves, candidate.mv);
                blocked.push(candidate.mv);
            }
        }
        let mut imminent_threat_evidence_cells = Vec::new();
        if !imminent_threat_moves.is_empty() {
            if let Some(obligation) = self.threat_view.threat_obligation(opponent) {
                for fact in &obligation.local_facts {
                    push_threat_fact_evidence(
                        &mut imminent_threat_evidence_cells,
                        &self.inner,
                        fact,
                    );
                }
                if !obligation.compound_entries.is_empty() {
                    for mv in compound_imminent_evidence_stones(
                        &self.inner,
                        opponent,
                        &obligation.compound_entries,
                    ) {
                        push_unique_move(&mut imminent_threat_evidence_cells, mv);
                    }
                }
            }
        }

        let mut counter_threat_moves = Vec::new();
        for candidate in &reply_candidates {
            if blocked.contains(&candidate.mv) {
                continue;
            }
            if candidate
                .roles
                .contains(&DefenderReplyRole::OffensiveCounter)
            {
                push_unique_move(&mut counter_threat_moves, candidate.mv);
                blocked.push(candidate.mv);
            }
        }
        let mut counter_threat_evidence_cells = Vec::new();
        for mv in counter_threat_moves.iter().copied() {
            push_candidate_evidence(
                &mut counter_threat_evidence_cells,
                &self.inner,
                current,
                mv,
                |kind| {
                    matches!(
                        kind,
                        LocalThreatKind::Five
                            | LocalThreatKind::OpenFour
                            | LocalThreatKind::ClosedFour
                            | LocalThreatKind::BrokenFour
                    )
                },
            );
        }

        normalize_moves_for_snapshot(&mut winning_evidence_cells);
        normalize_moves_for_snapshot(&mut immediate_threat_evidence_cells);
        normalize_moves_for_snapshot(&mut imminent_threat_evidence_cells);
        normalize_moves_for_snapshot(&mut counter_threat_evidence_cells);

        to_bridge_json(&ThreatSnapshot {
            winning_moves: bridge_moves(winning_moves),
            winning_evidence_cells: bridge_moves(winning_evidence_cells),
            immediate_threat_moves: bridge_moves(immediate_threat_moves),
            immediate_threat_evidence_cells: bridge_moves(immediate_threat_evidence_cells),
            imminent_threat_moves: bridge_moves(imminent_threat_moves),
            imminent_threat_evidence_cells: bridge_moves(imminent_threat_evidence_cells),
            counter_threat_moves: bridge_moves(counter_threat_moves),
            counter_threat_evidence_cells: bridge_moves(counter_threat_evidence_cells),
            forbidden_moves: bridge_moves(self.inner.forbidden_moves_for_current_player()),
        })
    }

    #[wasm_bindgen(js_name = "winningCells")]
    pub fn winning_cells(&self) -> String {
        to_bridge_json(&bridge_moves(self.inner.winning_line()))
    }

    #[wasm_bindgen(js_name = "undoLastMove")]
    pub fn undo_last_move(&mut self) {
        if let Some(mv) = self.inner.history.last().copied() {
            self.inner.undo_move(mv);
            self.threat_view = RollingThreatFrontier::from_board(&self.inner);
        }
    }

    #[wasm_bindgen(js_name = "toFen")]
    pub fn to_fen(&self) -> String {
        self.inner.to_fen()
    }

    #[wasm_bindgen(js_name = "fromFen")]
    pub fn from_fen(fen: &str) -> Result<WasmBoard, JsValue> {
        Board::from_fen(fen)
            .map(wasm_board_from_inner)
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen(js_name = "fromFenWithVariant")]
    pub fn from_fen_with_variant(fen: &str, variant: &str) -> Result<WasmBoard, JsValue> {
        let parsed = parse_variant_value(variant).map_err(|err| JsValue::from_str(&err))?;
        Board::from_fen(fen)
            .map(|mut inner| {
                inner.config.variant = parsed;
                wasm_board_from_inner(inner)
            })
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen(js_name = "cloneBoard")]
    pub fn clone_board(&self) -> WasmBoard {
        wasm_board_from_inner(self.inner.clone())
    }
}

#[wasm_bindgen]
pub struct WasmBot {
    inner: SearchBot,
}

#[wasm_bindgen]
impl WasmBot {
    #[wasm_bindgen(js_name = "createFromSpec")]
    pub fn create_from_spec(spec_json: &str) -> Result<WasmBot, JsValue> {
        let config = parse_bot_spec(spec_json).map_err(|err| JsValue::from_str(&err))?;

        Ok(WasmBot {
            inner: SearchBot::with_config(config),
        })
    }

    #[wasm_bindgen(js_name = "chooseMove")]
    pub fn choose_move(&mut self, board: &WasmBoard) -> String {
        let moves = board.inner.legal_moves();
        if moves.is_empty() {
            return "null".to_string();
        }
        let mv = self.inner.choose_move(&board.inner);
        to_bridge_json(&Some(BridgeMove::from(mv)))
    }

    pub fn name(&self) -> String {
        self.inner.name().into()
    }
}
