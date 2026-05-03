use std::time::Duration;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use std::hint::black_box;

use gomoku_core::{Board, Color, Move};

#[path = "../../benchmarks/scenarios.rs"]
mod scenarios;

fn criterion_config() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3))
        .sample_size(20)
        .without_plots()
}

fn bench_immediate_winning_moves(c: &mut Criterion) {
    let mut group = c.benchmark_group("core/immediate_winning_moves/current_player");

    for scenario in scenarios::SCENARIOS {
        let board = scenario.board();
        let color = board.current_player;
        group.bench_with_input(
            BenchmarkId::from_parameter(scenario.id),
            scenario,
            |b, _| {
                b.iter(|| black_box(board.immediate_winning_moves_for(color)));
            },
        );
    }

    group.finish();
}

fn bench_has_multiple_immediate_winning_moves(c: &mut Criterion) {
    let mut group = c.benchmark_group("core/has_multiple_immediate_winning_moves/current_player");

    for scenario in scenarios::SCENARIOS {
        let board = scenario.board();
        let color = board.current_player;
        group.bench_with_input(
            BenchmarkId::from_parameter(scenario.id),
            scenario,
            |b, _| {
                b.iter(|| black_box(board.has_multiple_immediate_winning_moves_for(color)));
            },
        );
    }

    group.finish();
}

fn bench_apply_and_undo(c: &mut Criterion) {
    let mut group = c.benchmark_group("core/apply_move_then_undo");

    for scenario in scenarios::SCENARIOS {
        let board = scenario.board();
        let mv = scenario.probe_move();
        group.bench_with_input(
            BenchmarkId::from_parameter(scenario.id),
            scenario,
            |b, _| {
                b.iter_batched(
                    || board.clone(),
                    |mut working| {
                        black_box(working.apply_move(mv).unwrap());
                        working.undo_move(mv);
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

fn bench_renju_forbidden_moves(c: &mut Criterion) {
    let mut group = c.benchmark_group("core/forbidden_moves/current_player");

    for scenario in scenarios::SCENARIOS.iter().filter(|scenario| {
        scenario.variant == gomoku_core::Variant::Renju && scenario.to_move == Color::Black
    }) {
        let board = scenario.board();
        group.bench_with_input(
            BenchmarkId::from_parameter(scenario.id),
            scenario,
            |b, _| {
                b.iter(|| black_box(board.forbidden_moves_for_current_player()));
            },
        );
    }

    group.finish();
}

fn candidate_moves(board: &Board, radius: usize) -> Vec<Move> {
    let size = board.config.board_size;
    let radius = radius as isize;
    let mut seen = vec![false; size * size];

    for row in 0..size {
        for col in 0..size {
            if board.cell(row, col).is_none() {
                continue;
            }

            for dr in -radius..=radius {
                for dc in -radius..=radius {
                    let r = row as isize + dr;
                    let c = col as isize + dc;
                    if r < 0 || r >= size as isize || c < 0 || c >= size as isize {
                        continue;
                    }

                    let row = r as usize;
                    let col = c as usize;
                    if board.cell(row, col).is_none() {
                        seen[row * size + col] = true;
                    }
                }
            }
        }
    }

    let mut moves = Vec::new();
    for row in 0..size {
        for col in 0..size {
            if seen[row * size + col] {
                moves.push(Move { row, col });
            }
        }
    }
    moves
}

fn bench_candidate_legality(c: &mut Criterion) {
    let mut group = c.benchmark_group("core/candidate_legality/current_player");

    for scenario in scenarios::SCENARIOS.iter().filter(|scenario| {
        scenario.variant == gomoku_core::Variant::Renju && scenario.to_move == Color::Black
    }) {
        let board = scenario.board();
        let candidates = candidate_moves(&board, 2);
        group.bench_with_input(
            BenchmarkId::from_parameter(scenario.id),
            scenario,
            |b, _| {
                b.iter(|| {
                    black_box(
                        candidates
                            .iter()
                            .copied()
                            .filter(|&mv| board.is_legal(mv))
                            .count(),
                    )
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    name = board_perf;
    config = criterion_config();
    targets =
        bench_immediate_winning_moves,
        bench_has_multiple_immediate_winning_moves,
        bench_apply_and_undo,
        bench_renju_forbidden_moves,
        bench_candidate_legality
);
criterion_main!(board_perf);
