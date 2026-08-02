use std::path::{Path, PathBuf};

pub fn compact_tool_history(value: &mut serde_json::Value) {
    let working_dirs = value
        .get("working_dir")
        .and_then(serde_json::Value::as_str)
        .map(workspace_roots)
        .unwrap_or_default();
    let Some(messages) = value.get_mut("messages").and_then(serde_json::Value::as_array_mut)
    else {
        return;
    };
    for message in messages {
        let Some(message) = message.as_object_mut() else {
            continue;
        };
        if let Some(activities) = message
            .get_mut("tool_activities")
            .and_then(serde_json::Value::as_array_mut)
        {
            compact_records(activities, &working_dirs);
        }
        if let Some(segments) = message
            .get_mut("segments")
            .and_then(serde_json::Value::as_array_mut)
        {
            for tools in segments.iter_mut().filter_map(|segment| {
                segment
                    .get_mut("tools")
                    .and_then(serde_json::Value::as_array_mut)
            }) {
                compact_records(tools, &working_dirs);
            }
        }
        remove_duplicate(message);
    }
}

fn compact_records(records: &mut [serde_json::Value], working_dirs: &[PathBuf]) {
    for record in records {
        let Some(record) = record.as_object_mut() else {
            continue;
        };
        if let Some(value) = record.get_mut("affected_paths") {
            if let Ok(mut paths) = serde_json::from_value::<Vec<String>>(value.clone()) {
                paths.retain(|path| keep_path(working_dirs, path));
                let (paths, _) = super::types_tool_result_details::bounded_affected_paths(paths);
                *value = serde_json::json!(paths);
            }
        }
        if let Some(value) = record.get_mut("file_changes") {
            if let Ok(mut changes) =
                serde_json::from_value::<Vec<super::types_tools::ToolFileChange>>(value.clone())
            {
                changes.retain(|change| keep_path(working_dirs, &change.path));
                let (changes, _) = super::tool_file_changes::bounded_sample(changes);
                *value = serde_json::json!(changes);
            }
        }
    }
}

fn keep_path(working_dirs: &[PathBuf], path: &str) -> bool {
    let path = Path::new(path);
    !working_dirs.iter().any(|root| {
        path.starts_with(root) && !super::tool_bash_change_hub::is_trackable(root, path)
    })
}

fn workspace_roots(path: &str) -> Vec<PathBuf> {
    let path = Path::new(path);
    if !path.is_absolute() || !path.is_dir() {
        return Vec::new();
    }
    let mut roots = vec![path.to_path_buf()];
    if let Ok(canonical) = dunce::canonicalize(path) {
        if canonical != path {
            roots.push(canonical);
        }
    }
    roots
}

fn remove_duplicate(message: &mut serde_json::Map<String, serde_json::Value>) {
    let Some(activities) = message.get("tool_activities") else {
        return;
    };
    let Some(segments) = message.get("segments").and_then(serde_json::Value::as_array) else {
        return;
    };
    let segmented = segments
        .iter()
        .filter_map(|segment| segment.get("tools").and_then(serde_json::Value::as_array))
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    if !segmented.is_empty() && activities.as_array() == Some(&segmented) {
        message.remove("tool_activities");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_only_an_exact_segment_duplicate() {
        let tool = serde_json::json!({"name": "bash", "summary": "pwd"});
        let mut session = serde_json::json!({
            "messages": [{
                "tool_activities": [tool.clone()],
                "segments": [{"tools": [tool], "content": ""}]
            }]
        });

        compact_tool_history(&mut session);

        assert!(session["messages"][0].get("tool_activities").is_none());
        assert_eq!(session["messages"][0]["segments"][0]["tools"][0]["name"], "bash");
    }

    #[test]
    fn preserves_legacy_or_non_equivalent_activity_history() {
        let mut session = serde_json::json!({
            "messages": [
                {"tool_activities": [{"name": "bash"}]},
                {
                    "tool_activities": [{"name": "bash"}],
                    "segments": [{"tools": [{"name": "grep"}], "content": ""}]
                }
            ]
        });

        compact_tool_history(&mut session);

        assert!(session["messages"][0].get("tool_activities").is_some());
        assert!(session["messages"][1].get("tool_activities").is_some());
    }

    #[test]
    fn compacts_generated_and_unbounded_legacy_change_details() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        std::fs::create_dir_all(root.join("node_modules/pkg")).expect("dependencies");
        let generated = root.join("node_modules/pkg/index.js");
        let changes = std::iter::once(generated.clone())
            .chain((0..200).map(|index| root.join(format!("src/file-{index}.rs"))))
            .map(|path| {
                serde_json::json!({
                    "path": path,
                    "status": "added",
                    "additions": 1,
                    "deletions": 0
                })
            })
            .collect::<Vec<_>>();
        let affected = changes
            .iter()
            .filter_map(|change| change["path"].as_str())
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let mut session = serde_json::json!({
            "working_dir": root,
            "messages": [{
                "tool_activities": [{
                    "name": "bash",
                    "affected_paths": affected,
                    "file_changes": changes
                }]
            }]
        });

        compact_tool_history(&mut session);

        let tool = &session["messages"][0]["tool_activities"][0];
        let paths = tool["affected_paths"].as_array().expect("affected paths");
        let changes = tool["file_changes"].as_array().expect("file changes");
        assert_eq!(paths.len(), 128);
        assert_eq!(changes.len(), 128);
        assert!(paths.iter().all(|path| path.as_str() != generated.to_str()));
        assert!(changes
            .iter()
            .all(|change| change["path"].as_str() != generated.to_str()));
    }
}
