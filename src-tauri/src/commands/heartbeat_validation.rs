use crate::models::{ScheduledWakeup, WakeupSchedule};
use chrono::{Local, NaiveDateTime, NaiveTime, TimeZone};

pub const MAX_WAKEUPS: usize = 64;
const MAX_NAME_LEN: usize = 120;
const MAX_MODEL_LEN: usize = 160;
const MAX_PROVIDER_LEN: usize = 64;
const MAX_PROMPT_LEN: usize = 12_000;
const MAX_DESCRIPTION_LEN: usize = 300;

const ALLOWED_PROVIDERS: &[&str] = &[
    "ollama",
    "groq",
    "google",
    "mistral",
    "cerebras",
    "openrouter",
    "openai",
    "deepseek",
    "xai",
    "moonshot",
    "zai",
    "codex-oauth",
];

pub fn validate_capacity(current_len: usize) -> Result<(), String> {
    if current_len >= MAX_WAKEUPS {
        return Err(format!("Maximum {} réveils", MAX_WAKEUPS));
    }
    Ok(())
}

pub fn validate_input(
    provider: &str,
    name: &str,
    model: &str,
    prompt: &str,
    description: &str,
    schedule: &WakeupSchedule,
    active: bool,
) -> Result<(), String> {
    validate_provider(provider)?;
    validate_text(name, "name", MAX_NAME_LEN)?;
    validate_text(model, "model", MAX_MODEL_LEN)?;
    validate_text(provider, "provider", MAX_PROVIDER_LEN)?;
    validate_text(prompt, "prompt", MAX_PROMPT_LEN)?;
    validate_optional_text(description, "description", MAX_DESCRIPTION_LEN)?;
    validate_schedule(schedule, active)
}

pub fn validate_wakeup(wakeup: &ScheduledWakeup) -> Result<(), String> {
    validate_input(
        &wakeup.provider,
        &wakeup.name,
        &wakeup.model,
        &wakeup.prompt,
        &wakeup.description,
        &wakeup.schedule,
        wakeup.active,
    )
}

fn validate_provider(provider: &str) -> Result<(), String> {
    if ALLOWED_PROVIDERS.contains(&provider) {
        Ok(())
    } else {
        Err("Provider non supporté".into())
    }
}

fn validate_text(value: &str, field: &str, max_len: usize) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("Champ {} requis", field));
    }
    if trimmed.chars().count() > max_len {
        return Err(format!("Champ {} trop long", field));
    }
    Ok(())
}

fn validate_optional_text(value: &str, field: &str, max_len: usize) -> Result<(), String> {
    if value.chars().count() > max_len {
        return Err(format!("Champ {} trop long", field));
    }
    Ok(())
}

fn validate_schedule(schedule: &WakeupSchedule, active: bool) -> Result<(), String> {
    match schedule {
        WakeupSchedule::Once { datetime } => {
            let dt = parse_local_datetime(datetime)?;
            if active && dt <= Local::now() {
                return Err("Date ponctuelle déjà passée".into());
            }
        }
        WakeupSchedule::Daily { time } => {
            parse_time(time)?;
        }
        WakeupSchedule::Weekly { weekday, time } => {
            if *weekday > 6 {
                return Err("Jour invalide".into());
            }
            parse_time(time)?;
        }
    }
    Ok(())
}

fn parse_time(value: &str) -> Result<NaiveTime, String> {
    NaiveTime::parse_from_str(value, "%H:%M").map_err(|_| "Heure invalide".to_string())
}

fn parse_local_datetime(value: &str) -> Result<chrono::DateTime<Local>, String> {
    let naive = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M")
        .map_err(|_| "Datetime invalide".to_string())?;
    Local
        .from_local_datetime(&naive)
        .single()
        .ok_or_else(|| "Datetime invalide".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_time() {
        let schedule = WakeupSchedule::Daily {
            time: "99:99".into(),
        };
        assert!(validate_schedule(&schedule, true).is_err());
    }

    #[test]
    fn rejects_invalid_weekday() {
        let schedule = WakeupSchedule::Weekly {
            weekday: 9,
            time: "08:00".into(),
        };
        assert!(validate_schedule(&schedule, true).is_err());
    }

    #[test]
    fn rejects_too_many_wakeups() {
        assert!(validate_capacity(MAX_WAKEUPS).is_err());
    }

    #[test]
    fn accepts_codex_provider() {
        let schedule = WakeupSchedule::Daily {
            time: "08:00".into(),
        };
        assert!(validate_input(
            "codex-oauth",
            "Test",
            "gpt-5.4",
            "Ping",
            "",
            &schedule,
            true,
        )
        .is_ok());
    }

    #[test]
    fn rejects_interactive_only_oauth_providers() {
        let schedule = WakeupSchedule::Daily {
            time: "08:00".into(),
        };
        for provider in ["xai-oauth", "moonshot-oauth"] {
            assert!(
                validate_input(provider, "Test", "model", "Ping", "", &schedule, true).is_err()
            );
        }
    }

    #[test]
    fn rejects_too_long_prompt() {
        let schedule = WakeupSchedule::Daily {
            time: "08:00".into(),
        };
        let prompt = "a".repeat(MAX_PROMPT_LEN + 1);
        assert!(validate_input("ollama", "Test", "llama", &prompt, "", &schedule, true).is_err());
    }
}
