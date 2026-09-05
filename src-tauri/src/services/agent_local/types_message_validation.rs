use super::types_message::AgentMessage;

pub(super) fn validate(message: &AgentMessage) -> Result<(), String> {
    let records = message.tool_activities.iter().flatten().chain(
        message
            .segments
            .iter()
            .flatten()
            .flat_map(|segment| &segment.tools),
    );
    for record in records {
        if record
            .domain
            .as_deref()
            .is_some_and(|domain| domain != "memory")
            || record
                .resolved_path
                .as_deref()
                .is_some_and(|path| path.is_empty() || path.len() > 4_096 || path.contains('\0'))
        {
            return Err("Historique d'outil invalide.".to_string());
        }
        if record.file_changes.len() > super::tool_file_changes::MAX_FILE_CHANGES {
            return Err("Historique de fichiers invalide.".to_string());
        }
        if super::tool_artifact_record::validate(&record.artifacts).is_err() {
            return Err("Historique d'artefacts invalide.".to_string());
        }
        if record.affected_paths.len() > super::types_tool_result_details::MAX_AFFECTED_PATHS
            || record
                .affected_paths
                .iter()
                .any(|path| path.is_empty() || path.len() > 4_096 || path.contains('\0'))
        {
            return Err("Historique de fichiers invalide.".to_string());
        }
        if record.result_meta.as_ref().is_some_and(|meta| {
            meta.warnings.len() > 16
                || meta.status.is_error() != record.is_error.unwrap_or(false)
                || meta.status.is_error() != meta.error.is_some()
                || meta
                    .warnings
                    .iter()
                    .any(|warning| !valid_meta_text(warning))
                || meta.error.as_ref().is_some_and(|error| {
                    error.code.is_empty()
                        || error.code.len() > 100
                        || !error.code.bytes().all(|byte| {
                            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
                        })
                        || error
                            .hint
                            .as_ref()
                            .is_some_and(|hint| !valid_meta_text(hint))
                })
        }) {
            return Err("Historique de résultat d'outil invalide.".to_string());
        }
        let mut total_diff_bytes = 0usize;
        for change in &record.file_changes {
            if let Some(diff) = &change.diff {
                total_diff_bytes = total_diff_bytes.saturating_add(
                    crate::services::git::diff_preview::preview_content_bytes(diff),
                );
            }
            if change.path.is_empty()
                || change.path.len() > 4_096
                || change.path.contains('\0')
                || change.additions > 2_000
                || change.deletions > 2_000
                || total_diff_bytes > super::tool_file_changes::MAX_FILE_CHANGE_DIFF_BYTES
                || change.diff.as_ref().is_some_and(|diff| {
                    !crate::services::git::diff_preview::is_bounded_preview(diff)
                })
            {
                return Err("Historique de fichiers invalide.".to_string());
            }
        }
    }
    Ok(())
}

fn valid_meta_text(text: &str) -> bool {
    super::tool_result_contract::is_safe_metadata_text(text, 1_000)
}
