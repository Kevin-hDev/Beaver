use super::memory_format::ParsedTopic;
use super::memory_paths::MemoryScope;
use super::memory_types::MAX_SUMMARY_BYTES;

pub async fn rebuild(scope: &MemoryScope) -> Result<Vec<String>, String> {
    let mut topics = super::memory_store::list_topics(scope).await;
    topics.sort_by(|left, right| {
        right
            .topic
            .updated_at
            .cmp(&left.topic.updated_at)
            .then_with(|| left.topic.title.cmp(&right.topic.title))
    });
    super::memory_io::write_atomic(&scope.registry_path(), render_registry(&topics).as_bytes())
        .await?;
    super::memory_io::write_atomic(&scope.summary_path(), render_summary(&topics).as_bytes())
        .await?;
    Ok(vec![
        scope.registry_path().to_string_lossy().into_owned(),
        scope.summary_path().to_string_lossy().into_owned(),
    ])
}

fn render_registry(topics: &[ParsedTopic]) -> String {
    let mut output = String::from("# Registre mémoire\n\n");
    for parsed in topics {
        let topic = &parsed.topic;
        output.push_str(&format!(
            "- [{}](topics/{}.md) — `{}` · `{}` — {} — {}\n",
            topic.title, topic.id, topic.memory_type, topic.status, topic.summary, topic.updated_at
        ));
    }
    output
}

fn render_summary(topics: &[ParsedTopic]) -> String {
    let mut output = String::from("# Résumé mémoire\n\n");
    for parsed in topics {
        let topic = &parsed.topic;
        let line = format!(
            "- **{}** — {} — mots-clés: {} — détails: {}\n",
            topic.title,
            topic.summary,
            topic.tags.join(", "),
            topic.path
        );
        if output.len().saturating_add(line.len()) > MAX_SUMMARY_BYTES {
            break;
        }
        output.push_str(&line);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_stays_bounded() {
        let topics = (0..300)
            .map(|index| ParsedTopic {
                topic: super::super::memory_types::MemoryTopic {
                    id: uuid::Uuid::new_v4().to_string(),
                    title: format!("Sujet {index}"),
                    summary: "Résumé compact mais suffisamment descriptif.".repeat(4),
                    memory_type: "reference".into(),
                    status: "confirmed".into(),
                    tags: vec!["test".into()],
                    created_at: "2026-07-24T20:00:00+00:00".into(),
                    updated_at: "2026-07-24T20:00:00+00:00".into(),
                    source: "user".into(),
                    session_id: uuid::Uuid::new_v4().to_string(),
                    path: format!("/memory/topics/{index}.md"),
                },
            })
            .collect::<Vec<_>>();

        assert!(render_summary(&topics).len() <= MAX_SUMMARY_BYTES);
    }
}
