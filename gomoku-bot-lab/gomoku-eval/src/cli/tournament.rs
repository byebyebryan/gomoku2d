use super::*;

pub(super) type BotFactory = TournamentBotFactory;
pub(super) type NamedBotFactory = (String, BotFactory);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TournamentPlan {
    pub(super) bot_names: Vec<String>,
    pub(super) anchor_names: Vec<String>,
    pub(super) anchor_report: Option<String>,
    pub(super) pairs: Vec<TournamentPair>,
}

pub(super) fn tournament_plan(
    schedule: CliTournamentSchedule,
    bots: Option<&str>,
    candidate: Option<&str>,
    candidates: Option<&str>,
    anchors: Option<&str>,
    anchor_report: Option<&str>,
) -> Result<TournamentPlan, String> {
    match schedule {
        CliTournamentSchedule::RoundRobin => {
            reject_anchor_report_args(schedule, anchor_report)?;
            reject_gauntlet_args(schedule, candidate, candidates, anchors)?;
            let bot_names =
                parse_required_bot_list(bots, "Round-robin tournament requires --bots.")?;
            if bot_names.len() < 2 {
                return Err("Round-robin tournament requires at least 2 bots.".to_string());
            }
            validate_unique_bot_names(&bot_names)?;
            let pairs = round_robin_pairs(bot_names.len());
            Ok(TournamentPlan {
                bot_names,
                anchor_names: Vec::new(),
                anchor_report: None,
                pairs,
            })
        }
        CliTournamentSchedule::HeadToHead => {
            reject_anchor_report_args(schedule, anchor_report)?;
            reject_gauntlet_args(schedule, candidate, candidates, anchors)?;
            let bot_names =
                parse_required_bot_list(bots, "Head-to-head tournament requires --bots.")?;
            if bot_names.len() != 2 {
                return Err("Head-to-head tournament requires exactly 2 bots.".to_string());
            }
            validate_unique_bot_names(&bot_names)?;
            Ok(TournamentPlan {
                bot_names,
                anchor_names: Vec::new(),
                anchor_report: None,
                pairs: vec![TournamentPair {
                    bot_a_idx: 0,
                    bot_b_idx: 1,
                }],
            })
        }
        CliTournamentSchedule::Gauntlet => {
            if bots.is_some() {
                return Err(
                    "Gauntlet tournament uses --candidate/--candidates and --anchors instead of --bots."
                        .to_string(),
                );
            }
            let candidate_names = parse_gauntlet_candidates(candidate, candidates)?;
            let anchor_names =
                parse_required_bot_list(anchors, "Gauntlet tournament requires --anchors.")?;
            if anchor_names.is_empty() {
                return Err("Gauntlet tournament requires at least 1 anchor.".to_string());
            }

            let candidate_count = candidate_names.len();
            let mut bot_names = candidate_names;
            bot_names.extend(anchor_names.clone());
            validate_unique_bot_names(&bot_names)?;
            let pairs = (0..candidate_count)
                .flat_map(|candidate_idx| {
                    (candidate_count..bot_names.len()).map(move |anchor_idx| TournamentPair {
                        bot_a_idx: candidate_idx,
                        bot_b_idx: anchor_idx,
                    })
                })
                .collect();
            Ok(TournamentPlan {
                bot_names,
                anchor_names,
                anchor_report: anchor_report.map(ToString::to_string),
                pairs,
            })
        }
    }
}

pub(super) fn parse_required_bot_list(
    input: Option<&str>,
    message: &str,
) -> Result<Vec<String>, String> {
    let Some(input) = input else {
        return Err(message.to_string());
    };
    let bot_names = parse_bot_list(input);
    if bot_names.is_empty() {
        return Err(message.to_string());
    }
    Ok(bot_names)
}

pub(super) fn parse_gauntlet_candidates(
    candidate: Option<&str>,
    candidates: Option<&str>,
) -> Result<Vec<String>, String> {
    match (candidate, candidates) {
        (Some(_), Some(_)) => Err(
            "Gauntlet tournament uses either --candidate or --candidates, not both.".to_string(),
        ),
        (Some(candidate), None) => {
            let candidate_names = parse_required_bot_list(
                Some(candidate),
                "Gauntlet tournament requires --candidate or --candidates.",
            )?;
            if candidate_names.len() != 1 {
                return Err(
                    "Gauntlet --candidate accepts exactly 1 bot; use --candidates for batch gauntlets."
                        .to_string(),
                );
            }
            Ok(candidate_names)
        }
        (None, Some(candidates)) => parse_required_bot_list(
            Some(candidates),
            "Gauntlet tournament requires --candidate or --candidates.",
        ),
        (None, None) => {
            Err("Gauntlet tournament requires --candidate or --candidates.".to_string())
        }
    }
}

pub(super) fn parse_bot_list(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToString::to_string)
        .collect()
}

pub(super) fn reject_gauntlet_args(
    schedule: CliTournamentSchedule,
    candidate: Option<&str>,
    candidates: Option<&str>,
    anchors: Option<&str>,
) -> Result<(), String> {
    if candidate.is_some() || candidates.is_some() || anchors.is_some() {
        return Err(format!(
            "{} tournament uses --bots, not --candidate/--candidates/--anchors.",
            schedule.label()
        ));
    }
    Ok(())
}

pub(super) fn reject_anchor_report_args(
    schedule: CliTournamentSchedule,
    anchor_report: Option<&str>,
) -> Result<(), String> {
    if anchor_report.is_some() {
        return Err(format!(
            "{} tournament does not use --anchor-report.",
            schedule.label()
        ));
    }
    Ok(())
}

pub(super) fn validate_unique_bot_names(bot_names: &[String]) -> Result<(), String> {
    for (idx, name) in bot_names.iter().enumerate() {
        if bot_names.iter().skip(idx + 1).any(|other| other == name) {
            return Err(format!("Duplicate bot in tournament schedule: {name}"));
        }
    }
    Ok(())
}

pub(super) fn load_anchor_reference(
    path: &PathBuf,
    source_path: String,
    anchor_names: &[String],
) -> Result<AnchorReferenceReport, String> {
    let json = std::fs::read_to_string(path)
        .map_err(|err| format!("Failed to read anchor report {}: {err}", path.display()))?;
    let value: serde_json::Value = serde_json::from_str(&json)
        .map_err(|err| format!("Failed to parse anchor report {}: {err}", path.display()))?;
    match value.get("report_kind").and_then(serde_json::Value::as_str) {
        Some("tournament") => {
            let source_report = TournamentReport::from_json(&json).map_err(|err| {
                format!("Failed to parse anchor report {}: {err}", path.display())
            })?;
            AnchorReferenceReport::from_report(Some(source_path), &source_report, anchor_names)
        }
        Some("published_tournament") => {
            let source_report = PublishedTournamentReport::from_json(&json).map_err(|err| {
                format!("Failed to parse anchor report {}: {err}", path.display())
            })?;
            AnchorReferenceReport::from_published_report(
                Some(source_path),
                &source_report,
                anchor_names,
            )
        }
        Some(other) => Err(format!("unsupported anchor report kind: {other}")),
        None => Err("anchor report is missing report_kind".to_string()),
    }
}

pub(super) fn make_bot_factory(
    spec: &str,
    search_time_ms: Option<u64>,
    search_cpu_time_ms: Option<u64>,
    search_budget_mode: CliSearchBudgetMode,
    search_cpu_reserve_ms: u64,
    search_cpu_max_move_ms: Option<u64>,
) -> Result<BotFactory, String> {
    let spec = spec.to_string();
    if spec == "random" {
        return Ok(Arc::new(|seed| Box::new(RandomBot::seeded(seed))));
    }
    if let Some(config) =
        lab_spec::search_config_from_lab_spec(&spec, search_time_ms, search_cpu_time_ms)
    {
        return match search_budget_mode {
            CliSearchBudgetMode::Strict => {
                Ok(Arc::new(move |_| Box::new(SearchBot::with_config(config))))
            }
            CliSearchBudgetMode::Pooled => {
                if search_time_ms.is_some() {
                    return Err(
                        "Pooled search budgeting currently supports --search-cpu-time-ms, not --search-time-ms."
                            .to_string(),
                    );
                }
                let Some(base_ms) = search_cpu_time_ms else {
                    return Err(
                        "Pooled search budgeting requires --search-cpu-time-ms.".to_string()
                    );
                };
                if let Some(max_move_ms) = search_cpu_max_move_ms {
                    if max_move_ms < base_ms {
                        return Err(
                            "--search-cpu-max-move-ms must be greater than or equal to --search-cpu-time-ms."
                                .to_string(),
                        );
                    }
                }
                Ok(Arc::new(move |_| {
                    Box::new(PooledSearchBot::new(
                        config,
                        PooledCpuBudgetConfig {
                            base_ms,
                            reserve_cap_ms: search_cpu_reserve_ms,
                            max_move_ms: search_cpu_max_move_ms,
                        },
                    ))
                }))
            }
        };
    }

    Err(format!(
        "Unknown bot type: '{spec}'. Use random, search-dN, or search-dN+suffixes."
    ))
}
