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

    pub fn check(&mut self, tool_calls: &[(String, serde_json::Value)]) -> Result<(), String> {
        let sig = compute_signature(tool_calls);
        let is_repeat = self.last_signature.as_ref() == Some(&sig);
        if is_repeat {
            self.consecutive_count += 1;
            if self.consecutive_count >= MAX_CONSECUTIVE_IDENTICAL {
                return Err(format!(
                    "Circuit breaker : {} appels identiques consécutifs détectés. Boucle probable, arrêt.",
                    self.consecutive_count
                ));
            }
        } else {
            self.last_signature = Some(sig);
            self.consecutive_count = 1;
        }
        Ok(())
    }
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
