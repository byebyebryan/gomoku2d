use std::path::Path;

use gomoku_bot::lab_spec;
use gomoku_eval::lethal_scenario::{
    run_lethal_scenarios, LethalScenarioReport, LethalScenarioResult, LETHAL_SCENARIO_CASES,
};
use gomoku_eval::scenario::{
    run_tactical_scenarios, ScenarioSearchConfig, TacticalScenarioGroupSummary,
    TacticalScenarioReport, TacticalScenarioResult, TACTICAL_SCENARIO_CASES,
};

pub fn run_tactical_scenarios_command(
    bots: &str,
    search_time_ms: Option<u64>,
    search_cpu_time_ms: Option<u64>,
    report_json: Option<&Path>,
) -> Result<bool, String> {
    let configs = parse_search_config_specs(bots, search_time_ms, search_cpu_time_ms)?;

    println!("--- Tactical Scenarios ---");
    println!(
        "Configs: {:?}",
        configs
            .iter()
            .map(|config| config.id.as_str())
            .collect::<Vec<_>>()
    );
    println!("Cases: {}", TACTICAL_SCENARIO_CASES.len());
    if let Some(ms) = search_time_ms {
        println!("Search time budget: {ms} ms/move");
    }
    if let Some(ms) = search_cpu_time_ms {
        println!("Search CPU-time budget: {ms} ms/move");
    }
    println!();

    let report = run_tactical_scenarios(&configs, TACTICAL_SCENARIO_CASES);
    for result in &report.results {
        print_tactical_scenario_result(result);
    }

    print_tactical_report_summary(&report);

    if let Some(path) = report_json {
        let json = report
            .to_json()
            .map_err(|err| format!("Failed to serialize tactical report: {err}"))?;
        std::fs::write(path, json)
            .map_err(|err| format!("Failed to write tactical report: {err}"))?;
        println!("Report JSON: {}", path.display());
    }

    Ok(report.hard_failed > 0)
}

pub fn run_lethal_scenarios_command(
    report_json: Option<&Path>,
    show_boards: bool,
) -> Result<bool, String> {
    println!("--- Lethal Scenarios ---");
    println!("Cases: {}", LETHAL_SCENARIO_CASES.len());
    println!();

    let report = run_lethal_scenarios(LETHAL_SCENARIO_CASES);
    for result in &report.results {
        print_lethal_scenario_result(result);
        if show_boards {
            print_lethal_scenario_board(result);
            println!();
        }
    }

    print_lethal_report_summary(&report);

    if let Some(path) = report_json {
        let json = report
            .to_json()
            .map_err(|err| format!("Failed to serialize lethal report: {err}"))?;
        std::fs::write(path, json)
            .map_err(|err| format!("Failed to write lethal report: {err}"))?;
        println!("Report JSON: {}", path.display());
    }

    Ok(report.failed > 0)
}

fn parse_search_config_specs(
    specs: &str,
    search_time_ms: Option<u64>,
    search_cpu_time_ms: Option<u64>,
) -> Result<Vec<ScenarioSearchConfig>, String> {
    let names: Vec<String> = specs
        .split(',')
        .map(|spec| spec.trim().to_string())
        .filter(|spec| !spec.is_empty())
        .collect();

    if names.is_empty() {
        return Err("At least one search config is required.".to_string());
    }

    names
        .into_iter()
        .map(|name| {
            let config =
                lab_spec::search_config_from_lab_spec(&name, 5, search_time_ms, search_cpu_time_ms)
                    .ok_or_else(|| {
                        format!(
                            "Unknown search config: '{name}'. Use search-dN or search-dN+suffixes."
                        )
                    })?;
            Ok(ScenarioSearchConfig { id: name, config })
        })
        .collect()
}

fn print_tactical_scenario_result(result: &TacticalScenarioResult) {
    let status = match result.status {
        "pass" => "PASS",
        "fail" => "FAIL",
        "hit" => "HIT",
        "miss" => "MISS",
        other => other,
    };
    let expected = if result.expected_moves.is_empty() {
        "-".to_string()
    } else {
        result.expected_moves.join("/")
    };
    let shape = result.shape.unwrap_or("-");
    println!(
        "{:<5} {:<10} {:<16} {:<8} {:<13} {:<12} {:?}/{:?} {:<48} actual {:<3} expect {:<7} depth {:>2} nodes {:>8} safety {:>5} eval {:>7} cand r/s {:>5}/{:<5} child {:>5}->{:<5} cap {:>4} legal r/s {:>6}/{:<6} tt {:>5}/{:<5} cut {:>5} time {:>4}ms",
        status,
        result.config_id,
        result.role,
        result.layer,
        result.intent,
        shape,
        result.variant,
        result.to_move,
        result.case_id,
        result.actual_move,
        expected,
        result.metrics.depth_reached,
        result.metrics.nodes,
        result.metrics.safety_nodes,
        result.metrics.eval_calls,
        result.metrics.root_candidate_generations,
        result.metrics.search_candidate_generations,
        result.metrics.child_moves_before_total,
        result.metrics.child_moves_after_total,
        result.metrics.child_cap_hits,
        result.metrics.root_legality_checks,
        result.metrics.search_legality_checks,
        result.metrics.tt_hits,
        result.metrics.tt_cutoffs,
        result.metrics.beta_cutoffs,
        result.metrics.time_ms
    );
}

fn print_tactical_group_summary(title: &str, summaries: &[TacticalScenarioGroupSummary]) {
    println!("\n{title}");
    for summary in summaries {
        println!(
            "  {:<16} {:>3}/{:<3} matched, {:>3} missed, {:>3} hard fail, avg depth {:>4.1}, avg total nodes {:>8.0}, avg safety {:>7.0}, avg time {:>5.1}ms",
            summary.key,
            summary.matched,
            summary.total,
            summary.missed,
            summary.hard_failures,
            summary.avg_depth_reached,
            summary.avg_total_nodes,
            summary.avg_safety_nodes,
            summary.avg_time_ms
        );
    }
}

fn print_tactical_report_summary(report: &TacticalScenarioReport) {
    println!(
        "\n--- Summary ---\nHard gates: {}/{} passed, {} failed\nDiagnostic probes: {}/{} hit, {} missed",
        report.hard_passed,
        report.hard_total,
        report.hard_failed,
        report.diagnostic_hits,
        report.diagnostic_total,
        report.diagnostic_misses
    );
    print_tactical_group_summary("By role", &report.role_summaries);
    print_tactical_group_summary("By layer", &report.layer_summaries);
    print_tactical_group_summary("By intent", &report.intent_summaries);
}

fn print_lethal_scenario_result(result: &LethalScenarioResult) {
    let status = if result.passed { "PASS" } else { "FAIL" };
    let kind = result
        .actual_kind
        .map(|kind| format!("{kind:?}"))
        .unwrap_or_else(|| "-".to_string());
    println!(
        "{:<5} {:?} attacker {:?} defender {:?} {:<52} kind {:<16} lethal {} expect {} targets {:<8} cover {:<8} defender-win {} escape {}",
        status,
        result.variant,
        result.attacker,
        result.defender,
        result.case_id,
        kind,
        result.actual_lethal,
        result.expected_lethal,
        display_move_list(&result.actual_terminal_targets),
        display_move_list(&result.actual_covering_replies),
        display_move_list(&result.actual_defender_immediate_wins),
        display_move_list(&result.actual_escaping_replies)
    );
}

fn print_lethal_scenario_board(result: &LethalScenarioResult) {
    println!();
    println!("{}", result.board_ascii);
}

fn print_lethal_report_summary(report: &LethalScenarioReport) {
    println!(
        "\n--- Summary ---\nLethal scenarios: {}/{} passed, {} failed",
        report.passed, report.total, report.failed
    );
}

fn display_move_list(moves: &[String]) -> String {
    if moves.is_empty() {
        "-".to_string()
    } else {
        moves.join("/")
    }
}
