use super::*;

pub(super) fn detect_git_commit() -> Option<String> {
    if let Some(sha) = option_env!("GITHUB_SHA") {
        return Some(sha.chars().take(12).collect());
    }

    let output = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!value.is_empty()).then_some(value)
}

pub(super) fn detect_git_dirty() -> Option<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    Some(!output.stdout.is_empty())
}

pub(super) fn detect_generated_at_utc() -> String {
    if let Ok(output) = Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
    {
        if output.status.success() {
            if let Ok(value) = String::from_utf8(output.stdout) {
                let value = value.trim().to_string();
                if !value.is_empty() {
                    return value;
                }
            }
        }
    }

    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    format!("unix:{seconds}")
}

pub(super) fn detect_generated_at_local() -> String {
    if let Ok(output) = Command::new("date")
        .args(["+%Y-%m-%d %H:%M:%S %Z"])
        .output()
    {
        if output.status.success() {
            if let Ok(value) = String::from_utf8(output.stdout) {
                let value = value.trim().to_string();
                if !value.is_empty() {
                    return value;
                }
            }
        }
    }

    detect_generated_at_utc()
}

pub(super) fn detect_linux_cpu_info() -> (Option<String>, Option<f64>) {
    let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") else {
        return (None, None);
    };

    let mut model = None;
    let mut mhz = None;
    for line in cpuinfo.lines() {
        if model.is_none() {
            if let Some(value) = line.strip_prefix("model name") {
                model = cpuinfo_value(value);
            }
        }
        if mhz.is_none() {
            if let Some(value) = line.strip_prefix("cpu MHz") {
                mhz = cpuinfo_value(value).and_then(|value| value.parse::<f64>().ok());
            }
        }
        if model.is_some() && mhz.is_some() {
            break;
        }
    }

    (model, mhz)
}

pub(super) fn cpuinfo_value(input: &str) -> Option<String> {
    input
        .split_once(':')
        .map(|(_, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
