use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};

use gomoku_bot::{Bot, SearchBot};

#[path = "../../benchmarks/scenarios.rs"]
mod scenarios;

fn criterion_config() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3))
        .sample_size(20)
        .without_plots()
}

fn bench_choose_move_depth3(c: &mut Criterion) {
    let mut group = c.benchmark_group("search/choose_move/depth3");

    for scenario in scenarios::SCENARIOS {
        group.bench_with_input(
            BenchmarkId::from_parameter(scenario.id),
            scenario,
            |b, scenario| {
                b.iter_batched(
                    || (SearchBot::new(3), scenario.board()),
                    |(mut bot, board)| black_box(bot.choose_move(&board)),
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(
    name = search_perf;
    config = criterion_config();
    targets = bench_choose_move_depth3
);
criterion_main!(search_perf);
