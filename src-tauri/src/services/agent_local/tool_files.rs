use crate::services::agent_local::security;
use crate::services::agent_local::types_tools::ToolResult;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

const MAX_READ_SIZE: u64 = 20 * 1024 * 1024;
const MAX_LIST_ENTRIES: usize = 500;
pub const DEFAULT_LIMIT: usize = 2000;
const MAX_LIMIT: usize = 50_000;

pub(crate) fn resolve_read_path(path: &str, working_dir: &Path) -> Result<PathBuf, String> {
    let p = Path::new(path);
    let raw = if p.is_absolute() {
        p.to_path_buf()
    } else {
        working_dir.join(p)
    };
    security::validate_read_path(&raw, working_dir)
}

fn resolve_write_path(path: &str, working_dir: &Path) -> Result<PathBuf, String> {
    let p = Path::new(path);
    let raw = if p.is_absolute() {
        p.to_path_buf()
    } else {
        working_dir.join(p)
    };
    security::validate_write_path(&raw)
}

pub async fn read_file(path: &str, working_dir: &Path, offset: usize, limit: usize) -> ToolResult {
    let resolved = match resolve_read_path(path, working_dir) {
        Ok(p) => p,
        Err(e) => return ToolResult::err(e),
    };
    match tokio::fs::metadata(&resolved).await {
        Ok(meta) if meta.len() > MAX_READ_SIZE => {
            return ToolResult::err("Fichier trop volumineux (max 20MB)");
        }
        Err(e) => return ToolResult::err(security::sanitize_error(e)),
        _ => {}
    }
    let raw = match tokio::fs::read_to_string(&resolved).await {
        Ok(c) => c,
        Err(e) => return ToolResult::err(security::sanitize_error(e)),
    };
    let lines: Vec<&str> = raw.lines().collect();
    let total = lines.len();
    let start = offset.min(total);
    let limit = limit.min(MAX_LIMIT);
    let end = start.saturating_add(limit).min(total);
    let slice = &lines[start..end];
    let mut output = String::with_capacity(slice.len() * 80);
    for (i, line) in slice.iter().enumerate() {
        let line_num = start + i + 1;
        output.push_str(&format!("{line_num}\t{line}\n"));
    }
    let remaining = total.saturating_sub(end);
    if remaining > 0 {
        output.push_str(&format!(
            "\n[{remaining} ligne(s) restante(s) — utilise offset={end} pour la suite]"
        ));
    }
    ToolResult::ok(output)
}

pub async fn write_file(path: &str, content: &str, working_dir: &Path) -> ToolResult {
    let raw = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        working_dir.join(path)
    };
    if matches!(
        tokio::fs::symlink_metadata(&raw).await,
        Ok(meta) if meta.file_type().is_symlink()
    ) {
        return ToolResult::err("Écriture sur symlink interdite");
    }
    let resolved = match resolve_write_path(path, working_dir) {
        Ok(p) => p,
        Err(e) => return ToolResult::err(e),
    };
    if let Some(parent) = resolved.parent() {
        if let Ok(real_parent) = parent.canonicalize() {
            let roots = security::allowed_write_roots();
            if !roots.iter().any(|r| real_parent.starts_with(r)) {
                return ToolResult::err("Écriture interdite hors des zones autorisées");
            }
        }
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            return ToolResult::err(security::sanitize_error(e));
        }
    }
    if resolved.is_symlink() {
        return ToolResult::err("Écriture sur symlink interdite");
    }
    match write_no_follow(&resolved, content).await {
        Ok(()) => ToolResult::ok(format!("Écrit: {}", resolved.display())),
        Err(e) => ToolResult::err(security::sanitize_error(e)),
    }
}

async fn write_no_follow(path: &Path, content: &str) -> std::io::Result<()> {
    let metadata = tokio::fs::symlink_metadata(path).await;
    if matches!(metadata, Ok(meta) if meta.file_type().is_symlink()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "symlink",
        ));
    }
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        options.custom_flags(libc::O_NOFOLLOW);
    }
    let mut file = options.open(path).await?;
    file.write_all(content.as_bytes()).await
}

pub async fn edit_file(
    path: &str,
    old_string: &str,
    new_string: &str,
    working_dir: &Path,
) -> ToolResult {
    let resolved = match resolve_write_path(path, working_dir) {
        Ok(p) => p,
        Err(e) => return ToolResult::err(e),
    };
    let content = match tokio::fs::read_to_string(&resolved).await {
        Ok(c) => c,
        Err(e) => return ToolResult::err(security::sanitize_error(e)),
    };
    let count = content.matches(old_string).count();
    if count == 0 {
        return ToolResult::err("Chaîne non trouvée");
    }
    if count > 1 {
        return ToolResult::err(format!("Chaîne trouvée {count} fois (doit être unique)"));
    }
    let start_line = content[..content.find(old_string).unwrap_or(0)]
        .chars()
        .filter(|c| *c == '\n')
        .count()
        + 1;
    let updated = content.replacen(old_string, new_string, 1);
    match tokio::fs::write(&resolved, &updated).await {
        Ok(()) => ToolResult::ok(format!(
            "Modifié: {} (ligne {})",
            resolved.display(),
            start_line
        ))
        .with_start_line(start_line),
        Err(e) => ToolResult::err(security::sanitize_error(e)),
    }
}

pub async fn list_dir(path: &str, working_dir: &Path) -> ToolResult {
    let resolved = match resolve_read_path(path, working_dir) {
        Ok(p) => p,
        Err(e) => return ToolResult::err(e),
    };
    let mut entries = Vec::new();
    let mut stack = vec![(resolved.clone(), 0u32)];

    while let Some((dir, depth)) = stack.pop() {
        if entries.len() >= MAX_LIST_ENTRIES {
            entries.push("... [tronqué]".to_string());
            break;
        }
        let mut read_dir = match tokio::fs::read_dir(&dir).await {
            Ok(r) => r,
            Err(e) => {
                entries.push(format!(
                    "... [erreur lecture dossier: {}]",
                    security::sanitize_error(e)
                ));
                continue;
            }
        };
        let mut children = Vec::new();
        while let Ok(Some(entry)) = read_dir.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }
            children.push(entry);
        }
        children.sort_by_key(|e| e.file_name());
        for entry in children {
            let name = entry.file_name().to_string_lossy().to_string();
            let indent = "  ".repeat(depth as usize);
            let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
            let suffix = if is_dir { "/" } else { "" };
            entries.push(format!("{indent}{name}{suffix}"));
            if is_dir && depth < 3 {
                stack.push((entry.path(), depth + 1));
            }
        }
    }
    ToolResult::ok(entries.join("\n"))
}
