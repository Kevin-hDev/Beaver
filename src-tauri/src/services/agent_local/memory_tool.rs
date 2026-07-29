use super::memory_paths::{lexical_path, path_arg, validate_in_scope, MemoryLayout, MemoryScope};
use super::memory_path_classification::classify_memory_path;
use super::types_tools::ToolResult;
use serde_json::Value;
use std::path::Path;

pub async fn dispatch_if_memory(
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
    session_id: &str,
) -> Option<ToolResult> {
    let raw_path = path_arg(tool_name, args)?;
    let layout = MemoryLayout::production();
    dispatch_with_layout(
        tool_name,
        args,
        working_dir,
        session_id,
        raw_path,
        &layout,
    )
    .await
}

async fn dispatch_with_layout(
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
    session_id: &str,
    raw_path: &str,
    layout: &MemoryLayout,
) -> Option<ToolResult> {
    let scope = match layout.scope_for_tool_path(raw_path, working_dir).await {
        Ok(Some(scope)) => scope,
        Ok(None) => return None,
        Err(error) => return Some(ToolResult::err(error)),
    };
    if !super::memory_runtime::read_allowed(session_id) {
        return Some(ToolResult::err("La mémoire est désactivée pour cette requête."));
    }
    let write = matches!(tool_name, "write_file" | "edit_file");
    if write && !super::memory_runtime::write_allowed(session_id) {
        return Some(ToolResult::err(
            "Une demande explicite est nécessaire pour modifier la mémoire.",
        ));
    }
    if !write && !scope.root.exists() {
        return Some(bound_result(
            session_id,
            ToolResult::ok("Aucune mémoire enregistrée dans cette portée."),
        ));
    }
    if write && scope.ensure().await.is_err() {
        return Some(ToolResult::err("Mémoire indisponible."));
    }
    let candidate = match lexical_path(raw_path, working_dir)
        .and_then(|path| validate_in_scope(&scope, &path))
    {
        Ok(path) => path,
        Err(error) => return Some(ToolResult::err(error)),
    };
    let result = dispatch_memory_tool(tool_name, args, working_dir, &scope, &candidate).await;
    Some(bound_result(session_id, result))
}

pub fn is_memory_operation(
    tool_name: &str,
    args: &Value,
    working_dir: Option<&Path>,
) -> bool {
    let Some(raw_path) = path_arg(tool_name, args) else {
        return false;
    };
    let root = crate::services::paths::data_dir().join("memory");
    classify_memory_path(raw_path, working_dir, &root)
        .map(|classification| classification.is_some())
        .unwrap_or(true)
}

pub fn write_authorization(
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
    session_id: &str,
) -> Option<bool> {
    if !matches!(tool_name, "write_file" | "edit_file")
        || !is_memory_operation(tool_name, args, Some(working_dir))
    {
        return None;
    }
    Some(super::memory_runtime::write_allowed(session_id))
}

pub fn event_domain(tool_name: &str, args: &Value) -> Option<String> {
    is_memory_operation(tool_name, args, None).then(|| "memory".to_string())
}

pub fn resolved_path_domain(path: Option<&str>) -> Option<String> {
    let root = crate::services::paths::data_dir().join("memory");
    classify_memory_path(path?, None, &root)
        .ok()
        .flatten()
        .map(|_| "memory".to_string())
}

async fn dispatch_memory_tool(
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
    scope: &MemoryScope,
    path: &Path,
) -> ToolResult {
    match tool_name {
        "read_file" if readable_file(scope, path) => {
            let offset = args["offset"].as_u64().unwrap_or(0) as usize;
            let limit = args["limit"]
                .as_u64()
                .unwrap_or(super::tool_files::DEFAULT_LIMIT as u64) as usize;
            super::tool_files::read_file(path.to_string_lossy().as_ref(), working_dir, offset, limit)
                .await
        }
        "grep" if searchable_path(scope, path) => {
            super::tool_grep::grep(
                args["pattern"].as_str().unwrap_or(""),
                Some(path.to_string_lossy().as_ref()),
                args["glob"].as_str(),
                working_dir,
            )
            .await
        }
        "glob" if path == scope.topics_dir() => {
            super::tool_glob::glob_files(
                args["pattern"].as_str().unwrap_or(""),
                Some(path.to_string_lossy().as_ref()),
                working_dir,
            )
            .await
        }
        "list_dir" if path == scope.topics_dir() => {
            super::tool_files::list_dir(path.to_string_lossy().as_ref(), working_dir).await
        }
        "write_file" => write(scope, path, args).await,
        "edit_file" => edit(scope, path, args).await,
        _ => ToolResult::err("Opération mémoire non autorisée."),
    }
}

async fn write(scope: &MemoryScope, path: &Path, args: &Value) -> ToolResult {
    let content = args["content"].as_str().unwrap_or("");
    match super::memory_store::write_topic(scope, path, content).await {
        Ok(_) => ToolResult::ok("Mémoire mise à jour."),
        Err(error) => ToolResult::err(error),
    }
}

async fn edit(scope: &MemoryScope, path: &Path, args: &Value) -> ToolResult {
    let old = args["old_string"].as_str().unwrap_or("");
    let new = args["new_string"].as_str().unwrap_or("");
    match super::memory_store::edit_topic(scope, path, old, new).await {
        Ok(_) => ToolResult::ok("Mémoire mise à jour."),
        Err(error) => ToolResult::err(error),
    }
}

fn readable_file(scope: &MemoryScope, path: &Path) -> bool {
    path == scope.registry_path()
        || path == scope.summary_path()
        || (path.parent() == Some(scope.topics_dir().as_path())
            && path.extension().and_then(|value| value.to_str()) == Some("md"))
}

fn searchable_path(scope: &MemoryScope, path: &Path) -> bool {
    path == scope.registry_path()
        || path == scope.summary_path()
        || path == scope.topics_dir()
        || readable_file(scope, path)
}

fn bound_result(session_id: &str, mut result: ToolResult) -> ToolResult {
    if result.is_error {
        return result;
    }
    let (content, truncated) =
        super::memory_runtime::consume_result(session_id, &result.content);
    result.content = content;
    result.truncated |= truncated;
    result
}

#[cfg(test)]
#[path = "memory_tool_tests.rs"]
mod tests;
