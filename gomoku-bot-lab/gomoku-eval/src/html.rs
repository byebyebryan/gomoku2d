pub(crate) fn escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub(crate) fn option_debug<T: std::fmt::Debug>(value: Option<T>) -> String {
    value
        .map(|value| format!("{value:?}"))
        .unwrap_or_else(|| "-".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_handles_special_chars() {
        assert_eq!(escape("<bot & 'x'>"), "&lt;bot &amp; &#39;x&#39;&gt;");
    }

    #[test]
    fn option_debug_formats_none_as_dash() {
        assert_eq!(option_debug(Some(7)), "7");
        assert_eq!(option_debug::<u8>(None), "-");
    }
}
