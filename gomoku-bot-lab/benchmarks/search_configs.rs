use gomoku_bot::SearchBotConfig;

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
            candidate_radius: 2,
            root_prefilter: true,
        },
    },
    LabSearchConfig {
        id: "balanced",
        config: SearchBotConfig {
            max_depth: 3,
            time_budget_ms: None,
            candidate_radius: 2,
            root_prefilter: true,
        },
    },
    LabSearchConfig {
        id: "deep",
        config: SearchBotConfig {
            max_depth: 5,
            time_budget_ms: None,
            candidate_radius: 2,
            root_prefilter: true,
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
) -> Option<SearchBotConfig> {
    let spec = spec.trim();

    if spec == "baseline" || spec == "search" {
        return Some(match time_budget_ms {
            Some(ms) => SearchBotConfig::custom_time_budget(ms),
            None => SearchBotConfig::custom_depth(default_depth),
        });
    }

    if let Some(depth) = spec
        .strip_prefix("baseline-")
        .or_else(|| spec.strip_prefix("search-"))
        .and_then(|value| value.parse::<i32>().ok())
    {
        return Some(SearchBotConfig::custom_depth(depth));
    }

    let alias = spec
        .strip_prefix("baseline-")
        .or_else(|| spec.strip_prefix("search-"))
        .unwrap_or(spec);

    lab_search_config(alias).map(|lab_config| lab_config.config)
}
