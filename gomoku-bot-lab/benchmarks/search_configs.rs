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
            cpu_time_budget_ms: None,
            candidate_radius: 2,
            root_prefilter: true,
        },
    },
    LabSearchConfig {
        id: "balanced",
        config: SearchBotConfig {
            max_depth: 3,
            time_budget_ms: None,
            cpu_time_budget_ms: None,
            candidate_radius: 2,
            root_prefilter: true,
        },
    },
    LabSearchConfig {
        id: "deep",
        config: SearchBotConfig {
            max_depth: 5,
            time_budget_ms: None,
            cpu_time_budget_ms: None,
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
    cpu_time_budget_ms: Option<u64>,
) -> Option<SearchBotConfig> {
    let spec = spec.trim();

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
        .and_then(|value| value.parse::<i32>().ok())
    {
        let mut config = SearchBotConfig::custom_depth(depth);
        config.time_budget_ms = time_budget_ms;
        config.cpu_time_budget_ms = cpu_time_budget_ms;
        return Some(config);
    }

    let alias = spec
        .strip_prefix("baseline-")
        .or_else(|| spec.strip_prefix("search-"))
        .unwrap_or(spec);

    lab_search_config(alias).map(|lab_config| {
        let mut config = lab_config.config;
        config.time_budget_ms = time_budget_ms.or(config.time_budget_ms);
        config.cpu_time_budget_ms = cpu_time_budget_ms.or(config.cpu_time_budget_ms);
        config
    })
}
