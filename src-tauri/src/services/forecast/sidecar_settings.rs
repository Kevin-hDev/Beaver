use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnloadPolicy {
    Never,
    After(Duration),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchSettings {
    pub device: String,
    pub keep_alive: String,
    pub unload_policy: UnloadPolicy,
}

pub fn current() -> LaunchSettings {
    let advanced = crate::services::config::read_config()
        .map(|config| config.advanced)
        .unwrap_or_default();
    let keep_alive = sanitize_keep_alive(&advanced.keep_alive);
    LaunchSettings {
        device: sanitize_device(&advanced.hardware_accel),
        unload_policy: parse_unload_policy(&keep_alive),
        keep_alive,
    }
}

impl LaunchSettings {
    pub fn env_vars(&self) -> [(String, String); 2] {
        [
            ("CLGO_FORECAST_DEVICE".to_string(), self.device.clone()),
            (
                "CLGO_FORECAST_KEEP_ALIVE".to_string(),
                self.keep_alive.clone(),
            ),
        ]
    }
}

fn sanitize_device(value: &str) -> String {
    if value == "cpu" { "cpu" } else { "gpu" }.to_string()
}

fn sanitize_keep_alive(value: &str) -> String {
    match value {
        "0" | "2m" | "5m" | "10m" | "15m" | "30m" | "forever" => value.to_string(),
        _ => "5m".to_string(),
    }
}

fn parse_unload_policy(value: &str) -> UnloadPolicy {
    if value == "forever" {
        return UnloadPolicy::Never;
    }
    if value == "0" {
        return UnloadPolicy::After(Duration::from_secs(0));
    }
    let minutes = value
        .strip_suffix('m')
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(5);
    UnloadPolicy::After(Duration::from_secs(minutes.saturating_mul(60)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_forever() {
        assert!(matches!(
            parse_unload_policy("forever"),
            UnloadPolicy::Never
        ));
    }

    #[test]
    fn parses_minutes() {
        assert_eq!(
            parse_unload_policy("2m"),
            UnloadPolicy::After(Duration::from_secs(120))
        );
    }
}
