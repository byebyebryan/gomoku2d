#[cfg(test)]
use crate::CorridorProofConfig;
use crate::{
    MoveOrdering, NullCellCulling, SafetyGate, SearchBotConfig, StaticEvaluation, ThreatViewMode,
};

pub fn search_config_from_lab_spec(
    spec: &str,
    time_budget_ms: Option<u64>,
    cpu_time_budget_ms: Option<u64>,
) -> Option<SearchBotConfig> {
    let spec = spec.trim();
    let mut parts = spec.split('+');
    let base = parts.next().unwrap_or_default();

    let mut config = base_search_config(base, time_budget_ms, cpu_time_budget_ms)?;

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
        config.move_ordering = MoveOrdering::Tactical;
        config.child_limit = Some(limit);
        return Some(config);
    }

    if let Some(limit) = suffix.strip_prefix("tactical-full-cap-") {
        let limit = parse_positive_limit(limit)?;
        config.move_ordering = MoveOrdering::TacticalFull;
        config.child_limit = Some(limit);
        return Some(config);
    }

    if let Some(config) = apply_corridor_proof_suffix(config, suffix) {
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
        "tactical-full" => {
            config.move_ordering = MoveOrdering::TacticalFull;
            Some(config)
        }
        "pattern-eval" => {
            config.static_eval = StaticEvaluation::PatternEval;
            Some(config)
        }
        "null-cull" => {
            config.null_cell_culling = NullCellCulling::Enabled;
            Some(config)
        }
        "scan-threat-view" => {
            config.threat_view_mode = ThreatViewMode::Scan;
            Some(config)
        }
        "rolling-frontier" => {
            config.threat_view_mode = ThreatViewMode::Rolling;
            Some(config)
        }
        "rolling-frontier-shadow" => {
            config.threat_view_mode = ThreatViewMode::RollingShadow;
            Some(config)
        }
        _ => apply_asymmetric_candidate_source_suffix(config, suffix),
    }
}

fn apply_corridor_proof_suffix(
    mut config: SearchBotConfig,
    suffix: &str,
) -> Option<SearchBotConfig> {
    let suffix = suffix.strip_prefix("corridor-proof-c")?;
    let (proof_candidate_limit, suffix) = suffix.split_once("-d")?;
    let (max_depth, suffix) = suffix.split_once("-w")?;
    let max_reply_width = suffix;

    config.corridor_proof.enabled = true;
    config.corridor_proof.proof_candidate_limit = parse_positive_limit(proof_candidate_limit)?;
    config.corridor_proof.max_depth = parse_positive_limit(max_depth)?;
    config.corridor_proof.max_reply_width = parse_positive_limit(max_reply_width)?;
    Some(config)
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
    time_budget_ms: Option<u64>,
    cpu_time_budget_ms: Option<u64>,
) -> Option<SearchBotConfig> {
    let depth = spec.strip_prefix("search-d")?.parse().ok()?;
    Some(with_budgets(
        SearchBotConfig::custom_depth(depth),
        time_budget_ms,
        cpu_time_budget_ms,
    ))
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
        let config = super::search_config_from_lab_spec("search-d3", Some(1000), None)
            .expect("expected search spec to parse");

        assert_eq!(config.max_depth, 3);
        assert_eq!(config.time_budget_ms, Some(1000));
        assert_eq!(config.cpu_time_budget_ms, None);
        assert_eq!(config.candidate_radius, 2);
        assert_eq!(config.safety_gate, super::SafetyGate::CurrentObligation);
        assert_eq!(config.threat_view_mode, super::ThreatViewMode::Rolling);
    }

    #[test]
    fn rejects_legacy_depth_specs_and_named_aliases() {
        for spec in [
            "search",
            "baseline",
            "search-3",
            "baseline-d3",
            "fast",
            "balanced",
            "deep",
        ] {
            assert!(
                super::search_config_from_lab_spec(spec, Some(1000), None).is_none(),
                "expected legacy spec '{spec}' to be rejected"
            );
        }
    }

    #[test]
    fn parses_lab_no_safety_suffix() {
        let depth_spec =
            super::search_config_from_lab_spec("search-d3+no-safety", Some(1000), None)
                .expect("expected no-safety search spec to parse");
        assert_eq!(depth_spec.max_depth, 3);
        assert_eq!(depth_spec.time_budget_ms, Some(1000));
        assert_eq!(depth_spec.cpu_time_budget_ms, None);
        assert_eq!(depth_spec.candidate_radius, 2);
        assert_eq!(depth_spec.safety_gate, super::SafetyGate::None);

        assert!(super::search_config_from_lab_spec(
            "search-d3+no-safety+opponent-reply-search-probe",
            None,
            None,
        )
        .is_none());
    }

    #[test]
    fn parses_tactical_full_ordering_suffixes() {
        let config = super::search_config_from_lab_spec("search-d3+tactical-full", None, None)
            .expect("expected tactical-full ordering spec to parse");

        assert_eq!(config.move_ordering, super::MoveOrdering::TacticalFull);

        let capped =
            super::search_config_from_lab_spec("search-d7+tactical-full-cap-8", None, None)
                .expect("expected tactical-full cap shorthand spec to parse");
        assert_eq!(capped.max_depth, 7);
        assert_eq!(capped.move_ordering, super::MoveOrdering::TacticalFull);
        assert_eq!(capped.child_limit, Some(8));
        assert!(
            super::search_config_from_lab_spec("search-d7+tactical-full-cap-0", None, None)
                .is_none()
        );
    }

    #[test]
    fn parses_pattern_eval_suffix() {
        let config = super::search_config_from_lab_spec("search-d3+pattern-eval", None, None)
            .expect("expected pattern eval spec to parse");

        assert_eq!(config.static_eval, super::StaticEvaluation::PatternEval);
    }

    #[test]
    fn parses_null_cell_culling_suffix() {
        let config = super::search_config_from_lab_spec("search-d3+null-cull", None, None)
            .expect("expected null-cull spec to parse");

        assert_eq!(config.null_cell_culling, super::NullCellCulling::Enabled);
    }

    #[test]
    fn parses_threat_view_suffixes() {
        let default = super::search_config_from_lab_spec("search-d3", None, None)
            .expect("expected default search spec to parse");
        assert_eq!(default.threat_view_mode, super::ThreatViewMode::Rolling);

        let scan = super::search_config_from_lab_spec("search-d3+scan-threat-view", None, None)
            .expect("expected scan threat-view spec to parse");
        assert_eq!(scan.threat_view_mode, super::ThreatViewMode::Scan);

        let rolling = super::search_config_from_lab_spec("search-d3+rolling-frontier", None, None)
            .expect("expected rolling frontier spec to parse");
        assert_eq!(rolling.threat_view_mode, super::ThreatViewMode::Rolling);

        let shadow =
            super::search_config_from_lab_spec("search-d3+rolling-frontier-shadow", None, None)
                .expect("expected rolling frontier shadow spec to parse");
        assert_eq!(
            shadow.threat_view_mode,
            super::ThreatViewMode::RollingShadow
        );
    }

    #[test]
    fn parses_child_cap_suffix() {
        let config =
            super::search_config_from_lab_spec("search-d5+tactical-full+child-cap-12", None, None)
                .expect("expected child cap spec to parse");

        assert_eq!(config.child_limit, Some(12));
        assert_eq!(config.move_ordering, super::MoveOrdering::TacticalFull);
        assert!(super::search_config_from_lab_spec("search-d5+child-cap-0", None, None).is_none());
    }

    #[test]
    fn parses_tactical_cap_shorthand_suffix() {
        let config = super::search_config_from_lab_spec("search-d7+tactical-cap-8", None, None)
            .expect("expected tactical cap shorthand spec to parse");

        assert_eq!(config.max_depth, 7);
        assert_eq!(config.move_ordering, super::MoveOrdering::Tactical);
        assert_eq!(config.child_limit, Some(8));
        assert!(
            super::search_config_from_lab_spec("search-d7+tactical-cap-0", None, None).is_none()
        );
    }

    #[test]
    fn parses_near_all_radius_suffixes() {
        let r1 = super::search_config_from_lab_spec("search-d3+near-all-r1", None, None)
            .expect("expected near-all-r1 search spec to parse");
        assert_eq!(r1.max_depth, 3);
        assert_eq!(r1.candidate_radius, 1);
        assert_eq!(r1.candidate_opponent_radius, None);
        assert_eq!(r1.safety_gate, super::SafetyGate::CurrentObligation);

        let r3 =
            super::search_config_from_lab_spec("search-d3+near-all-r3+no-safety", Some(1000), None)
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
            crate::CandidateSource::NearSelfOpponent {
                self_radius: 2,
                opponent_radius: 1
            }
        );
        assert_eq!(config.cpu_time_budget_ms, Some(1000));
    }

    #[test]
    fn parses_corridor_proof_suffix() {
        let config = super::search_config_from_lab_spec(
            "search-d5+corridor-proof-c16-d8-w3",
            None,
            Some(1000),
        )
        .expect("expected corridor proof suffix to parse");

        assert_eq!(
            config.corridor_proof,
            super::CorridorProofConfig {
                enabled: true,
                max_depth: 8,
                max_reply_width: 3,
                proof_candidate_limit: 16,
            }
        );
        assert_eq!(config.cpu_time_budget_ms, Some(1000));

        for spec in [
            "search-d5+corridor-proof-c0-d8-w3",
            "search-d5+corridor-proof-c16-d0-w3",
            "search-d5+corridor-proof-c16-d8-w0",
            "search-d5+corridor-proof-c16-d8-w3-margin-50000",
        ] {
            assert!(
                super::search_config_from_lab_spec(spec, None, None).is_none(),
                "expected invalid corridor proof spec '{spec}' to be rejected"
            );
        }
    }

    #[test]
    fn rejects_tactical_feature_flags() {
        assert!(super::search_config_from_lab_spec("search-d3+magic", None, None).is_none());
        assert!(super::search_config_from_lab_spec("search-d3+candidates", None, None).is_none());
        assert!(super::search_config_from_lab_spec("search-d3+ordering", None, None).is_none());
        assert!(super::search_config_from_lab_spec("search-d3+eval", None, None).is_none());
        assert!(super::search_config_from_lab_spec("search-d3+all", None, None).is_none());
        assert!(super::search_config_from_lab_spec("search-d3+shape-eval", None, None).is_none());
        assert!(super::search_config_from_lab_spec("search-d3+near-all-r0", None, None).is_none());
        assert!(super::search_config_from_lab_spec("search-d3+near-all-r4", None, None).is_none());
        assert!(super::search_config_from_lab_spec("search-d3+corridor", None, None).is_none());
        assert!(super::search_config_from_lab_spec("search-d3+corridor-q", None, None).is_none());
        assert!(super::search_config_from_lab_spec("search-d3+corridor-qd4", None, None).is_none());
        assert!(super::search_config_from_lab_spec(
            "search-d3+near-self-r0-opponent-r1",
            None,
            None
        )
        .is_none());
        assert!(super::search_config_from_lab_spec(
            "search-d3+near-self-r2-opponent-r2",
            None,
            None
        )
        .is_none());
    }

    #[test]
    fn rejects_bare_depth_specs() {
        assert!(super::search_config_from_lab_spec("d3", None, None).is_none());
        assert!(super::search_config_from_lab_spec("3", None, None).is_none());
    }
}
