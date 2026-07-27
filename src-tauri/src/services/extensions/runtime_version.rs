use super::types::MINIMUM_NODE_MAJOR;

pub fn validate_node(version: &str) -> Result<(), String> {
    let major = version
        .strip_prefix('v')
        .and_then(|value| value.split('.').next())
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| "Version Node.js incompatible.".to_string())?;
    if major < MINIMUM_NODE_MAJOR {
        return Err("Version Node.js incompatible.".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_node;

    #[test]
    fn rejects_old_or_invalid_node_versions() {
        assert!(validate_node("v20.0.0").is_ok());
        assert!(validate_node("v24.18.0").is_ok());
        assert!(validate_node("v18.20.0").is_err());
        assert!(validate_node("unknown").is_err());
    }
}
