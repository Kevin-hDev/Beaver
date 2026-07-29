use serde_json::Value;

use super::types_interactive::{
    AgentInteractiveAnswer, AgentInteractiveOption, AgentInteractiveQuestion,
};

pub const MAX_QUESTIONS: usize = 5;
pub const MIN_OPTIONS: usize = 2;
pub const MAX_OPTIONS: usize = 4;
pub const MAX_HEADER_CHARS: usize = 30;
pub const MAX_QUESTION_CHARS: usize = 500;
pub const MAX_LABEL_CHARS: usize = 80;
pub const MAX_OPTION_ID_CHARS: usize = 80;
pub const MAX_DESCRIPTION_CHARS: usize = 1500;
pub const MAX_PREVIEW_CHARS: usize = 1500;
pub const MAX_CUSTOM_ANSWER_CHARS: usize = 1500;

pub fn parse_questions(args: &Value) -> Result<Vec<AgentInteractiveQuestion>, String> {
    let items = args
        .get("questions")
        .and_then(Value::as_array)
        .ok_or_else(|| "paramètre 'questions' requis".to_string())?;
    if items.is_empty() || items.len() > MAX_QUESTIONS {
        return Err("questions doit contenir entre 1 et 5 éléments".into());
    }
    items.iter().map(parse_question).collect()
}

pub fn validate_answers(
    questions: &[AgentInteractiveQuestion],
    answers: Vec<AgentInteractiveAnswer>,
) -> Result<Vec<AgentInteractiveAnswer>, String> {
    if answers.len() != questions.len() {
        return Err("réponse interactive incomplète".into());
    }
    for answer in &answers {
        let Some(question) = questions.get(answer.question_index) else {
            return Err("réponse interactive invalide".into());
        };
        if answer.selected_labels.is_empty()
            && answer.selected_ids.is_empty()
            && answer.custom_answer.is_none()
        {
            return Err("réponse interactive vide".into());
        }
        if !question.multi_select && selected_count(answer) > 1 {
            return Err("choix multiple non autorisé".into());
        }
        let selected_other = answer.selected_labels.iter().any(|label| label == "other")
            || answer.selected_ids.iter().any(|id| id == "other");
        if selected_other != answer.custom_answer.is_some() {
            return Err("réponse autre invalide".into());
        }
        for label in &answer.selected_labels {
            if chars_len(label) > MAX_LABEL_CHARS {
                return Err("choix trop long".into());
            }
            if label != "other" && !question.options.iter().any(|option| option.label == *label) {
                return Err("choix inconnu".into());
            }
        }
        for id in &answer.selected_ids {
            if chars_len(id) > MAX_OPTION_ID_CHARS {
                return Err("identifiant de choix trop long".into());
            }
            if id != "other"
                && !question
                    .options
                    .iter()
                    .any(|option| option.id.as_deref() == Some(id))
            {
                return Err("choix inconnu".into());
            }
        }
        if let Some(custom) = answer.custom_answer.as_deref() {
            validate_text(custom, MAX_CUSTOM_ANSWER_CHARS, "réponse autre")?;
        }
    }
    Ok(answers)
}

fn parse_question(value: &Value) -> Result<AgentInteractiveQuestion, String> {
    let obj = value
        .as_object()
        .ok_or_else(|| "chaque question doit être un objet".to_string())?;
    let header = required_text(obj.get("header"), MAX_HEADER_CHARS, "header")?;
    let question = required_text(obj.get("question"), MAX_QUESTION_CHARS, "question")?;
    let options_value = obj
        .get("options")
        .and_then(Value::as_array)
        .ok_or_else(|| "options requis".to_string())?;
    if options_value.len() < MIN_OPTIONS || options_value.len() > MAX_OPTIONS {
        return Err("chaque question doit avoir 2 à 4 options".into());
    }
    let options: Vec<_> = options_value
        .iter()
        .map(parse_option)
        .collect::<Result<_, _>>()?;
    if options.iter().filter(|option| option.recommended).count() != 1 {
        return Err("chaque question doit avoir exactement une option recommandée".into());
    }
    Ok(AgentInteractiveQuestion {
        header,
        question,
        options,
        multi_select: obj
            .get("multi_select")
            .or_else(|| obj.get("multiSelect"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

fn parse_option(value: &Value) -> Result<AgentInteractiveOption, String> {
    let obj = value
        .as_object()
        .ok_or_else(|| "chaque option doit être un objet".to_string())?;
    Ok(AgentInteractiveOption {
        id: optional_text(obj.get("id"), MAX_OPTION_ID_CHARS, "id")?,
        label: required_text(obj.get("label"), MAX_LABEL_CHARS, "label")?,
        description: required_text(obj.get("description"), MAX_DESCRIPTION_CHARS, "description")?,
        recommended: obj
            .get("recommended")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        preview: optional_text(obj.get("preview"), MAX_PREVIEW_CHARS, "preview")?,
    })
}

fn required_text(value: Option<&Value>, max: usize, label: &str) -> Result<String, String> {
    let text = value
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{label} requis"))?;
    validate_text(text, max, label)
}

fn optional_text(value: Option<&Value>, max: usize, label: &str) -> Result<Option<String>, String> {
    match value.and_then(Value::as_str) {
        Some(text) => Ok(Some(validate_text(text, max, label)?)),
        None => Ok(None),
    }
}

fn validate_text(text: &str, max: usize, label: &str) -> Result<String, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(format!("{label} vide"));
    }
    if chars_len(trimmed) > max {
        return Err(format!("{label} trop long"));
    }
    Ok(trimmed.to_string())
}

fn chars_len(value: &str) -> usize {
    value.chars().count()
}

fn selected_count(answer: &AgentInteractiveAnswer) -> usize {
    answer.selected_labels.len().max(answer.selected_ids.len())
}
