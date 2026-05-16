//! WASM binding layer for `gomoku-core` and `gomoku-bot`.
//!
//! This crate is a **bridge only** — it translates between Rust types and JS
//! values. It contains no game logic, no rule semantics, and no bot strategy.
//! All authoritative behaviour lives in `gomoku-core` and `gomoku-bot`;
//! `gomoku-wasm` just exposes it across the Wasm boundary.

use js_sys::Reflect;
use js_sys::{Array, Object};
use wasm_bindgen::prelude::*;

use gomoku_bot::{
    frontier::RollingThreatFrontier,
    tactical::{defender_hint_reply_candidates_from_view, DefenderReplyRole, ThreatView},
    Bot, LeafCorridorConfig, MoveOrdering, RandomBot, SearchBot, SearchBotConfig, StaticEvaluation,
};
use gomoku_core::rules::Variant;
use gomoku_core::{Board, Color, GameResult, Move, RuleConfig};

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
