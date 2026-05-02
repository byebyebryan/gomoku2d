use std::time::Duration;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use std::hint::black_box;

use gomoku_bot::{Bot, SearchBot};

#[path = "../../benchmarks/scenarios.rs"]
mod scenarios;

#[path = "../../benchmarks/search_configs.rs"]
mod search_configs;

fn criterion_config() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3))
        .sample_size(20)
        .without_plots()
}

fn bench_choose_move_lab_configs(c: &mut Criterion) {
    let mut group = c.benchmark_group("search/choose_move/lab_configs");

    for lab_config in search_configs::LAB_SEARCH_CONFIGS {
        for scenario in scenarios::SCENARIOS {
            group.bench_with_input(
                BenchmarkId::new(lab_config.id, scenario.id),
                &(lab_config.config, scenario),
                |b, (config, scenario)| {
                    b.iter_batched(
                        || (SearchBot::with_config(*config), scenario.board()),
                        |(mut bot, board)| black_box(bot.choose_move(&board)),
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }

    group.finish();
}

criterion_group!(
    name = search_perf;
    config = criterion_config();
    targets = bench_choose_move_lab_configs
);
criterion_main!(search_perf);
