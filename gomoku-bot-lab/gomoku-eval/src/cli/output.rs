use super::*;

pub(super) fn exit_with_error(message: impl AsRef<str>) -> ! {
    eprintln!("{}", message.as_ref());
    std::process::exit(2);
}

pub(super) fn pooled_budget_label(reserve_cap_ms: u64, max_move_ms: Option<u64>) -> String {
    match max_move_ms {
        Some(max_move_ms) => format!(
            "Search budget mode: pooled (reserve cap {reserve_cap_ms} ms, max move {max_move_ms} ms)"
        ),
        None => format!("Search budget mode: pooled (reserve cap {reserve_cap_ms} ms)"),
    }
}

pub(super) fn variant_label(variant: &Variant) -> &'static str {
    match variant {
        Variant::Freestyle => "freestyle",
        Variant::Renju => "renju",
    }
}

pub(super) fn eval_context(options: &EvalOptions) -> EvalContext {
    if options.search_budget_mode != CliSearchBudgetMode::Pooled
        && options.search_cpu_max_move_ms.is_some()
    {
        exit_with_error("--search-cpu-max-move-ms requires --search-budget-mode pooled.");
    }

    let variant = match options.rule.as_str() {
        "renju" => Variant::Renju,
        "freestyle" => Variant::Freestyle,
        other => exit_with_error(format!(
            "Unknown rule variant '{other}'. Use 'renju' or 'freestyle'."
        )),
    };
    let rule_label = variant_label(&variant);
    EvalContext {
        config: RuleConfig {
            variant,
            ..Default::default()
        },
        rule_label,
        limits: MatchLimits {
            max_moves: options.max_moves,
            max_game_ms: options.max_game_ms,
        },
        search_time_ms: options.search_time_ms,
        search_cpu_time_ms: options.search_cpu_time_ms,
        search_budget_mode: options.search_budget_mode,
        search_cpu_reserve_ms: options.search_cpu_reserve_ms,
        search_cpu_max_move_ms: options.search_cpu_max_move_ms,
        seed: options.seed,
    }
}

pub(super) fn print_move_progress(
    move_num: usize,
    game_idx: u32,
    player: Color,
    mv: Move,
    time_ms: u64,
) {
    let time_str = if time_ms >= 1000 {
        format!("{:.1}s", time_ms as f64 / 1000.0)
    } else {
        format!("{}ms", time_ms)
    };
    println!(
        "  Game {:3}  move {:3}  {} {:3}  ({})",
        game_idx + 1,
        move_num,
        match player {
            Color::Black => "B",
            Color::White => "W",
        },
        mv.to_notation(),
        time_str,
    );
}

pub(super) fn end_reason_suffix(reason: MatchEndReason) -> String {
    match reason {
        MatchEndReason::Natural => String::new(),
        reason => format!(" ({})", reason.label()),
    }
}

pub(super) fn print_game_result(i: u32, total: u32, mr: &MatchResult) {
    let suffix = end_reason_suffix(mr.end_reason);
    match &mr.result {
        GameResult::Winner(c) => println!("  Game {:3}/{:3}  {:?} wins{}", i + 1, total, c, suffix),
        GameResult::Draw => println!("  Game {:3}/{:3}  Draw{}", i + 1, total, suffix),
        GameResult::Ongoing => unreachable!(),
    }
}

pub(super) fn print_renju_rule_fixture_result(result: &RenjuRuleFixtureResult) {
    let status = if result.passed { "PASS" } else { "FAIL" };
    let expected = if result.expected_legal {
        "legal"
    } else {
        "forbidden"
    };
    let actual = if result.actual_legal {
        "legal"
    } else {
        "forbidden"
    };
    println!(
        "{:<5} {:<44} {:?} {:<3} expect {:<9} actual {:<9} source {}",
        status, result.id, result.color, result.candidate, expected, actual, result.source
    );
}

pub(super) fn print_renju_rule_fixture_board(result: &RenjuRuleFixtureResult) {
    println!();
    for row in &result.board {
        println!("{row}");
    }
}

pub(super) fn print_renju_rule_report_summary(report: &RenjuRuleReport) {
    println!(
        "\n--- Summary ---\nRenju rule fixtures: {}/{} passed, {} failed",
        report.passed, report.total, report.failed
    );
}

pub(super) fn print_analysis_fixture_result(result: &AnalysisFixtureResult) {
    let status = if result.passed { "PASS" } else { "FAIL" };
    let notes = if result.actual.tactical_notes.is_empty() {
        "-".to_string()
    } else {
        result
            .actual
            .tactical_notes
            .iter()
            .map(|note| format!("{note:?}"))
            .collect::<Vec<_>>()
            .join("/")
    };
    println!(
        "{:<5} {:<34} winner {:<7} root {:<14} forced {:>2}..{:<2} chance {:<4} loser {:<4} notes {}",
        status,
        result.case_id,
        result
            .actual
            .winner
            .map(|winner| format!("{winner:?}"))
            .unwrap_or_else(|| "-".to_string()),
        format!("{:?}", result.actual.root_cause),
        result.actual.final_forced_interval.start_ply,
        result.actual.final_forced_interval.end_ply,
        result
            .actual
            .last_chance_ply
            .map(|ply| ply.to_string())
            .unwrap_or_else(|| "-".to_string()),
        result
            .actual
            .critical_loser_ply
            .map(|ply| ply.to_string())
            .unwrap_or_else(|| "-".to_string()),
        notes
    );
    for failure in &result.failures {
        println!("      {failure}");
    }
}

pub(super) fn print_analysis_fixture_report_summary(report: &AnalysisFixtureReport) {
    println!(
        "\n--- Summary ---\n{} passed / {} total ({} failed)",
        report.passed, report.total, report.failed
    );
}

pub(super) fn print_analysis_batch_report_summary(report: &AnalysisBatchReport) {
    println!(
        "\n--- Summary ---\n{} analyzed / {} total ({} failed)",
        report.analyzed, report.total, report.failed
    );
    println!(
        "summary: total {}, unclear {}, ongoing/draw {}, errors {}",
        report.total,
        report.summary.unclear,
        report.summary.ongoing_or_draw,
        report.summary.analysis_error
    );
}
