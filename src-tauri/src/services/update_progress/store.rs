use super::types::{UpdateOperationSnapshot, UpdateOperationStatus, UpdateProgressMode};
use indexmap::IndexMap;

pub const MAX_OPERATIONS: usize = 32;
const MAX_ID_CHARS: usize = 64;
const MAX_LABEL_CHARS: usize = 200;
const MAX_ERROR_KEY_CHARS: usize = 120;

#[derive(Default)]
pub(super) struct UpdateProgressStore {
    operations: IndexMap<String, UpdateOperationSnapshot>,
}

impl UpdateProgressStore {
    pub fn upsert(&mut self, operation: UpdateOperationSnapshot) -> Result<bool, &'static str> {
        validate(&operation)?;
        if let Some(current) = self.operations.get(&operation.id) {
            // Cancellation is monotonic: only a terminal result may replace it.
            if current.status == UpdateOperationStatus::Cancelling
                && !operation.status.is_terminal()
            {
                return Ok(false);
            }
            if operation.sequence <= current.sequence {
                return Ok(false);
            }
        } else if self.operations.len() == MAX_OPERATIONS {
            let terminal = self
                .operations
                .iter()
                .find_map(|(id, value)| value.status.is_terminal().then(|| id.clone()))
                .ok_or("update-progress-full")?;
            self.operations.shift_remove(&terminal);
        }
        self.operations.insert(operation.id.clone(), operation);
        Ok(true)
    }

    pub fn dismiss(&mut self, id: &str) -> Result<bool, &'static str> {
        validate_id(id)?;
        if self
            .operations
            .get(id)
            .is_some_and(|operation| !operation.status.is_terminal())
        {
            return Err("command-not-available");
        }
        Ok(self.operations.shift_remove(id).is_some())
    }

    pub fn get(&self, id: &str) -> Option<&UpdateOperationSnapshot> {
        self.operations.get(id)
    }

    pub fn snapshot(&self) -> Vec<UpdateOperationSnapshot> {
        self.operations.values().cloned().collect()
    }
}

pub(super) fn validate_id(id: &str) -> Result<(), &'static str> {
    let len = id.chars().count();
    if len == 0
        || len > MAX_ID_CHARS
        || !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_:".contains(c))
    {
        return Err("update-progress-invalid");
    }
    Ok(())
}

fn validate(operation: &UpdateOperationSnapshot) -> Result<(), &'static str> {
    validate_id(&operation.id)?;
    let label_len = operation.label.chars().count();
    if label_len == 0 || label_len > MAX_LABEL_CHARS {
        return Err("update-progress-invalid");
    }
    if operation.percent.is_some_and(|value| value > 100)
        || (operation.progress_mode == UpdateProgressMode::Determinate)
            != operation.percent.is_some()
        || (operation.status.is_terminal() && operation.can_cancel)
        || (operation.can_retry && operation.status != UpdateOperationStatus::Failed)
        || operation.queue_position.is_some() != (operation.status == UpdateOperationStatus::Queued)
        || operation
            .error_key
            .as_ref()
            .is_some_and(|key| key.chars().count() > MAX_ERROR_KEY_CHARS)
    {
        return Err("update-progress-invalid");
    }
    Ok(())
}
