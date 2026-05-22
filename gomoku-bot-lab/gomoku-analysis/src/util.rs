use gomoku_core::Move;

pub(crate) fn push_unique_move(moves: &mut Vec<Move>, mv: Move) {
    if !moves.contains(&mv) {
        moves.push(mv);
    }
}

pub(crate) fn normalize_moves(moves: &mut Vec<Move>) {
    moves.sort_by_key(|mv| (mv.row, mv.col));
    moves.dedup();
}
