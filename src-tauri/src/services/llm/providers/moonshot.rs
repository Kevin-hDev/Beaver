pub fn is_forced_thinking(model: &str) -> bool {
    is_k3(model) || model.starts_with("kimi-k2.7-code") || model.starts_with("kimi-for-coding")
}

pub fn is_k3(model: &str) -> bool {
    model == "k3" || model.starts_with("k3-") || model.starts_with("kimi-k3")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_families_are_exact() {
        assert!(is_k3("k3"));
        assert!(is_k3("k3-256k"));
        assert!(is_forced_thinking("kimi-for-coding"));
        assert!(!is_forced_thinking("kimi-k2.6"));
    }
}
