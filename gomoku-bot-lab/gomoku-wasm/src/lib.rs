//! WASM binding layer for `gomoku-core` and `gomoku-bot`.
//!
//! This crate is a **bridge only** — it translates between Rust types and JS
//! values. It contains no game logic, no rule semantics, and no bot strategy.
//! All authoritative behaviour lives in `gomoku-core` and `gomoku-bot`;
//! `gomoku-wasm` just exposes it across the Wasm boundary.

use js_sys::Object;
use js_sys::Reflect;
use wasm_bindgen::prelude::*;

use gomoku_bot::{Bot, RandomBot, SearchBot};
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

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct WasmBoard {
    inner: Board,
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
        WasmBoard {
            inner: Board::new(RuleConfig::default()),
        }
    }

    #[wasm_bindgen(js_name = "createWithVariant")]
    pub fn create_with_variant(variant: &str) -> WasmBoard {
        let v = match variant {
            "renju" => Variant::Renju,
            _ => Variant::Freestyle,
        };
        WasmBoard {
            inner: Board::new(RuleConfig {
                variant: v,
                ..RuleConfig::default()
            }),
        }
    }

    #[wasm_bindgen(js_name = "applyMove")]
    pub fn apply_move(&mut self, row: usize, col: usize) -> JsValue {
        let mv = Move { row, col };
        let result = self.inner.apply_move(mv);
        let obj = Object::new();
        match result {
            Ok(game_result) => {
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

    #[wasm_bindgen(js_name = "immediateWinningMovesFor")]
    pub fn immediate_winning_moves_for(&self, player: u8) -> Vec<JsValue> {
        let color = match player {
            1 => Color::Black,
            2 => Color::White,
            _ => return vec![],
        };

        moves_to_js(self.inner.immediate_winning_moves_for(color))
    }

    #[wasm_bindgen(js_name = "forbiddenMovesForCurrentPlayer")]
    pub fn forbidden_moves_for_current_player(&self) -> Vec<JsValue> {
        moves_to_js(self.inner.forbidden_moves_for_current_player())
    }

    #[wasm_bindgen(js_name = "undoLastMove")]
    pub fn undo_last_move(&mut self) {
        if let Some(mv) = self.inner.history.last().copied() {
            self.inner.undo_move(mv);
        }
    }

    #[wasm_bindgen(js_name = "toFen")]
    pub fn to_fen(&self) -> String {
        self.inner.to_fen()
    }

    #[wasm_bindgen(js_name = "fromFen")]
    pub fn from_fen(fen: &str) -> Result<WasmBoard, JsValue> {
        Board::from_fen(fen)
            .map(|inner| WasmBoard { inner })
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
                WasmBoard { inner }
            })
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen(js_name = "cloneBoard")]
    pub fn clone_board(&self) -> WasmBoard {
        WasmBoard {
            inner: self.inner.clone(),
        }
    }
}

enum BotInner {
    Random(RandomBot),
    Search(SearchBot),
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
            inner: BotInner::Random(RandomBot::new()),
        }
    }

    #[wasm_bindgen(js_name = "createBaseline")]
    pub fn create_baseline(depth: i32) -> WasmBot {
        WasmBot {
            inner: BotInner::Search(SearchBot::new(depth)),
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
