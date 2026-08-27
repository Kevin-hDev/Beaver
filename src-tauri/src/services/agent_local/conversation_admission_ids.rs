use std::collections::HashSet;

use uuid::Uuid;

use super::conversation_admission::{error, ConversationAdmissionError};

pub(super) fn unique_uuid<F>(
    used: &mut HashSet<String>,
    generator: &mut F,
) -> Result<String, ConversationAdmissionError>
where
    F: FnMut() -> String,
{
    for _ in 0..4 {
        let candidate = generator();
        let valid = Uuid::parse_str(&candidate)
            .ok()
            .is_some_and(|id| id.get_version_num() == 4);
        if valid && used.insert(candidate.clone()) {
            return Ok(candidate);
        }
    }
    Err(error())
}

pub(super) fn allocate_ids<F>(
    used: &mut HashSet<String>,
    mut generator: F,
) -> Result<(String, String, String), ConversationAdmissionError>
where
    F: FnMut() -> String,
{
    Ok((
        unique_uuid(used, &mut generator)?,
        unique_uuid(used, &mut generator)?,
        unique_uuid(used, &mut generator)?,
    ))
}
