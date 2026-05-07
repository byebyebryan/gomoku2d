use gomoku_core::{Color, Move};

const CELL_SIZE: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportBoardMarker {
    pub mv: Move,
    pub notation: String,
    pub classes: Vec<String>,
    pub label: Option<String>,
    pub actual_stone: Option<Color>,
    pub hide_stone: bool,
}

impl ReportBoardMarker {
    pub fn new(mv: Move) -> Self {
        Self {
            mv,
            notation: mv.to_notation(),
            classes: Vec::new(),
            label: None,
            actual_stone: None,
            hide_stone: false,
        }
    }

    pub fn with_classes(mut self, classes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.classes = classes.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_actual_stone(mut self, color: Color) -> Self {
        self.actual_stone = Some(color);
        self
    }

    pub fn without_underlying_stone(mut self) -> Self {
        self.hide_stone = true;
        self
    }
}

pub fn report_board_css() -> &'static str {
    r#"
    .proof-board {
      --proof-cell-size: 20px;
      position: relative;
      display: grid;
      gap: 0;
      width: max-content;
      padding: 0;
      background: #d7ad63;
      border: 1px solid #7d592f;
      box-shadow: inset 0 0 0 1px rgba(255,255,255,0.18), 0 2px 0 rgba(0,0,0,0.26);
      box-sizing: border-box;
      overflow: hidden;
    }
    .proof-board::before {
      content: "";
      position: absolute;
      z-index: 0;
      left: calc(var(--proof-cell-size) / 2);
      top: calc(var(--proof-cell-size) / 2);
      width: var(--proof-grid-span);
      height: var(--proof-grid-span);
      background-image:
        linear-gradient(rgba(66, 43, 20, 0.48) 1px, transparent 1px),
        linear-gradient(90deg, rgba(66, 43, 20, 0.48) 1px, transparent 1px);
      background-size: var(--proof-cell-size) var(--proof-cell-size);
      pointer-events: none;
    }
    .proof-cell {
      position: relative;
      z-index: 1;
      width: var(--proof-cell-size);
      height: var(--proof-cell-size);
      background: transparent;
      box-sizing: border-box;
    }
    .proof-stone {
      position: absolute;
      top: 2px;
      right: 3px;
      bottom: 4px;
      left: 3px;
      z-index: 1;
      border-radius: 999px;
      box-shadow: 0 1px 1px rgba(0,0,0,0.5);
      transform: translateY(-1px);
    }
    .proof-stone--black {
      background: radial-gradient(circle at 35% 30%, #4d4f55 0%, #15171b 58%, #050608 100%);
      border: 1px solid #030405;
    }
    .proof-stone--white {
      background: radial-gradient(circle at 35% 30%, #fff7e6 0%, #e4d7b8 62%, #9d8f75 100%);
      border: 1px solid #8f8068;
    }
    .proof-actual-stone {
      position: absolute;
      top: 2px;
      right: 3px;
      bottom: 4px;
      left: 3px;
      z-index: 1;
      border-radius: 999px;
      background: transparent;
      box-shadow: 0 1px 1px rgba(0,0,0,0.24);
      transform: translateY(-1px);
    }
    .proof-actual-stone--black {
      border: 2px solid #07090c;
    }
    .proof-actual-stone--white {
      border: 2px solid #f6eed8;
      box-shadow: 0 1px 1px rgba(0,0,0,0.24), 0 0 0 1px #8f8068;
    }
    .proof-marker {
      position: absolute;
      inset: 1px;
      z-index: 2;
      display: grid;
      place-items: center;
      color: #101214;
      font-size: 10px;
      font-weight: 800;
      line-height: 1;
      pointer-events: none;
      transform: translateY(-2px);
    }
"#
}

pub fn render_report_board(rows: &[String], markers: &[ReportBoardMarker]) -> String {
    let size = rows.len();
    let grid_span = size.saturating_sub(1) * CELL_SIZE + 1;
    let cells = rows
        .iter()
        .enumerate()
        .flat_map(|(row, line)| {
            line.chars()
                .enumerate()
                .map(move |(col, stone)| report_cell_html(markers, row, col, stone))
        })
        .collect::<String>();

    format!(
        "<div class=\"proof-board\" style=\"--proof-grid-span: {grid_span}px; grid-template-columns: repeat({size}, var(--proof-cell-size)); grid-template-rows: repeat({size}, var(--proof-cell-size));\">{cells}</div>",
        grid_span = grid_span,
        size = size,
        cells = cells,
    )
}

fn report_cell_html(markers: &[ReportBoardMarker], row: usize, col: usize, stone: char) -> String {
    let marker = markers
        .iter()
        .find(|marker| marker.mv.row == row && marker.mv.col == col);
    let classes = cell_classes(stone, marker);
    let move_attr = marker
        .map(|marker| format!(" data-move=\"{}\"", html_escape(&marker.notation)))
        .unwrap_or_default();
    let stone_html = if marker.is_some_and(|marker| marker.hide_stone) {
        ""
    } else {
        match stone {
            'B' => "<span class=\"proof-stone proof-stone--black\"></span>",
            'W' => "<span class=\"proof-stone proof-stone--white\"></span>",
            _ => "",
        }
    };
    let actual_stone_html = marker.and_then(actual_stone_html).unwrap_or_default();
    let marker_html = marker
        .and_then(|marker| marker.label.as_ref())
        .map(|label| format!("<span class=\"proof-marker\">{}</span>", html_escape(label)))
        .unwrap_or_default();

    format!(
        "<div class=\"{classes}\"{move_attr}>{stone_html}{actual_stone_html}{marker_html}</div>",
        classes = classes,
        move_attr = move_attr,
        stone_html = stone_html,
        actual_stone_html = actual_stone_html,
        marker_html = marker_html,
    )
}

fn cell_classes(stone: char, marker: Option<&ReportBoardMarker>) -> String {
    let mut classes = vec!["proof-cell".to_string()];
    match stone {
        'B' => classes.push("proof-cell--stone-black".to_string()),
        'W' => classes.push("proof-cell--stone-white".to_string()),
        _ => {}
    }
    if let Some(marker) = marker {
        classes.extend(marker.classes.iter().cloned());
    }
    classes.join(" ")
}

fn actual_stone_html(marker: &ReportBoardMarker) -> Option<&'static str> {
    match marker.actual_stone? {
        Color::Black => {
            Some("<span class=\"proof-actual-stone proof-actual-stone--black\"></span>")
        }
        Color::White => {
            Some("<span class=\"proof-actual-stone proof-actual-stone--white\"></span>")
        }
    }
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_board_renders_grid_crossings_and_stones() {
        let rows = vec!["...".to_string(), ".B.".to_string(), "..W".to_string()];
        let marker = ReportBoardMarker::new(Move { row: 1, col: 1 })
            .with_classes(["marker--winning"])
            .with_label("W");

        let html = render_report_board(&rows, &[marker]);

        assert!(html.contains("class=\"proof-board\""));
        assert!(html.contains("--proof-grid-span: 41px"));
        assert!(html.contains("grid-template-columns: repeat(3, var(--proof-cell-size))"));
        assert!(html.contains("class=\"proof-stone proof-stone--black\""));
        assert!(html.contains("class=\"proof-stone proof-stone--white\""));
        assert!(html.contains("marker--winning"));
        assert!(html.contains("data-move=\"B2\""));
    }

    #[test]
    fn report_board_can_render_actual_move_ring_without_a_stone() {
        let rows = vec![".".to_string()];
        let marker =
            ReportBoardMarker::new(Move { row: 0, col: 0 }).with_actual_stone(Color::Black);

        let html = render_report_board(&rows, &[marker]);

        assert!(html.contains("proof-actual-stone--black"));
    }

    #[test]
    fn report_board_can_hide_underlying_stone_for_actual_move_ring() {
        let rows = vec!["B".to_string()];
        let marker = ReportBoardMarker::new(Move { row: 0, col: 0 })
            .with_actual_stone(Color::Black)
            .without_underlying_stone();

        let html = render_report_board(&rows, &[marker]);

        assert!(html.contains("proof-actual-stone--black"));
        assert!(!html.contains("proof-stone--black"));
    }
}
