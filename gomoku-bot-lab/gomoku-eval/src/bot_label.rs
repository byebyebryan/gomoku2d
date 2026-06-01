pub fn compact_bot_label(bot: &str, budgeted_unqualified_search: bool) -> String {
    if bot == "random" {
        return "RandomBot".to_string();
    }

    let mut parts = bot.split('+');
    let Some(base) = parts.next() else {
        return bot.to_string();
    };
    let Some(depth) = searchbot_base_depth(base, budgeted_unqualified_search) else {
        return bot.to_string();
    };

    let mut label = format!("SearchBot_D{depth}");
    for feature in parts {
        label.push('+');
        label.push_str(&compact_searchbot_feature_label(feature));
    }
    label
}

pub fn compact_bot_label_parts(
    bot: &str,
    budgeted_unqualified_search: bool,
) -> (String, Option<String>) {
    let label = compact_bot_label(bot, budgeted_unqualified_search);
    let Some((primary, modifiers)) = label.split_once('+') else {
        return (label, None);
    };

    (
        primary.to_string(),
        Some(modifiers.split('+').collect::<Vec<_>>().join(" + ")),
    )
}

fn searchbot_base_depth(bot: &str, _budgeted_unqualified_search: bool) -> Option<i32> {
    match bot.strip_prefix("search-d") {
        Some(depth) => depth.parse::<i32>().ok(),
        None => None,
    }
}

fn compact_searchbot_feature_label(feature: &str) -> String {
    if let Some(cap) = feature.strip_prefix("tactical-cap-") {
        return format!("TCap{cap}");
    }
    if let Some(cap) = feature.strip_prefix("tactical-full-cap-") {
        return format!("TFullCap{cap}");
    }
    if let Some(cap) = feature.strip_prefix("child-cap-") {
        return format!("Cap{cap}");
    }
    if let Some(radius) = feature.strip_prefix("near-all-r") {
        return format!("NearR{radius}");
    }
    if let Some(rest) = feature.strip_prefix("near-self-r") {
        if let Some((self_radius, opponent_radius)) = rest.split_once("-opponent-r") {
            return format!("SelfR{self_radius}OppR{opponent_radius}");
        }
    }
    if feature.starts_with("corridor-proof-") {
        return "Corridor Proof".to_string();
    }

    match feature {
        "pattern-eval" => "Pattern".to_string(),
        "rolling-frontier" => "Rolling".to_string(),
        "rolling-frontier-shadow" => "RollingShadow".to_string(),
        "tactical-full" => "TFull".to_string(),
        "no-safety" => "NoSafety".to_string(),
        _ => feature.to_string(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn labels_current_searchbot_specs_compactly() {
        assert_eq!(
            super::compact_bot_label(
                "search-d5+tactical-cap-16+pattern-eval+corridor-proof-c16-d8-w4",
                false,
            ),
            "SearchBot_D5+TCap16+Pattern+Corridor Proof"
        );
        assert_eq!(super::compact_bot_label("random", false), "RandomBot");
        assert_eq!(super::compact_bot_label("search-d3", false), "SearchBot_D3");
        assert_eq!(super::compact_bot_label("search", true), "search");
    }
}
