const MAX_CONSECUTIVE_IDENTICAL: usize = 6;

pub struct CircuitBreaker {
    last_signature: Option<String>,
    consecutive_count: usize,
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            last_signature: None,
            consecutive_count: 0,
        }
    }

    pub fn check(
        &mut self,
        tool_calls: &[(String, serde_json::Value)],
        owner_session_id: &str,
    ) -> Result<(), String> {
        let active_calls = tool_calls
            .iter()
            .filter(|(name, args)| !is_passive_bash_control(name, args, owner_session_id))
            .cloned()
            .collect::<Vec<_>>();
        if active_calls.is_empty() {
            return Ok(());
        }
        let sig = compute_signature(&active_calls);
        let is_repeat = self.last_signature.as_ref() == Some(&sig);
        if is_repeat {
            self.consecutive_count += 1;
            if self.consecutive_count >= MAX_CONSECUTIVE_IDENTICAL {
                return Err("circuit_breaker".to_string());
            }
        } else {
            self.last_signature = Some(sig);
            self.consecutive_count = 1;
        }
        Ok(())
    }
}

fn is_passive_bash_control(name: &str, args: &serde_json::Value, owner_session_id: &str) -> bool {
    const ALLOWED_FIELDS: [&str; 6] = [
        "session_id",
        "chars",
        "eof",
        "stop",
        "yield_time_ms",
        "yield-time-ms",
    ];

    if name != "bash_control" {
        return false;
    }
    let Some(args) = args.as_object() else {
        return false;
    };
    if args
        .keys()
        .any(|key| !ALLOWED_FIELDS.contains(&key.as_str()))
    {
        return false;
    }
    let Some(process_id) = args.get("session_id").and_then(serde_json::Value::as_str) else {
        return false;
    };
    if args
        .get("chars")
        .is_some_and(|value| value.as_str() != Some(""))
        || args
            .get("eof")
            .is_some_and(|value| value.as_bool() != Some(false))
        || args
            .get("stop")
            .is_some_and(|value| value.as_bool() != Some(false))
        || args
            .get("yield_time_ms")
            .is_some_and(|value| value.as_u64().is_none())
        || args
            .get("yield-time-ms")
            .is_some_and(|value| value.as_u64().is_none())
    {
        return false;
    }
    super::tool_bash_registry::contains_owned(process_id, owner_session_id)
}

fn normalize_json(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => {
            let mut pairs: Vec<_> = map.iter().collect();
            pairs.sort_by_key(|(k, _)| k.as_str());
            let entries: Vec<String> = pairs
                .into_iter()
                .map(|(k, v)| format!("{}:{}", k, normalize_json(v)))
                .collect();
            format!("{{{}}}", entries.join(","))
        }
        serde_json::Value::Array(arr) => {
            let entries: Vec<String> = arr.iter().map(normalize_json).collect();
            format!("[{}]", entries.join(","))
        }
        _ => value.to_string(),
    }
}

fn compute_signature(tool_calls: &[(String, serde_json::Value)]) -> String {
    let mut parts = Vec::with_capacity(tool_calls.len());
    for (name, args) in tool_calls {
        parts.push(format!("{}|{}", name, normalize_json(args)));
    }
    parts.join("\n")
}
