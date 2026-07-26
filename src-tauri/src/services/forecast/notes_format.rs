use super::notes_types::ForecastNote;
use super::notes_validation;

pub(super) fn serialize(note: &ForecastNote) -> String {
    format!(
        "---\nid: \"{}\"\nanalysis_id: \"{}\"\ndate: \"{}\"\ntype: \"{}\"\nsource: \"{}\"\ncreated_at: \"{}\"\nupdated_at: \"{}\"\n---\n\n# {}\n\n{}\n",
        note.id,
        note.analysis_id,
        note.date,
        note.note_type,
        note.source,
        note.created_at,
        note.updated_at,
        note.title,
        note.content
    )
}

pub(super) fn parse(
    raw: &str,
    expected_analysis_id: &str,
    expected_note_id: &str,
) -> Result<ForecastNote, String> {
    let normalized = raw.replace("\r\n", "\n");
    let rest = normalized.strip_prefix("---\n").ok_or_else(invalid_note)?;
    let (meta, body) = rest.split_once("\n---\n").ok_or_else(invalid_note)?;
    let body = body.trim();
    let (title_line, content) = body.split_once('\n').unwrap_or((body, ""));
    let title = title_line
        .strip_prefix("# ")
        .map(str::trim)
        .ok_or_else(invalid_note)?;
    let id = meta_value(meta, "id").ok_or_else(invalid_note)?;
    let analysis_id = meta_value(meta, "analysis_id").ok_or_else(invalid_note)?;
    if id != expected_note_id || analysis_id != expected_analysis_id {
        return Err(invalid_note());
    }
    notes_validation::id(&id, "Note invalide")?;
    notes_validation::id(&analysis_id, "Note invalide")?;
    Ok(ForecastNote {
        id,
        analysis_id,
        date: notes_validation::field(&required_meta(meta, "date")?)?,
        title: notes_validation::title(title)?,
        note_type: notes_validation::note_type(
            &meta_value(meta, "type").unwrap_or_else(|| "context".into()),
        )?,
        source: notes_validation::source(
            &meta_value(meta, "source").unwrap_or_else(|| "user".into()),
        )?,
        content: notes_validation::content(content.trim())?,
        file_path: String::new(),
        created_at: notes_validation::metadata(&required_meta(meta, "created_at")?)?,
        updated_at: notes_validation::metadata(&required_meta(meta, "updated_at")?)?,
    })
}

fn required_meta(meta: &str, key: &str) -> Result<String, String> {
    meta_value(meta, key).ok_or_else(invalid_note)
}

fn meta_value(meta: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    meta.lines()
        .find_map(|line| line.trim().strip_prefix(&prefix).map(clean_meta_value))
}

fn clean_meta_value(value: &str) -> String {
    value.trim().trim_matches('"').to_string()
}

fn invalid_note() -> String {
    "Note invalide".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ANALYSIS_ID: &str = "550e8400-e29b-41d4-a716-446655440000";
    const NOTE_ID: &str = "550e8400-e29b-41d4-a716-446655440001";

    #[test]
    fn malformed_front_matter_never_panics() {
        assert!(parse("\n---\n", ANALYSIS_ID, NOTE_ID).is_err());
        assert!(parse("---\n---\n", ANALYSIS_ID, NOTE_ID).is_err());
        assert!(parse("---\nid: \"x\"\n---\nbody", ANALYSIS_ID, NOTE_ID).is_err());
        assert!(parse("", ANALYSIS_ID, NOTE_ID).is_err());
    }

    #[test]
    fn rejects_metadata_that_does_not_match_the_requested_path() {
        let note = sample();
        let raw = serialize(&note).replace(ANALYSIS_ID, "550e8400-e29b-41d4-a716-446655440099");
        assert!(parse(&raw, ANALYSIS_ID, NOTE_ID).is_err());
    }

    #[test]
    fn round_trip_preserves_a_valid_note() {
        let note = sample();
        let parsed = parse(&serialize(&note), ANALYSIS_ID, NOTE_ID).unwrap();
        assert_eq!(parsed.title, note.title);
        assert_eq!(parsed.content, note.content);
    }

    fn sample() -> ForecastNote {
        ForecastNote {
            id: NOTE_ID.into(),
            analysis_id: ANALYSIS_ID.into(),
            date: "2026-07-23".into(),
            title: "Test".into(),
            note_type: "context".into(),
            source: "user".into(),
            content: "Contenu".into(),
            file_path: String::new(),
            created_at: "2026-07-23T00:00:00Z".into(),
            updated_at: "2026-07-23T00:00:00Z".into(),
        }
    }
}
