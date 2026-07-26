use super::limits::MAX_ROOTS_PER_SOURCE;
use super::source_paths::{env_root, openclaw_workspaces};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DocumentCandidate {
    pub name: &'static str,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SourceSpec {
    pub id: &'static str,
    pub display_name: &'static str,
    pub detection_roots: Vec<PathBuf>,
    pub documents: Vec<DocumentCandidate>,
    pub rule_roots: Vec<PathBuf>,
    pub skill_roots: Vec<PathBuf>,
}

pub fn source_specs(home: &Path) -> Vec<SourceSpec> {
    let xdg = env_root("XDG_CONFIG_HOME").unwrap_or_else(|| home.join(".config"));
    let kimi = env_root("KIMI_CODE_HOME").unwrap_or_else(|| home.join(".kimi-code"));
    source_specs_with(home, &xdg, &kimi)
}

pub(crate) fn source_specs_with(home: &Path, xdg: &Path, kimi: &Path) -> Vec<SourceSpec> {
    let claude = home.join(".claude");
    let codex = home.join(".codex");
    let agents = home.join(".agents");
    let hermes = home.join(".hermes");
    let qwen = home.join(".qwen");
    let zcode = home.join(".zcode");
    let openclaw = home.join(".openclaw");
    let opencode = xdg.join("opencode");
    let openclaw_workspaces = openclaw_workspaces(&openclaw, home);
    let openclaw_document_root = openclaw_workspaces
        .iter()
        .find(|workspace| workspace.join("AGENTS.md").is_file())
        .unwrap_or(&openclaw_workspaces[0]);
    let mut openclaw_skill_roots = vec![openclaw.join("skills")];
    let mut openclaw_rule_roots = Vec::new();
    for workspace in &openclaw_workspaces {
        openclaw_rule_roots.push(workspace.join("rules"));
        openclaw_skill_roots.push(workspace.join("skills"));
        openclaw_skill_roots.push(workspace.join(".agents/skills"));
    }

    let specs = vec![
        spec("claude", "Claude Code", &claude, "CLAUDE.md"),
        spec("codex", "Codex", &codex, "AGENTS.md"),
        spec("agents", "Agents", &agents, "AGENTS.md"),
        SourceSpec {
            id: "hermes",
            display_name: "Hermes Agent",
            detection_roots: vec![hermes.clone()],
            documents: vec![],
            rule_roots: vec![hermes.join("rules")],
            skill_roots: vec![hermes.join("skills")],
        },
        SourceSpec {
            id: "qwen",
            display_name: "Qwen Code",
            detection_roots: vec![qwen.clone()],
            documents: vec![document(&qwen, "QWEN.md")],
            rule_roots: vec![qwen.join("rules"), qwen.join("output-language.md")],
            skill_roots: vec![qwen.join("skills")],
        },
        spec("zcode", "ZCode", &zcode, "AGENTS.md"),
        SourceSpec {
            id: "openclaw",
            display_name: "OpenClaw",
            detection_roots: std::iter::once(openclaw.clone())
                .chain(openclaw_workspaces.iter().cloned())
                .collect(),
            documents: vec![document(openclaw_document_root, "AGENTS.md")],
            rule_roots: openclaw_rule_roots,
            skill_roots: openclaw_skill_roots,
        },
        SourceSpec {
            id: "opencode",
            display_name: "OpenCode",
            detection_roots: vec![opencode.clone()],
            documents: vec![document(&opencode, "AGENTS.md")],
            rule_roots: vec![opencode.join("rules")],
            skill_roots: vec![opencode.join("skills")],
        },
        SourceSpec {
            id: "kimi",
            display_name: "Kimi Code",
            detection_roots: vec![
                kimi.to_path_buf(),
                home.join(".kimi"),
                home.join(".kimi-webbridge"),
                home.join(".kimi-work"),
            ],
            documents: vec![document(kimi, "AGENTS.md")],
            rule_roots: vec![kimi.join("rules")],
            skill_roots: vec![kimi.join("skills"), home.join(".kimi/skills")],
        },
    ];
    debug_assert!(specs.iter().all(|value| {
        value.skill_roots.len() <= MAX_ROOTS_PER_SOURCE
            && value.rule_roots.len() <= MAX_ROOTS_PER_SOURCE
            && value.detection_roots.len() <= MAX_ROOTS_PER_SOURCE
    }));
    specs
}

fn spec(
    id: &'static str,
    display_name: &'static str,
    root: &Path,
    document_name: &'static str,
) -> SourceSpec {
    SourceSpec {
        id,
        display_name,
        detection_roots: vec![root.to_path_buf()],
        documents: vec![document(root, document_name)],
        rule_roots: vec![root.join("rules")],
        skill_roots: vec![root.join("skills")],
    }
}

fn document(root: &Path, name: &'static str) -> DocumentCandidate {
    DocumentCandidate {
        name,
        path: root.join(name),
    }
}

#[cfg(test)]
#[path = "source_specs_tests.rs"]
mod tests;
