use gomoku_bot::{SafetyGate, SearchBotConfig};

pub struct LabSearchConfig {
    pub id: &'static str,
    pub config: SearchBotConfig,
}

pub const LAB_SEARCH_CONFIGS: &[LabSearchConfig] = &[
    LabSearchConfig {
        id: "fast",
        config: SearchBotConfig {
            max_depth: 2,
            time_budget_ms: None,
            cpu_time_budget_ms: None,
            candidate_radius: 2,
            safety_gate: SafetyGate::OpponentReplySearchProbe,
        },
    },
    LabSearchConfig {
        id: "balanced",
        config: SearchBotConfig {
            max_depth: 3,
            time_budget_ms: None,
            cpu_time_budget_ms: None,
            candidate_radius: 2,
            safety_gate: SafetyGate::OpponentReplySearchProbe,
        },
    },
    LabSearchConfig {
        id: "deep",
        config: SearchBotConfig {
            max_depth: 5,
            time_budget_ms: None,
            cpu_time_budget_ms: None,
            candidate_radius: 2,
            safety_gate: SafetyGate::OpponentReplySearchProbe,
        },
    },
];

pub fn lab_search_config(id: &str) -> Option<&'static LabSearchConfig> {
    LAB_SEARCH_CONFIGS.iter().find(|config| config.id == id)
}

#[allow(dead_code)]
pub fn search_config_from_lab_spec(
    spec: &str,
    default_depth: i32,
    time_budget_ms: Option<u64>,
    cpu_time_budget_ms: Option<u64>,
) -> Option<SearchBotConfig> {
    let spec = spec.trim();
    let mut parts = spec.split('+');
    let base = parts.next().unwrap_or_default();

    let mut config = base_search_config(base, default_depth, time_budget_ms, cpu_time_budget_ms)?;

    for suffix in parts {
        config = apply_lab_suffix(config, suffix)?;
    }

    Some(config)
}

fn apply_lab_suffix(mut config: SearchBotConfig, suffix: &str) -> Option<SearchBotConfig> {
    match suffix {
        "near-all-r1" => {
            config.candidate_radius = 1;
            Some(config)
        }
        "near-all-r2" => {
            config.candidate_radius = 2;
            Some(config)
        }
        "near-all-r3" => {
            config.candidate_radius = 3;
            Some(config)
        }
        "no-safety" => {
            config.safety_gate = SafetyGate::None;
            Some(config)
        }
        "opponent-reply-search-probe" => {
            config.safety_gate = SafetyGate::OpponentReplySearchProbe;
            Some(config)
        }
        _ => None,
    }
}

fn base_search_config(
    spec: &str,
    default_depth: i32,
    time_budget_ms: Option<u64>,
    cpu_time_budget_ms: Option<u64>,
) -> Option<SearchBotConfig> {
    if spec == "baseline" || spec == "search" {
        let mut config = SearchBotConfig::custom_depth(default_depth);
        config.time_budget_ms = time_budget_ms;
        config.cpu_time_budget_ms = cpu_time_budget_ms;
        if time_budget_ms.is_some() || cpu_time_budget_ms.is_some() {
            config.max_depth = 20;
        }
        return Some(config);
    }

    if let Some(depth) = spec
        .strip_prefix("baseline-")
        .or_else(|| spec.strip_prefix("search-"))
        .and_then(parse_depth_alias)
    {
        return Some(with_budgets(
            SearchBotConfig::custom_depth(depth),
            time_budget_ms,
            cpu_time_budget_ms,
        ));
    }

    let alias = spec
        .strip_prefix("baseline-")
        .or_else(|| spec.strip_prefix("search-"))
        .unwrap_or(spec);

    lab_search_config(alias).map(|lab_config| {
        with_budgets(
            lab_config.config,
            time_budget_ms.or(lab_config.config.time_budget_ms),
            cpu_time_budget_ms.or(lab_config.config.cpu_time_budget_ms),
        )
    })
}

fn parse_depth_alias(alias: &str) -> Option<i32> {
    alias.strip_prefix('d').unwrap_or(alias).parse().ok()
}

fn with_budgets(
    mut config: SearchBotConfig,
    time_budget_ms: Option<u64>,
    cpu_time_budget_ms: Option<u64>,
) -> SearchBotConfig {
    config.time_budget_ms = time_budget_ms;
    config.cpu_time_budget_ms = cpu_time_budget_ms;
    config
}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_explicit_depth_specs() {
        let config = super::search_config_from_lab_spec("search-d3", 5, Some(1000), None)
            .expect("expected search spec to parse");

        assert_eq!(config.max_depth, 3);
        assert_eq!(config.time_budget_ms, Some(1000));
        assert_eq!(config.cpu_time_budget_ms, None);
        assert_eq!(config.candidate_radius, 2);
        assert_eq!(
            config.safety_gate,
            super::SafetyGate::OpponentReplySearchProbe
        );
    }

    #[test]
    fn preserves_legacy_depth_specs_and_named_aliases() {
        let legacy_depth = super::search_config_from_lab_spec("search-3", 5, None, Some(250))
            .expect("expected legacy depth spec to parse");
        assert_eq!(legacy_depth.max_depth, 3);
        assert_eq!(legacy_depth.cpu_time_budget_ms, Some(250));

        let alias = super::search_config_from_lab_spec("balanced", 5, Some(1000), None)
            .expect("expected named alias to parse");
        assert_eq!(alias.max_depth, 3);
        assert_eq!(alias.time_budget_ms, Some(1000));
    }

    #[test]
    fn parses_lab_no_safety_suffix() {
        let depth_spec =
            super::search_config_from_lab_spec("search-d3+no-safety", 5, Some(1000), None)
                .expect("expected no-safety search spec to parse");
        assert_eq!(depth_spec.max_depth, 3);
        assert_eq!(depth_spec.time_budget_ms, Some(1000));
        assert_eq!(depth_spec.cpu_time_budget_ms, None);
        assert_eq!(depth_spec.candidate_radius, 2);
        assert_eq!(depth_spec.safety_gate, super::SafetyGate::None);

        let alias = super::search_config_from_lab_spec("balanced+no-safety", 5, None, Some(250))
            .expect("expected no-safety alias spec to parse");
        assert_eq!(alias.max_depth, 3);
        assert_eq!(alias.cpu_time_budget_ms, Some(250));
        assert_eq!(alias.safety_gate, super::SafetyGate::None);

        let explicit = super::search_config_from_lab_spec(
            "balanced+no-safety+opponent-reply-search-probe",
            5,
            None,
            None,
        )
        .expect("expected explicit safety gate suffix to parse");
        assert_eq!(explicit.max_depth, 3);
        assert_eq!(
            explicit.safety_gate,
            super::SafetyGate::OpponentReplySearchProbe
        );
    }

    #[test]
    fn parses_near_all_radius_suffixes() {
        let r1 = super::search_config_from_lab_spec("search-d3+near-all-r1", 5, None, None)
            .expect("expected near-all-r1 search spec to parse");
        assert_eq!(r1.max_depth, 3);
        assert_eq!(r1.candidate_radius, 1);
        assert_eq!(r1.safety_gate, super::SafetyGate::OpponentReplySearchProbe);

        let r3 = super::search_config_from_lab_spec(
            "balanced+near-all-r3+no-safety",
            5,
            Some(1000),
            None,
        )
        .expect("expected combined radius and safety suffixes to parse");
        assert_eq!(r3.max_depth, 3);
        assert_eq!(r3.time_budget_ms, Some(1000));
        assert_eq!(r3.candidate_radius, 3);
        assert_eq!(r3.safety_gate, super::SafetyGate::None);
    }

    #[test]
    fn rejects_tactical_feature_flags() {
        assert!(super::search_config_from_lab_spec("search-d3+magic", 5, None, None).is_none());
        assert!(
            super::search_config_from_lab_spec("search-d3+candidates", 5, None, None).is_none()
        );
        assert!(super::search_config_from_lab_spec("search-d3+ordering", 5, None, None).is_none());
        assert!(super::search_config_from_lab_spec("search-d3+eval", 5, None, None).is_none());
        assert!(super::search_config_from_lab_spec("search-d3+all", 5, None, None).is_none());
        assert!(
            super::search_config_from_lab_spec("search-d3+shape-eval", 5, None, None).is_none()
        );
        assert!(
            super::search_config_from_lab_spec("search-d3+no-prefilter", 5, None, None).is_none()
        );
        assert!(
            super::search_config_from_lab_spec("search-d3+near-all-r0", 5, None, None).is_none()
        );
        assert!(
            super::search_config_from_lab_spec("search-d3+near-all-r4", 5, None, None).is_none()
        );
    }

    #[test]
    fn rejects_bare_depth_specs() {
        assert!(super::search_config_from_lab_spec("d3", 5, None, None).is_none());
        assert!(super::search_config_from_lab_spec("3", 5, None, None).is_none());
    }
}
