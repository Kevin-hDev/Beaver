use super::subagent_tool_profile::SubagentToolProfile;
use serde_json::Value;
use std::path::{Component, Path, PathBuf};

pub async fn validate_for_session(
    session_id: &str,
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
) -> Result<Option<SubagentToolProfile>, String> {
    let profile = profile_for_session(session_id).await?;
    let Some(profile) = profile else {
        return Ok(None);
    };
    let skills_enabled = super::agent_settings::is_tool_enabled("load_skill").await;
    validate_for_profile(profile, skills_enabled, tool_name, args, working_dir)?;
    Ok(Some(profile))
}

pub async fn profile_for_session(
    session_id: &str,
) -> Result<Option<SubagentToolProfile>, String> {
    let session = super::session_store::get(session_id)
        .await
        .map_err(|_| "Session d'outil indisponible.".to_string())?;
    let is_child = session.parent_session_id.is_some() || session.subagent_type.is_some();
    if !is_child {
        return Ok(None);
    }
    SubagentToolProfile::from_session_type(session.subagent_type.as_deref()).map(Some)
}

pub fn validate_for_profile(
    profile: SubagentToolProfile,
    skills_enabled: bool,
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
) -> Result<(), String> {
    if !profile.allows(tool_name, skills_enabled) {
        return Err("Outil indisponible pour ce sous-agent.".to_string());
    }
    if super::memory_tool::is_memory_operation(tool_name, args, Some(working_dir))? {
        return if tool_name == "read_file" {
            Ok(())
        } else {
            Err(
                "Les sous-agents peuvent seulement lire une mémoire sélectionnée et suggérer une modification au parent."
                    .to_string(),
            )
        };
    }
    match tool_name {
        "read_file" | "write_file" | "edit_file" | "list_dir" => {
            validate_path_argument(args.get("path"), working_dir)?;
        }
        "grep" | "glob" => {
            if let Some(path) = args.get("path") {
                validate_path_argument(Some(path), working_dir)?;
            }
        }
        "bash" => validate_bash(profile, args, working_dir)?,
        _ => {}
    }
    Ok(())
}

pub(super) fn validate_path_argument(
    value: Option<&Value>,
    working_dir: &Path,
) -> Result<(), String> {
    let path = value
        .and_then(Value::as_str)
        .ok_or_else(|| "Chemin invalide.".to_string())?;
    validate_confined_path(path, working_dir)
}

fn validate_confined_path(path: &str, working_dir: &Path) -> Result<(), String> {
    if path.is_empty() || path.contains('\0') {
        return Err("Chemin invalide.".to_string());
    }
    let candidate = Path::new(path);
    if candidate
        .components()
        .any(|component| component == Component::ParentDir)
    {
        return Err("Chemin hors du dossier autorisé.".to_string());
    }
    let root = working_dir
        .canonicalize()
        .map_err(|_| "Dossier de travail inaccessible.".to_string())?;
    let raw = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        root.join(candidate)
    };
    let resolved = canonical_existing_ancestor(&raw)?;
    if resolved.starts_with(&root) {
        Ok(())
    } else {
        Err("Chemin hors du dossier autorisé.".to_string())
    }
}

fn canonical_existing_ancestor(path: &Path) -> Result<PathBuf, String> {
    let mut current = path;
    while !current.exists() {
        current = current
            .parent()
            .ok_or_else(|| "Chemin invalide.".to_string())?;
    }
    current
        .canonicalize()
        .map_err(|_| "Chemin inaccessible.".to_string())
}

fn validate_bash(
    profile: SubagentToolProfile,
    args: &Value,
    working_dir: &Path,
) -> Result<(), String> {
    if let Some(workdir) = args.get("workdir") {
        validate_path_argument(Some(workdir), working_dir)?;
    }
    let command = args
        .get("command")
        .and_then(Value::as_str)
        .ok_or_else(|| "Commande invalide.".to_string())?;
    match profile {
        SubagentToolProfile::Explorer => super::subagent_explorer_bash::validate(command, working_dir),
        SubagentToolProfile::Coder => validate_coder_bash(command, working_dir),
    }
}

fn validate_coder_bash(command: &str, working_dir: &Path) -> Result<(), String> {
    if command.contains('\0') || command.lines().count() > 1 {
        return Err("Commande hors du worktree refusée.".to_string());
    }
    let tokens = command.split_whitespace().collect::<Vec<_>>();
    for (index, raw) in tokens.iter().enumerate() {
        let token = raw.trim_matches(['\'', '"', ';', '(', ')', ',']);
        let path_token = token
            .rsplit_once('>')
            .map_or(token, |(_, candidate)| candidate);
        if token == ".." || token.starts_with("../") || token.contains("/../") {
            return Err("Commande hors du worktree refusée.".to_string());
        }
        let is_system_executable = index == 0 && (token.starts_with("/bin/") || token.starts_with("/usr/bin/"));
        if Path::new(path_token).is_absolute() && !is_system_executable {
            validate_confined_path(path_token, working_dir)?;
        }
        if matches!(tokens.get(index.wrapping_sub(1)), Some(&"-C" | &"cd" | &"pushd")) {
            validate_confined_path(token, working_dir)?;
        }
    }
    Ok(())
}
