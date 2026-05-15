use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;

use gomoku_bot::frontier::RollingThreatFrontier;
use gomoku_bot::search::{
    pipeline_bench_candidate_moves, pipeline_bench_evaluate, pipeline_bench_evaluate_static,
};
use gomoku_bot::tactical::SearchThreatPolicy;
use gomoku_bot::StaticEvaluation;
use gomoku_core::{Board, Color, Move};

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

fn bench_pattern_static_eval(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline/static_eval/pattern_eval/current_player");

    for scenario in scenarios::SCENARIOS {
        let board = scenario.board();
        let color = board.current_player;
        group.bench_with_input(
            BenchmarkId::from_parameter(scenario.id),
            scenario,
            |b, _| {
                b.iter(|| {
                    black_box(pipeline_bench_evaluate_static(
                        &board,
                        color,
                        StaticEvaluation::PatternEval,
                    ))
                })
            },
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

fn bench_tactical_ordering_summary(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline/tactical_ordering_summary/current_player");

    for scenario in scenarios::SCENARIOS {
        let board = scenario.board();
        let player = board.current_player;
        let probe = scenario.probe_move();
        let dirty = dirty_tactical_input(&board, probe).unwrap_or_else(|| {
            panic!(
                "scenario '{}' could not build a dirty tactical ordering input near {}",
                scenario.id, scenario.probe_move
            )
        });

        group.bench_with_input(
            BenchmarkId::new("scan", scenario.id),
            &(board.clone(), player, probe),
            |b, (board, player, probe)| {
                b.iter(|| {
                    black_box(
                        SearchThreatPolicy
                            .ordering_summary_for_legal_player(board, *player, *probe),
                    )
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("rolling_clean", scenario.id),
            &(RollingThreatFrontier::from_board(&board), player, probe),
            |b, (frontier, player, probe)| {
                b.iter(|| {
                    black_box(
                        frontier
                            .search_ordering_summary_for_legal_player_with_source(*player, *probe)
                            .0,
                    )
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("scan_dirty_equivalent", scenario.id),
            &dirty,
            |b, dirty| {
                b.iter(|| {
                    black_box(SearchThreatPolicy.ordering_summary_for_legal_player(
                        &dirty.board,
                        dirty.player,
                        dirty.probe,
                    ))
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("dirty_raw_summary", scenario.id),
            &dirty,
            |b, dirty| {
                b.iter(|| {
                    black_box(SearchThreatPolicy.raw_ordering_summary_for_legal_player(
                        &dirty.board,
                        dirty.player,
                        dirty.probe,
                    ))
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("dirty_raw_annotation", scenario.id),
            &dirty,
            |b, dirty| {
                b.iter(|| {
                    black_box(SearchThreatPolicy.raw_annotation_for_legal_player(
                        &dirty.board,
                        dirty.player,
                        dirty.probe,
                    ))
                })
            },
        );

        let raw_annotation = SearchThreatPolicy.raw_annotation_for_legal_player(
            &dirty.board,
            dirty.player,
            dirty.probe,
        );
        group.bench_with_input(
            BenchmarkId::new("dirty_effective_from_annotation", scenario.id),
            &(dirty.board.clone(), raw_annotation),
            |b, (board, annotation)| {
                b.iter(|| {
                    black_box(
                        SearchThreatPolicy.effective_ordering_summary_from_raw(board, annotation),
                    )
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("rolling_dirty", scenario.id),
            &dirty,
            |b, dirty| {
                b.iter_batched_ref(
                    || {
                        let mut frontier = RollingThreatFrontier::from_board(&dirty.source_board);
                        frontier.apply_move(dirty.dirtying_move).unwrap();
                        frontier
                    },
                    |frontier| {
                        black_box(
                            frontier
                                .search_ordering_summary_for_legal_player_with_source(
                                    dirty.player,
                                    dirty.probe,
                                )
                                .0,
                        )
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );

        group.bench_with_input(
            BenchmarkId::new("rolling_lite_clean", scenario.id),
            &(RollingThreatFrontier::from_board(&board), player, probe),
            |b, (frontier, player, probe)| {
                b.iter(|| {
                    black_box(
                        frontier
                            .tactical_lite_rank_for_player_with_source(*player, *probe)
                            .0,
                    )
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("rolling_lite_dirty", scenario.id),
            &dirty,
            |b, dirty| {
                b.iter_batched_ref(
                    || {
                        let mut frontier = RollingThreatFrontier::from_board(&dirty.source_board);
                        frontier.apply_move(dirty.dirtying_move).unwrap();
                        frontier
                    },
                    |frontier| {
                        black_box(
                            frontier
                                .tactical_lite_rank_for_player_with_source(
                                    dirty.player,
                                    dirty.probe,
                                )
                                .0,
                        )
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
}

#[derive(Clone)]
struct DirtyTacticalInput {
    source_board: Board,
    dirtying_move: Move,
    board: Board,
    player: Color,
    probe: Move,
}

fn dirty_tactical_input(board: &Board, preferred_probe: Move) -> Option<DirtyTacticalInput> {
    let dirtying_player = board.current_player;
    let query_player = dirtying_player.opponent();

    for probe in nearby_moves(board, preferred_probe, query_player) {
        for dirtying_move in nearby_moves(board, probe, dirtying_player) {
            if dirtying_move == probe {
                continue;
            }

            let mut dirty_board = board.clone();
            dirty_board.apply_move(dirtying_move).ok()?;
            if dirty_board.is_legal_for_color(probe, query_player) {
                return Some(DirtyTacticalInput {
                    source_board: board.clone(),
                    dirtying_move,
                    board: dirty_board,
                    player: query_player,
                    probe,
                });
            }
        }
    }

    None
}

fn nearby_moves(board: &Board, center: Move, player: Color) -> Vec<Move> {
    let size = board.config.board_size;
    if size == 0 || center.row >= size || center.col >= size {
        return Vec::new();
    }

    let rmin = center.row.saturating_sub(2);
    let rmax = (center.row + 2).min(size - 1);
    let cmin = center.col.saturating_sub(2);
    let cmax = (center.col + 2).min(size - 1);
    let mut moves = Vec::new();
    for row in rmin..=rmax {
        for col in cmin..=cmax {
            let mv = Move { row, col };
            if board.is_empty(row, col) && board.is_legal_for_color(mv, player) {
                moves.push(mv);
            }
        }
    }
    moves
}

criterion_group!(
    name = pipeline_perf;
    config = criterion_config();
    targets =
        bench_static_eval,
        bench_pattern_static_eval,
        bench_candidate_moves,
        bench_tactical_ordering_summary
);
criterion_main!(pipeline_perf);
