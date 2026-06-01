use std::time::Duration;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use std::hint::black_box;

use gomoku_bot::{lab_spec::search_config_from_lab_spec, Bot, SearchBot};
use gomoku_lab_support::scenarios;

const BENCH_SPECS: &[&str] = &[
    "search-d1",
    "search-d3+pattern-eval",
    "search-d5+tactical-cap-16+pattern-eval",
    "search-d7+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w4",
];

fn criterion_config() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3))
        .sample_size(20)
        .without_plots()
}

fn bench_choose_move_lab_configs(c: &mut Criterion) {
    let mut group = c.benchmark_group("search/choose_move/lab_configs");

    for spec in BENCH_SPECS {
        let config = search_config_from_lab_spec(spec, None, None)
            .unwrap_or_else(|| panic!("benchmark spec should parse: {spec}"));
        for scenario in scenarios::SCENARIOS {
            group.bench_with_input(
                BenchmarkId::new(*spec, scenario.id),
                &(config, scenario),
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
