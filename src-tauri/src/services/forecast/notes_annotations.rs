use super::notes_types::ForecastNote;
use super::notes_validation;
use crate::services::forecast::types::{
    Annotation, AnnotationSource, ForecastResult, MAX_ANNOTATIONS,
};

pub(super) fn upsert(analysis: &mut ForecastResult, note: &ForecastNote) -> Result<bool, String> {
    let source = source_from_text(&note.source)?;
    if let Some(annotation) = analysis
        .annotations
        .iter_mut()
        .find(|item| item.id == note.id)
    {
        let changed = annotation.date != note.date
            || annotation.text != note.title
            || !same_source(&annotation.source, &source)
            || annotation.note_content.as_deref() != Some(note.content.as_str())
            || annotation.note_type.as_deref() != Some(note.note_type.as_str())
            || annotation.note_created_at.as_deref() != Some(note.created_at.as_str())
            || annotation.note_updated_at.as_deref() != Some(note.updated_at.as_str());
        if changed {
            *annotation = from_note(note, source);
        }
        return Ok(changed);
    }
    if analysis.annotations.len() >= MAX_ANNOTATIONS {
        return Err("Limite de notes atteinte".into());
    }
    analysis.annotations.push(from_note(note, source));
    Ok(true)
}

pub(super) fn note_from_annotation(
    analysis: &ForecastResult,
    annotation: &Annotation,
) -> Result<ForecastNote, String> {
    let title = annotation
        .note_title
        .clone()
        .unwrap_or_else(|| legacy_title(&annotation.text));
    let note = ForecastNote {
        id: annotation.id.clone(),
        analysis_id: analysis.id.clone(),
        date: annotation.date.clone(),
        title,
        note_type: annotation
            .note_type
            .clone()
            .unwrap_or_else(|| "context".into()),
        source: source_text(&annotation.source).into(),
        content: annotation
            .note_content
            .clone()
            .unwrap_or_else(|| annotation.text.clone()),
        file_path: String::new(),
        created_at: annotation
            .note_created_at
            .clone()
            .unwrap_or_else(|| analysis.created_at.clone()),
        updated_at: annotation
            .note_updated_at
            .clone()
            .unwrap_or_else(|| analysis.created_at.clone()),
    };
    validate_recovered_note(&note)?;
    Ok(note)
}

fn legacy_title(text: &str) -> String {
    let title = text
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("Note")
        .trim();
    title.chars().take(80).collect()
}

fn validate_recovered_note(note: &ForecastNote) -> Result<(), String> {
    notes_validation::id(&note.id, "Note invalide")?;
    notes_validation::field(&note.date)?;
    notes_validation::title(&note.title)?;
    notes_validation::note_type(&note.note_type)?;
    notes_validation::source(&note.source)?;
    notes_validation::content(&note.content)?;
    notes_validation::metadata(&note.created_at)?;
    notes_validation::metadata(&note.updated_at)?;
    Ok(())
}

fn from_note(note: &ForecastNote, source: AnnotationSource) -> Annotation {
    Annotation {
        id: note.id.clone(),
        date: note.date.clone(),
        text: note.title.clone(),
        source,
        note_title: Some(note.title.clone()),
        note_type: Some(note.note_type.clone()),
        note_content: Some(note.content.clone()),
        note_created_at: Some(note.created_at.clone()),
        note_updated_at: Some(note.updated_at.clone()),
    }
}

fn source_from_text(source: &str) -> Result<AnnotationSource, String> {
    match source {
        "user" => Ok(AnnotationSource::User),
        "llm" => Ok(AnnotationSource::Llm),
        _ => Err("Source de note invalide".into()),
    }
}

fn source_text(source: &AnnotationSource) -> &'static str {
    match source {
        AnnotationSource::User => "user",
        AnnotationSource::Llm => "llm",
    }
}

fn same_source(left: &AnnotationSource, right: &AnnotationSource) -> bool {
    matches!(
        (left, right),
        (AnnotationSource::User, AnnotationSource::User)
            | (AnnotationSource::Llm, AnnotationSource::Llm)
    )
}
