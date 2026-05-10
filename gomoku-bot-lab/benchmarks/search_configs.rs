use gomoku_bot::{
    CorridorPortalConfig, CorridorPortalSideConfig, MoveOrdering, SafetyGate, SearchAlgorithm,
    SearchBotConfig, StaticEvaluation,
};

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
            candidate_opponent_radius: None,
            safety_gate: SafetyGate::OpponentReplyLocalThreatProbe,
            move_ordering: MoveOrdering::TranspositionFirstBoardOrder,
            child_limit: None,
            search_algorithm: SearchAlgorithm::AlphaBetaIterativeDeepening,
            static_eval: StaticEvaluation::LineShapeEval,
            corridor_portals: CorridorPortalConfig::DISABLED,
        },
    },
    LabSearchConfig {
        id: "balanced",
        config: SearchBotConfig {
            max_depth: 3,
            time_budget_ms: None,
            cpu_time_budget_ms: None,
            candidate_radius: 2,
            candidate_opponent_radius: None,
            safety_gate: SafetyGate::OpponentReplyLocalThreatProbe,
            move_ordering: MoveOrdering::TranspositionFirstBoardOrder,
            child_limit: None,
            search_algorithm: SearchAlgorithm::AlphaBetaIterativeDeepening,
            static_eval: StaticEvaluation::LineShapeEval,
            corridor_portals: CorridorPortalConfig::DISABLED,
        },
    },
    LabSearchConfig {
        id: "deep",
        config: SearchBotConfig {
            max_depth: 5,
            time_budget_ms: None,
            cpu_time_budget_ms: None,
            candidate_radius: 2,
            candidate_opponent_radius: None,
            safety_gate: SafetyGate::OpponentReplyLocalThreatProbe,
            move_ordering: MoveOrdering::TranspositionFirstBoardOrder,
            child_limit: None,
            search_algorithm: SearchAlgorithm::AlphaBetaIterativeDeepening,
            static_eval: StaticEvaluation::LineShapeEval,
            corridor_portals: CorridorPortalConfig::DISABLED,
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
    if let Some(limit) = suffix.strip_prefix("child-cap-") {
        let limit = parse_positive_limit(limit)?;
        config.child_limit = Some(limit);
        return Some(config);
    }

    if let Some(limit) = suffix.strip_prefix("tactical-cap-") {
        let limit = parse_positive_limit(limit)?;
        config.move_ordering = MoveOrdering::TacticalFirst;
        config.child_limit = Some(limit);
        return Some(config);
    }

    if let Some(portal) = parse_corridor_portal_suffix(suffix) {
        match portal.side {
            CorridorPortalSideSuffix::Own => config.corridor_portals.own = portal.config,
            CorridorPortalSideSuffix::Opponent => {
                config.corridor_portals.opponent = portal.config;
            }
        }
        return Some(config);
    }

    match suffix {
        "near-all-r1" => {
            config.candidate_radius = 1;
            config.candidate_opponent_radius = None;
            Some(config)
        }
        "near-all-r2" => {
            config.candidate_radius = 2;
            config.candidate_opponent_radius = None;
            Some(config)
        }
        "near-all-r3" => {
            config.candidate_radius = 3;
            config.candidate_opponent_radius = None;
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
        "opponent-reply-local-threat-probe" => {
            config.safety_gate = SafetyGate::OpponentReplyLocalThreatProbe;
            Some(config)
        }
        "tactical-first" => {
            config.move_ordering = MoveOrdering::TacticalFirst;
            Some(config)
        }
        "pattern-eval" => {
            config.static_eval = StaticEvaluation::PatternEval;
            Some(config)
        }
        _ => apply_asymmetric_candidate_source_suffix(config, suffix),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CorridorPortalSideSuffix {
    Own,
    Opponent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CorridorPortalSuffix {
    side: CorridorPortalSideSuffix,
    config: CorridorPortalSideConfig,
}

fn parse_corridor_portal_suffix(suffix: &str) -> Option<CorridorPortalSuffix> {
    let (side, suffix) = if let Some(suffix) = suffix.strip_prefix("corridor-own-d") {
        (CorridorPortalSideSuffix::Own, suffix)
    } else if let Some(suffix) = suffix.strip_prefix("corridor-opponent-d") {
        (CorridorPortalSideSuffix::Opponent, suffix)
    } else {
        return None;
    };

    let (max_depth, max_reply_width) = suffix.split_once("-w")?;
    Some(CorridorPortalSuffix {
        side,
        config: CorridorPortalSideConfig {
            enabled: true,
            max_depth: parse_positive_limit(max_depth)?,
            max_reply_width: parse_positive_limit(max_reply_width)?,
        },
    })
}

fn apply_asymmetric_candidate_source_suffix(
    mut config: SearchBotConfig,
    suffix: &str,
) -> Option<SearchBotConfig> {
    let suffix = suffix.strip_prefix("near-self-r")?;
    let (self_radius, suffix) = suffix.split_once("-opponent-r")?;
    let self_radius = parse_candidate_radius(self_radius)?;
    let opponent_radius = parse_candidate_radius(suffix)?;
    if self_radius == opponent_radius {
        return None;
    }
    config.candidate_radius = self_radius;
    config.candidate_opponent_radius = Some(opponent_radius);
    Some(config)
}

fn parse_candidate_radius(value: &str) -> Option<usize> {
    match value.parse::<usize>().ok()? {
        radius @ 1..=3 => Some(radius),
        _ => None,
    }
}

fn parse_positive_limit(value: &str) -> Option<usize> {
    let limit = value.parse::<usize>().ok()?;
    if limit == 0 {
        return None;
    }
    Some(limit)
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
            super::SafetyGate::OpponentReplyLocalThreatProbe
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
    fn parses_lab_local_threat_safety_suffix() {
        let config = super::search_config_from_lab_spec(
            "search-d3+opponent-reply-local-threat-probe",
            5,
            None,
            None,
        )
        .expect("expected local-threat safety spec to parse");

        assert_eq!(
            config.safety_gate,
            super::SafetyGate::OpponentReplyLocalThreatProbe
        );
    }

    #[test]
    fn parses_tactical_move_ordering_suffix() {
        let config = super::search_config_from_lab_spec("search-d3+tactical-first", 5, None, None)
            .expect("expected tactical ordering spec to parse");

        assert_eq!(config.move_ordering, super::MoveOrdering::TacticalFirst);
    }

    #[test]
    fn parses_pattern_eval_suffix() {
        let config = super::search_config_from_lab_spec("search-d3+pattern-eval", 5, None, None)
            .expect("expected pattern eval spec to parse");

        assert_eq!(config.static_eval, super::StaticEvaluation::PatternEval);
    }

    #[test]
    fn parses_child_cap_suffix() {
        let config = super::search_config_from_lab_spec(
            "search-d5+tactical-first+child-cap-12",
            3,
            None,
            None,
        )
        .expect("expected child cap spec to parse");

        assert_eq!(config.child_limit, Some(12));
        assert_eq!(config.move_ordering, super::MoveOrdering::TacticalFirst);
        assert!(
            super::search_config_from_lab_spec("search-d5+child-cap-0", 3, None, None).is_none()
        );
    }

    #[test]
    fn parses_tactical_cap_shorthand_suffix() {
        let config = super::search_config_from_lab_spec("search-d7+tactical-cap-8", 3, None, None)
            .expect("expected tactical cap shorthand spec to parse");

        assert_eq!(config.max_depth, 7);
        assert_eq!(config.move_ordering, super::MoveOrdering::TacticalFirst);
        assert_eq!(config.child_limit, Some(8));
        assert!(
            super::search_config_from_lab_spec("search-d7+tactical-cap-0", 3, None, None).is_none()
        );
    }

    #[test]
    fn parses_near_all_radius_suffixes() {
        let r1 = super::search_config_from_lab_spec("search-d3+near-all-r1", 5, None, None)
            .expect("expected near-all-r1 search spec to parse");
        assert_eq!(r1.max_depth, 3);
        assert_eq!(r1.candidate_radius, 1);
        assert_eq!(r1.candidate_opponent_radius, None);
        assert_eq!(
            r1.safety_gate,
            super::SafetyGate::OpponentReplyLocalThreatProbe
        );

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
        assert_eq!(r3.candidate_opponent_radius, None);
        assert_eq!(r3.safety_gate, super::SafetyGate::None);
    }

    #[test]
    fn parses_asymmetric_candidate_source_suffix() {
        let config = super::search_config_from_lab_spec(
            "search-d5+tactical-cap-8+near-self-r2-opponent-r1",
            3,
            None,
            Some(1000),
        )
        .expect("expected asymmetric candidate source spec to parse");

        assert_eq!(config.max_depth, 5);
        assert_eq!(config.child_limit, Some(8));
        assert_eq!(config.candidate_radius, 2);
        assert_eq!(config.candidate_opponent_radius, Some(1));
        assert_eq!(
            config.candidate_source(),
            gomoku_bot::CandidateSource::NearSelfOpponent {
                self_radius: 2,
                opponent_radius: 1
            }
        );
        assert_eq!(config.cpu_time_budget_ms, Some(1000));
    }

    #[test]
    fn parses_corridor_portal_suffixes() {
        let config = super::search_config_from_lab_spec(
            "search-d5+corridor-own-d6-w3+corridor-opponent-d4-w2",
            3,
            None,
            Some(1000),
        )
        .expect("expected corridor portal suffixes to parse");

        assert_eq!(config.max_depth, 5);
        assert!(config.corridor_portals.own.enabled);
        assert_eq!(config.corridor_portals.own.max_depth, 6);
        assert_eq!(config.corridor_portals.own.max_reply_width, 3);
        assert!(config.corridor_portals.opponent.enabled);
        assert_eq!(config.corridor_portals.opponent.max_depth, 4);
        assert_eq!(config.corridor_portals.opponent.max_reply_width, 2);
        assert_eq!(config.cpu_time_budget_ms, Some(1000));

        assert!(
            super::search_config_from_lab_spec("search-d5+corridor-own-d0-w3", 3, None, None)
                .is_none()
        );
        assert!(
            super::search_config_from_lab_spec("search-d5+corridor-own-d4-w0", 3, None, None)
                .is_none()
        );
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
            super::search_config_from_lab_spec("search-d3+near-all-r0", 5, None, None).is_none()
        );
        assert!(
            super::search_config_from_lab_spec("search-d3+near-all-r4", 5, None, None).is_none()
        );
        assert!(super::search_config_from_lab_spec("search-d3+corridor", 5, None, None).is_none());
        assert!(
            super::search_config_from_lab_spec("search-d3+corridor-q", 5, None, None).is_none()
        );
        assert!(
            super::search_config_from_lab_spec("search-d3+corridor-qd4", 5, None, None).is_none()
        );
        assert!(super::search_config_from_lab_spec(
            "search-d3+near-self-r0-opponent-r1",
            5,
            None,
            None
        )
        .is_none());
        assert!(super::search_config_from_lab_spec(
            "search-d3+near-self-r2-opponent-r2",
            5,
            None,
            None
        )
        .is_none());
    }

    #[test]
    fn rejects_bare_depth_specs() {
        assert!(super::search_config_from_lab_spec("d3", 5, None, None).is_none());
        assert!(super::search_config_from_lab_spec("3", 5, None, None).is_none());
    }
}
