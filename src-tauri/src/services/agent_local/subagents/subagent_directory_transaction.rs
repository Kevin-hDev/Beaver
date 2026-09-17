use super::types_subagent_change::SubagentChangedPath;
use std::path::{Component, Path, PathBuf};

struct PlannedChange {
    target: PathBuf,
    source: Option<PathBuf>,
    backup: PathBuf,
}

struct AppliedChange {
    target: PathBuf,
    backup: Option<PathBuf>,
    installed: bool,
}

pub async fn apply(
    project: &Path,
    stage: &Path,
    changes: &[SubagentChangedPath],
) -> Result<(), String> {
    let project = project.to_path_buf();
    let stage = stage.to_path_buf();
    let changes = changes.to_vec();
    tokio::task::spawn_blocking(move || apply_sync(&project, &stage, &changes))
        .await
        .map_err(|_| generic_error())?
}

fn apply_sync(
    project: &Path,
    stage: &Path,
    changes: &[SubagentChangedPath],
) -> Result<(), String> {
    let project = project.canonicalize().map_err(|_| generic_error())?;
    let stage = stage.canonicalize().map_err(|_| generic_error())?;
    let transaction_id = uuid::Uuid::new_v4().to_string();
    let backup_root = project.join(format!(".cl-go-transaction-{transaction_id}"));
    if backup_root.exists() {
        return Err(generic_error());
    }
    let planned = plan_changes(&project, &stage, &backup_root, changes)?;
    std::fs::create_dir(&backup_root).map_err(|_| generic_error())?;
    let mut applied = Vec::new();
    let mut created_dirs = Vec::new();
    for plan in planned {
        if let Err(error) = apply_one(&plan, &mut applied, &mut created_dirs) {
            return rollback(&backup_root, &mut applied, &mut created_dirs).and(Err(error));
        }
    }
    if std::fs::remove_dir_all(&backup_root).is_err() {
        return rollback(&backup_root, &mut applied, &mut created_dirs)
            .and(Err(generic_error()));
    }
    Ok(())
}

fn plan_changes(
    project: &Path,
    stage: &Path,
    backup_root: &Path,
    changes: &[SubagentChangedPath],
) -> Result<Vec<PlannedChange>, String> {
    let mut planned = Vec::with_capacity(changes.len());
    for changed in changes {
        let relative = safe_relative(&changed.path)?;
        let target = project.join(relative);
        validate_target(project, &target)?;
        let source = if changed.kind == "D" {
            None
        } else {
            let source = stage.join(relative);
            let source_metadata = std::fs::symlink_metadata(&source).map_err(|_| generic_error())?;
            if !source_metadata.is_file() || source_metadata.file_type().is_symlink() {
                return Err(generic_error());
            }
            let canonical = source.canonicalize().map_err(|_| generic_error())?;
            if !canonical.starts_with(stage) {
                return Err(generic_error());
            }
            Some(canonical)
        };
        planned.push(PlannedChange {
            target,
            source,
            backup: backup_root.join(relative),
        });
    }
    Ok(planned)
}

fn apply_one(
    plan: &PlannedChange,
    applied: &mut Vec<AppliedChange>,
    created_dirs: &mut Vec<PathBuf>,
) -> Result<(), String> {
    ensure_parent(plan.target.parent().ok_or_else(generic_error)?, created_dirs)?;
    let backup = if plan.target.exists() {
        let parent = plan.backup.parent().ok_or_else(generic_error)?;
        std::fs::create_dir_all(parent).map_err(|_| generic_error())?;
        std::fs::rename(&plan.target, &plan.backup).map_err(|_| generic_error())?;
        Some(plan.backup.clone())
    } else {
        None
    };
    applied.push(AppliedChange {
        target: plan.target.clone(),
        backup,
        installed: false,
    });
    let Some(source) = plan.source.as_deref() else { return Ok(()) };
    let tmp = plan.target.with_file_name(format!(".cl-go-{}.tmp", uuid::Uuid::new_v4()));
    if std::fs::copy(source, &tmp).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return Err(generic_error());
    }
    let permissions = std::fs::metadata(source).map_err(|_| generic_error())?.permissions();
    if std::fs::set_permissions(&tmp, permissions).is_err()
        || std::fs::rename(&tmp, &plan.target).is_err()
    {
        let _ = std::fs::remove_file(&tmp);
        return Err(generic_error());
    }
    if let Some(last) = applied.last_mut() {
        last.installed = true;
    }
    Ok(())
}

fn rollback(
    backup_root: &Path,
    applied: &mut [AppliedChange],
    created_dirs: &mut [PathBuf],
) -> Result<(), String> {
    let mut restored = true;
    for entry in applied.iter().rev() {
        if entry.installed && std::fs::remove_file(&entry.target).is_err() {
            restored = false;
        }
        if let Some(backup) = entry.backup.as_deref() {
            if std::fs::rename(backup, &entry.target).is_err() {
                restored = false;
            }
        }
    }
    for dir in created_dirs.iter().rev() {
        let _ = std::fs::remove_dir(dir);
    }
    let _ = std::fs::remove_dir_all(backup_root);
    restored.then_some(()).ok_or_else(|| "Restauration du dossier impossible".into())
}

fn ensure_parent(parent: &Path, created: &mut Vec<PathBuf>) -> Result<(), String> {
    let mut missing = Vec::new();
    let mut cursor = parent;
    while !cursor.exists() {
        missing.push(cursor.to_path_buf());
        cursor = cursor.parent().ok_or_else(generic_error)?;
    }
    std::fs::create_dir_all(parent).map_err(|_| generic_error())?;
    missing.reverse();
    created.extend(missing);
    Ok(())
}

fn validate_target(project: &Path, target: &Path) -> Result<(), String> {
    let mut cursor = project.to_path_buf();
    let relative = target.strip_prefix(project).map_err(|_| generic_error())?;
    for component in relative.components() {
        cursor.push(component);
        let Ok(metadata) = std::fs::symlink_metadata(&cursor) else { break };
        if metadata.file_type().is_symlink() || metadata.is_dir() && cursor == target {
            return Err(generic_error());
        }
    }
    Ok(())
}

fn safe_relative(value: &str) -> Result<&Path, String> {
    let path = Path::new(value);
    if path.is_absolute() || path.components().any(|part| matches!(part, Component::ParentDir)) {
        Err(generic_error())
    } else {
        Ok(path)
    }
}

fn generic_error() -> String {
    "Application du changement impossible".to_string()
}
