use super::limits::MAX_INSTRUCTION_BYTES;
use super::models::{DiscoveredItem, ImportItemKind};
use super::source_specs::source_specs;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(crate) struct ExternalRuleContent {
    pub source_id: String,
    pub source_name: String,
    pub content: String,
}

pub(crate) fn selected_rule_contents(home: &Path) -> Vec<ExternalRuleContent> {
    let allowed_roots = source_specs(home)
        .into_iter()
        .map(|spec| {
            let roots = spec
                .rule_roots
                .into_iter()
                .filter_map(|root| root.canonicalize().ok())
                .collect::<Vec<_>>();
            (spec.id, roots)
        })
        .collect::<BTreeMap<_, _>>();
    let mut rules = super::selected_rules(home);
    rules.sort_by(|left, right| {
        left.public
            .source_id
            .cmp(&right.public.source_id)
            .then(left.path.cmp(&right.path))
    });
    rules
        .into_iter()
        .filter_map(|rule| {
            let roots = allowed_roots.get(rule.public.source_id.as_str())?;
            read_rule_content(&rule, roots)
        })
        .collect()
}

fn read_rule_content(
    rule: &DiscoveredItem,
    allowed_roots: &[PathBuf],
) -> Option<ExternalRuleContent> {
    if rule.public.kind != ImportItemKind::Rule {
        return None;
    }
    let canonical = rule.path.canonicalize().ok()?;
    if !is_allowed(&canonical, allowed_roots) {
        return None;
    }
    let file = File::open(&canonical).ok()?;
    let metadata = file.metadata().ok()?;
    if !metadata.is_file() || metadata.len() > MAX_INSTRUCTION_BYTES {
        return None;
    }
    let verified = canonical.canonicalize().ok()?;
    if verified != canonical || !is_allowed(&verified, allowed_roots) {
        return None;
    }
    let mut content = String::new();
    file.take(MAX_INSTRUCTION_BYTES + 1)
        .read_to_string(&mut content)
        .ok()?;
    if content.len() as u64 > MAX_INSTRUCTION_BYTES {
        return None;
    }
    Some(ExternalRuleContent {
        source_id: rule.public.source_id.clone(),
        source_name: rule.public.source_name.clone(),
        content,
    })
}

fn is_allowed(path: &Path, roots: &[PathBuf]) -> bool {
    roots.iter().any(|root| path.starts_with(root))
}

#[cfg(test)]
#[path = "rule_content_tests.rs"]
mod tests;
