use std::path::{Path, PathBuf};

const MAX_TOTAL_BYTES: usize = 200 * 1024;
const LIMIT_NOTICE: &str =
    "Additional selected instructions were omitted because the context limit was reached.";

pub async fn load_agent_md(project_dir: Option<&Path>) -> Option<String> {
    let data_dir = crate::services::paths::data_dir();
    let external_rules = match dirs::home_dir() {
        Some(home) => tokio::task::spawn_blocking(move || {
            crate::services::agent_import::selected_rule_contents(&home)
        })
        .await
        .unwrap_or_default(),
        None => Vec::new(),
    };
    load_agent_md_with_rules(&data_dir, project_dir, external_rules).await
}

#[cfg(test)]
pub async fn load_agent_md_from(data_dir: &Path, project_dir: Option<&Path>) -> Option<String> {
    load_agent_md_with_rules(data_dir, project_dir, Vec::new()).await
}

async fn load_agent_md_with_rules(
    data_dir: &Path,
    project_dir: Option<&Path>,
    mut external_rules: Vec<crate::services::agent_import::ExternalRuleContent>,
) -> Option<String> {
    let mut sections: Vec<String> = Vec::new();
    let mut total_bytes: usize = 0;

    let global_path = data_dir.join("AGENTS.md");
    if let Ok(content) = tokio::fs::read_to_string(&global_path).await {
        let source = global_path.display().to_string();
        push_content(
            &source,
            "global instructions",
            &content,
            &mut sections,
            &mut total_bytes,
        );
    }
    for name in crate::services::agent_import::enabled_hidden_documents(data_dir) {
        push_file(
            &data_dir.join(&name),
            "imported global instructions",
            &mut sections,
            &mut total_bytes,
        )
        .await;
    }
    external_rules.sort_by(|left, right| {
        left.source_id.cmp(&right.source_id)
    });
    for rule in external_rules {
        let label = format!("external rule · {}", rule.source_name);
        push_content(
            "selected external rule",
            &label,
            &rule.content,
            &mut sections,
            &mut total_bytes,
        );
    }

    if let Some(dir) = project_dir {
        collect_project_entries(dir, &mut sections, &mut total_bytes).await;
    }

    if sections.is_empty() {
        return None;
    }

    let header = "The following global and project instructions define how you should behave. \
                  You MUST follow them exactly as written. More specific project instructions \
                  appear after global instructions.";

    Some(format!("{}\n\n{}", header, sections.join("\n\n")))
}

async fn collect_project_entries(dir: &Path, sections: &mut Vec<String>, total_bytes: &mut usize) {
    push_file(
        &dir.join("AGENTS.md"),
        "project instructions",
        sections,
        total_bytes,
    )
    .await;
    push_file(
        &dir.join(".cl-go").join("AGENTS.md"),
        "project instructions",
        sections,
        total_bytes,
    )
    .await;

    let rules_dir = dir.join(".cl-go").join("rules");
    if let Ok(mut entries) = tokio::fs::read_dir(&rules_dir).await {
        let mut rule_files: Vec<PathBuf> = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                rule_files.push(path);
            }
        }
        rule_files.sort();
        for rule_file in rule_files {
            push_file(&rule_file, "project rules", sections, total_bytes).await;
        }
    }
}

async fn push_file(path: &Path, label: &str, sections: &mut Vec<String>, total_bytes: &mut usize) {
    if *total_bytes >= MAX_TOTAL_BYTES {
        return;
    }
    if let Ok(content) = tokio::fs::read_to_string(path).await {
        let source = path.display().to_string();
        push_content(&source, label, &content, sections, total_bytes);
    }
}

fn push_content(
    source: &str,
    label: &str,
    content: &str,
    sections: &mut Vec<String>,
    total_bytes: &mut usize,
) {
    let trimmed = content.trim();
    if trimmed.is_empty() || *total_bytes >= MAX_TOTAL_BYTES {
        return;
    }
    let entry = format!("Contents of {source} ({label}):\n\n{trimmed}");
    if entry.len() > MAX_TOTAL_BYTES.saturating_sub(*total_bytes) {
        if LIMIT_NOTICE.len() <= MAX_TOTAL_BYTES.saturating_sub(*total_bytes) {
            sections.push(LIMIT_NOTICE.to_string());
            *total_bytes += LIMIT_NOTICE.len();
        } else {
            *total_bytes = MAX_TOTAL_BYTES;
        }
        return;
    }
    *total_bytes += entry.len();
    sections.push(entry);
}

#[cfg(test)]
#[path = "agent_md_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "agent_md_import_tests.rs"]
mod import_tests;
