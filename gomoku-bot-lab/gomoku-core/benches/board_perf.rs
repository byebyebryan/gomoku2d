use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};

use gomoku_core::Color;

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
        group.bench_with_input(BenchmarkId::from_parameter(scenario.id), scenario, |b, _| {
            b.iter(|| black_box(board.immediate_winning_moves_for(color)));
        });
    }

    group.finish();
}

fn bench_apply_and_undo(c: &mut Criterion) {
    let mut group = c.benchmark_group("core/apply_move_then_undo");

    for scenario in scenarios::SCENARIOS {
        let board = scenario.board();
        let mv = scenario.probe_move();
        group.bench_with_input(BenchmarkId::from_parameter(scenario.id), scenario, |b, _| {
            b.iter_batched(
                || board.clone(),
                |mut working| {
                    black_box(working.apply_move(mv).unwrap());
                    working.undo_move(mv);
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_renju_forbidden_moves(c: &mut Criterion) {
    let mut group = c.benchmark_group("core/forbidden_moves/current_player");

    for scenario in scenarios::SCENARIOS
        .iter()
        .filter(|scenario| scenario.variant == gomoku_core::Variant::Renju && scenario.to_move == Color::Black)
    {
        let board = scenario.board();
        group.bench_with_input(BenchmarkId::from_parameter(scenario.id), scenario, |b, _| {
            b.iter(|| black_box(board.forbidden_moves_for_current_player()));
        });
    }

    group.finish();
}

criterion_group!(
    name = board_perf;
    config = criterion_config();
    targets =
        bench_immediate_winning_moves,
        bench_apply_and_undo,
        bench_renju_forbidden_moves
);
criterion_main!(board_perf);
