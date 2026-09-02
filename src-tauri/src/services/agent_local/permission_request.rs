use crate::services::extensions::ExtensionEffect;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionPermissionDisplay {
    pub extension_id: String,
    pub extension_name: String,
    #[serde(rename = "effectClass")]
    pub effect: ExtensionEffect,
    pub action_summary: String,
    pub allow_session: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRequest {
    pub id: String,
    pub tool_name: String,
    pub arguments: Value,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extension: Option<ExtensionPermissionDisplay>,
}

pub fn native(id: String, tool_name: &str, arguments: &Value) -> PermissionRequest {
    PermissionRequest {
        id,
        tool_name: tool_name.to_string(),
        arguments: arguments.clone(),
        extension: None,
    }
}

pub fn for_extension(
    id: String,
    extension_id: &str,
    extension_name: &str,
    tool_name: &str,
    effect: ExtensionEffect,
    arguments: &Value,
) -> PermissionRequest {
    let redacted = super::sensitive_data::redact_json(arguments);
    let serialized = serde_json::to_string(&redacted).unwrap_or_else(|_| "{}".to_string());
    let action_summary = serialized
        .chars()
        .take(crate::services::extensions::MAX_PERMISSION_SUMMARY_CHARS)
        .collect();
    PermissionRequest {
        id,
        tool_name: tool_name.to_string(),
        arguments: serde_json::json!({}),
        extension: Some(ExtensionPermissionDisplay {
            extension_id: extension_id.to_string(),
            extension_name: extension_name.to_string(),
            effect,
            action_summary,
            allow_session: super::permission_policy::extension_effect_policy(effect)
                .allow_session_cache,
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::services::extensions::ExtensionEffect;
    use serde_json::json;

    #[test]
    fn extension_request_hides_arguments_and_bounds_a_redacted_summary() {
        let sensitive_value = "sensitive-sentinel-value";
        let long = "é".repeat(600);
        let request = super::for_extension(
            "request-id".to_string(),
            "plugin-id",
            "Plugin",
            "plugin.tool",
            ExtensionEffect::Secret,
            &json!({"token": sensitive_value, "content": long}),
        );
        let value = serde_json::to_value(request).expect("serialize request");

        assert_eq!(value["arguments"], json!({}));
        assert_eq!(value["extensionId"], "plugin-id");
        assert_eq!(value["extensionName"], "Plugin");
        assert_eq!(value["effectClass"], "secret");
        assert_eq!(value["allowSession"], false);
        let summary = value["actionSummary"].as_str().expect("summary");
        assert!(!summary.contains(sensitive_value));
        assert!(
            summary.chars().count() <= crate::services::extensions::MAX_PERMISSION_SUMMARY_CHARS
        );
        assert!(summary.is_char_boundary(summary.len()));
    }

    #[test]
    fn cacheable_extension_effects_are_declared_to_the_frontend() {
        let request = super::for_extension(
            "request-id".to_string(),
            "plugin-id",
            "Plugin",
            "plugin.tool",
            ExtensionEffect::LocalWrite,
            &json!({}),
        );

        let value = serde_json::to_value(request).expect("serialize request");
        assert_eq!(value["allowSession"], true);
    }
}
