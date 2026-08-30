use serde_json::Value;

const KIND: &str = "tool_result";
const RAW_FORMAT: &str = "raw_following";
const LEGACY_PREFIX: &str = r#"{"kind":"tool_result""#;

pub(super) fn replace_output(rendered: &str, replacement: &str) -> String {
    if let Some((metadata, _)) = raw_parts(rendered) {
        return format!("{metadata}\n{replacement}");
    }
    if let Some(mut legacy) = legacy_value(rendered) {
        if is_legacy_envelope(&legacy) {
            legacy["output"] = Value::String(replacement.to_string());
            return serde_json::to_string(&legacy).unwrap_or_else(|_| replacement.to_string());
        }
    }
    replacement.to_string()
}

pub(super) fn output_starts_with(rendered: &str, prefix: &str) -> bool {
    if let Some((_, output)) = raw_parts(rendered) {
        return output.starts_with(prefix);
    }
    if let Some(value) = legacy_value(rendered) {
        if is_legacy_envelope(&value) {
            return value["output"]
                .as_str()
                .is_some_and(|output| output.starts_with(prefix));
        }
    }
    rendered.starts_with(prefix)
}

fn raw_parts(rendered: &str) -> Option<(&str, &str)> {
    let (metadata, output) = rendered.split_once('\n')?;
    let value: Value = serde_json::from_str(metadata).ok()?;
    (is_tool_metadata(&value) && value["outputFormat"] == RAW_FORMAT).then_some((metadata, output))
}

fn is_legacy_envelope(value: &Value) -> bool {
    is_tool_metadata(value) && value.get("output").is_some_and(Value::is_string)
}

fn is_tool_metadata(value: &Value) -> bool {
    value["kind"] == KIND
        && value.get("tool").is_some_and(Value::is_string)
        && value["status"].as_str().is_some_and(|status| {
            matches!(
                status,
                "success" | "running" | "partial" | "error" | "cancelled" | "stopped"
            )
        })
}

pub(crate) fn rendered_status_is_error(rendered: &str) -> bool {
    rendered
        .lines()
        .next()
        .and_then(|line| serde_json::from_str::<Value>(line).ok())
        .is_some_and(|value| {
            is_tool_metadata(&value)
                && value["status"]
                    .as_str()
                    .is_some_and(|status| matches!(status, "error" | "cancelled"))
        })
}

fn legacy_value(rendered: &str) -> Option<Value> {
    if !rendered.starts_with(LEGACY_PREFIX) {
        return None;
    }
    serde_json::from_str(rendered).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_metadata_is_preserved_when_output_is_replaced() {
        let rendered = concat!(
            r#"{"kind":"tool_result","tool":"bash","status":"error","outputFormat":"raw_following","error":{"code":"failed"}}"#,
            "\nlarge output"
        );
        let compacted = replace_output(rendered, "omitted");

        assert!(compacted.contains(r#""code":"failed""#));
        assert!(compacted.ends_with("\nomitted"));
        assert!(output_starts_with(&compacted, "omitted"));
    }

    #[test]
    fn legacy_json_keeps_error_metadata() {
        let rendered = r#"{"kind":"tool_result","tool":"bash","status":"error","output":"large","error":{"code":"failed"}}"#;
        let compacted = replace_output(rendered, "omitted");
        let value: Value = serde_json::from_str(&compacted).unwrap();

        assert_eq!(value["status"], "error");
        assert_eq!(value["error"]["code"], "failed");
        assert_eq!(value["output"], "omitted");
    }

    #[test]
    fn ordinary_json_cannot_impersonate_tool_metadata() {
        let ordinary = r#"{"kind":"tool_result","output":"omitted"}"#;

        assert!(!output_starts_with(ordinary, "omitted"));
    }
}
