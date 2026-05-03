use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;

use gomoku_bot::search::{pipeline_bench_candidate_moves, pipeline_bench_evaluate};

#[path = "../../benchmarks/scenarios.rs"]
mod scenarios;

fn criterion_config() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(2))
        .sample_size(20)
        .without_plots()
}

fn bench_static_eval(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline/static_eval/current_player");

    for scenario in scenarios::SCENARIOS {
        let board = scenario.board();
        let color = board.current_player;
        group.bench_with_input(
            BenchmarkId::from_parameter(scenario.id),
            scenario,
            |b, _| b.iter(|| black_box(pipeline_bench_evaluate(&board, color))),
        );
    }

    group.finish();
}

fn bench_candidate_moves(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline/candidate_moves/r2");

    for scenario in scenarios::SCENARIOS {
        let board = scenario.board();
        group.bench_with_input(
            BenchmarkId::from_parameter(scenario.id),
            scenario,
            |b, _| b.iter(|| black_box(pipeline_bench_candidate_moves(&board, 2))),
        );
    }

    group.finish();
}

criterion_group!(
    name = pipeline_perf;
    config = criterion_config();
    targets =
        bench_static_eval,
        bench_candidate_moves
);
criterion_main!(pipeline_perf);
