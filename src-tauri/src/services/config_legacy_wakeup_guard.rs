use crate::models::ScheduledWakeup;
use std::path::Path;

pub(super) fn ensure_round_trip(path: &Path) -> Result<(), String> {
    let content = match crate::services::private_store::read_bounded_regular(path, 2 * 1024 * 1024)
        .map_err(|_| write_error())?
    {
        crate::services::private_store::BoundedFile::Missing => return Ok(()),
        crate::services::private_store::BoundedFile::Content(content) => content,
    };
    let value: serde_json::Value = serde_json::from_slice(&content).map_err(|_| write_error())?;
    let Some(entries) = value.get("scheduled_wakeups") else {
        return Ok(());
    };
    let entries = entries.as_array().ok_or_else(write_error)?;
    if entries
        .iter()
        .any(|entry| serde_json::from_value::<ScheduledWakeup>(entry.clone()).is_err())
    {
        ::log::warn!("[config] réécriture refusée : wakeup historique inconnu");
        return Err(write_error());
    }
    Ok(())
}

fn write_error() -> String {
    "Écriture de la configuration impossible.".into()
}
