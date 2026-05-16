//! WASM binding layer for `gomoku-core` and `gomoku-bot`.
//!
//! This crate is a **bridge only** — it translates between Rust types and JS
//! values. It contains no game logic, no rule semantics, and no bot strategy.
//! All authoritative behaviour lives in `gomoku-core` and `gomoku-bot`;
//! `gomoku-wasm` just exposes it across the Wasm boundary.

use js_sys::Reflect;
use js_sys::{Array, Object};
use serde::Serialize;
use serde_json::Value;
use wasm_bindgen::prelude::*;

use gomoku_analysis::{
    AnalysisOptions, GameAnalysis, ReplayAnalysisCounters, ReplayAnalysisSession,
    ReplayAnalysisStepStatus, ReplayFrameAnnotations, ReplyPolicy,
};
use gomoku_bot::{
    frontier::RollingThreatFrontier,
    tactical::{defender_hint_reply_candidates_from_view, DefenderReplyRole, ThreatView},
    Bot, LeafCorridorConfig, MoveOrdering, RandomBot, SearchBot, SearchBotConfig, StaticEvaluation,
};
use gomoku_core::rules::Variant;
use gomoku_core::{Board, Color, GameResult, Move, Replay, RuleConfig};

#[cfg(test)]
mod replay_analysis_tests {
    use super::{analysis_options_from_json, WasmReplayAnalyzer};
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
    use super::WasmBoard;
    use gomoku_core::Move;

    #[test]
    fn hash_string_exports_exact_unsigned_hash() {
        let mut board = WasmBoard::create_with_variant("freestyle");

        board.inner.apply_move(Move { row: 7, col: 7 }).unwrap();

        let hash = board.hash_string();
        assert!(hash.parse::<u64>().is_ok());
        assert_eq!(hash, board.inner.hash().to_string());
    }
}

fn moves_to_js(moves: Vec<Move>) -> Vec<JsValue> {
    moves
        .into_iter()
        .map(|mv| {
            let obj = Object::new();
            let _ = Reflect::set(&obj, &"row".into(), &(mv.row as f64).into());
            let _ = Reflect::set(&obj, &"col".into(), &(mv.col as f64).into());
            obj.into()
        })
        .collect()
}

fn moves_to_js_array(moves: Vec<Move>) -> Array {
    let arr = Array::new();
    for mv in moves_to_js(moves) {
        arr.push(&mv);
    }
    arr
}

fn set_moves(obj: &Object, key: &str, moves: Vec<Move>) {
    let _ = Reflect::set(obj, &key.into(), &moves_to_js_array(moves).into());
}

fn push_unique_move(moves: &mut Vec<Move>, mv: Move) {
    if !moves.contains(&mv) {
        moves.push(mv);
    }
}

fn wasm_board_from_inner(inner: Board) -> WasmBoard {
    let threat_view = RollingThreatFrontier::from_board(&inner);
    WasmBoard { inner, threat_view }
}

const REPLAY_ANALYZER_STEP_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ReplayAnalysisStatus {
    Running,
    Resolved,
    Unclear,
    Unsupported,
    Error,
}

#[derive(Debug, Clone, Serialize)]
struct ReplayAnalysisStepResult {
    schema_version: u32,
    status: ReplayAnalysisStatus,
    done: bool,
    current_ply: Option<usize>,
    annotations: Vec<ReplayFrameAnnotations>,
    analysis: Option<GameAnalysis>,
    error: Option<String>,
    counters: ReplayAnalysisCounters,
}

fn replay_analysis_error(message: impl Into<String>) -> ReplayAnalysisStepResult {
    ReplayAnalysisStepResult {
        schema_version: REPLAY_ANALYZER_STEP_SCHEMA_VERSION,
        status: ReplayAnalysisStatus::Error,
        done: true,
        current_ply: None,
        annotations: Vec::new(),
        analysis: None,
        error: Some(message.into()),
        counters: ReplayAnalysisCounters::default(),
    }
}

fn analysis_options_from_json(options_json: &str) -> Result<AnalysisOptions, String> {
    let trimmed = options_json.trim();
    if trimmed.is_empty() {
        return Ok(AnalysisOptions::default());
    }

    let value = serde_json::from_str::<Value>(trimmed)
        .map_err(|err| format!("invalid options json: {err}"))?;
    let mut options = AnalysisOptions::default();

    let Some(object) = value.as_object() else {
        return Err("analysis options must be a JSON object".to_string());
    };

    if let Some(max_depth) = object.get("max_depth") {
        let value = max_depth
            .as_u64()
            .ok_or_else(|| "max_depth must be a non-negative integer".to_string())?;
        options.max_depth = value as usize;
    }

    if let Some(max_scan_plies) = object.get("max_scan_plies") {
        options.max_scan_plies = if max_scan_plies.is_null() {
            None
        } else {
            Some(max_scan_plies.as_u64().ok_or_else(|| {
                "max_scan_plies must be null or a non-negative integer".to_string()
            })? as usize)
        };
    }

    options.reply_policy = ReplyPolicy::CorridorReplies;
    Ok(options)
}

fn replay_analysis_status(status: ReplayAnalysisStepStatus) -> ReplayAnalysisStatus {
    match status {
        ReplayAnalysisStepStatus::Running => ReplayAnalysisStatus::Running,
        ReplayAnalysisStepStatus::Resolved => ReplayAnalysisStatus::Resolved,
        ReplayAnalysisStepStatus::Unclear => ReplayAnalysisStatus::Unclear,
        ReplayAnalysisStepStatus::Unsupported => ReplayAnalysisStatus::Unsupported,
    }
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
            ReplayAnalysisStepResult {
                schema_version: REPLAY_ANALYZER_STEP_SCHEMA_VERSION,
                status: replay_analysis_status(step.status),
                done: step.done,
                current_ply: step.current_ply,
                annotations: step.annotations,
                analysis: step.analysis,
                error: None,
                counters: step.counters,
            }
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
    pub fn create_with_variant(variant: &str) -> WasmBoard {
        let v = match variant {
            "renju" => Variant::Renju,
            _ => Variant::Freestyle,
        };
        wasm_board_from_inner(Board::new(RuleConfig {
            variant: v,
            ..RuleConfig::default()
        }))
    }

    #[wasm_bindgen(js_name = "applyMove")]
    pub fn apply_move(&mut self, row: usize, col: usize) -> JsValue {
        let mv = Move { row, col };
        let result = self.inner.apply_move(mv);
        let obj = Object::new();
        match result {
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
                let _ = Reflect::set(&obj, &"result".into(), &result_str.into());
                let _ = Reflect::set(&obj, &"error".into(), &JsValue::NULL);
            }
            Err(err) => {
                let _ = Reflect::set(&obj, &"result".into(), &JsValue::NULL);
                let _ = Reflect::set(&obj, &"error".into(), &err.to_string().into());
            }
        }
        obj.into()
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
    pub fn legal_moves(&self) -> Vec<JsValue> {
        moves_to_js(self.inner.legal_moves())
    }

    #[wasm_bindgen(js_name = "threatSnapshot")]
    pub fn threat_snapshot(&self) -> JsValue {
        let current = self.inner.current_player;
        let opponent = current.opponent();

        let winning_moves = self.threat_view.immediate_winning_moves_for(current);
        let mut blocked = winning_moves.clone();

        let immediate_threat_moves = self
            .threat_view
            .immediate_winning_moves_for(opponent)
            .into_iter()
            .filter(|mv| !blocked.contains(mv))
            .collect::<Vec<_>>();
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

        let obj = Object::new();
        set_moves(&obj, "winningMoves", winning_moves);
        set_moves(&obj, "immediateThreatMoves", immediate_threat_moves);
        set_moves(&obj, "imminentThreatMoves", imminent_threat_moves);
        set_moves(&obj, "counterThreatMoves", counter_threat_moves);
        set_moves(
            &obj,
            "forbiddenMoves",
            self.inner.forbidden_moves_for_current_player(),
        );
        obj.into()
    }

    #[wasm_bindgen(js_name = "winningCells")]
    pub fn winning_cells(&self) -> Vec<JsValue> {
        moves_to_js(self.inner.winning_line())
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
        Board::from_fen(fen)
            .map(|mut inner| {
                inner.config.variant = match variant {
                    "renju" => Variant::Renju,
                    _ => Variant::Freestyle,
                };
                wasm_board_from_inner(inner)
            })
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen(js_name = "cloneBoard")]
    pub fn clone_board(&self) -> WasmBoard {
        wasm_board_from_inner(self.inner.clone())
    }
}

enum BotInner {
    Random(Box<RandomBot>),
    Search(Box<SearchBot>),
}

#[wasm_bindgen]
pub struct WasmBot {
    inner: BotInner,
}

#[wasm_bindgen]
impl WasmBot {
    #[wasm_bindgen(js_name = "createRandom")]
    pub fn create_random() -> WasmBot {
        WasmBot {
            inner: BotInner::Random(Box::new(RandomBot::new())),
        }
    }

    #[wasm_bindgen(js_name = "createBaseline")]
    pub fn create_baseline(depth: i32) -> WasmBot {
        WasmBot {
            inner: BotInner::Search(Box::new(SearchBot::new(depth))),
        }
    }

    #[wasm_bindgen(js_name = "createSearch")]
    pub fn create_search(
        depth: i32,
        child_limit: i32,
        pattern_eval: bool,
        corridor_proof_depth: i32,
        corridor_proof_width: i32,
        corridor_proof_candidate_limit: i32,
    ) -> WasmBot {
        let mut config = SearchBotConfig::custom_depth(depth);
        if child_limit > 0 {
            config.move_ordering = MoveOrdering::Tactical;
            config.child_limit = Some(child_limit as usize);
        }
        if pattern_eval {
            config.static_eval = StaticEvaluation::PatternEval;
        }
        if corridor_proof_depth > 0
            && corridor_proof_width > 0
            && corridor_proof_candidate_limit > 0
        {
            config.leaf_corridor = LeafCorridorConfig {
                enabled: true,
                max_depth: corridor_proof_depth as usize,
                max_reply_width: corridor_proof_width as usize,
                proof_candidate_limit: corridor_proof_candidate_limit as usize,
            };
        }

        WasmBot {
            inner: BotInner::Search(Box::new(SearchBot::with_config(config))),
        }
    }

    #[wasm_bindgen(js_name = "chooseMove")]
    pub fn choose_move(&mut self, board: &WasmBoard) -> JsValue {
        let moves = board.inner.legal_moves();
        if moves.is_empty() {
            return JsValue::NULL;
        }
        let mv = match &mut self.inner {
            BotInner::Random(bot) => bot.choose_move(&board.inner),
            BotInner::Search(bot) => bot.choose_move(&board.inner),
        };
        let obj = Object::new();
        let _ = Reflect::set(&obj, &"row".into(), &(mv.row as f64).into());
        let _ = Reflect::set(&obj, &"col".into(), &(mv.col as f64).into());
        obj.into()
    }

    pub fn name(&self) -> String {
        match &self.inner {
            BotInner::Random(bot) => bot.name().into(),
            BotInner::Search(bot) => bot.name().into(),
        }
    }
}
