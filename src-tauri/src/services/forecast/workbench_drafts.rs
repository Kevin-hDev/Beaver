use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

const MAX_DRAFTS: usize = 100;
const MAX_DRAFT_STORE_BYTES: usize = 64 * 1024;
const SECTIONS: &[&str] = &[
    "data",
    "forecast",
    "evaluation",
    "comparison",
    "scenarios",
    "notes",
    "report",
];
static STORE_LOCK: Mutex<()> = Mutex::const_new(());

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForecastWorkbenchDraft {
    pub section: String,
    pub revision: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredDraft {
    session_id: String,
    analysis_id: Option<String>,
    draft: ForecastWorkbenchDraft,
}

impl Default for ForecastWorkbenchDraft {
    fn default() -> Self {
        Self {
            section: "data".into(),
            revision: 1,
        }
    }
}

pub async fn load(
    context: &super::workbench_context::ForecastWorkbenchContext,
) -> Result<ForecastWorkbenchDraft, String> {
    let _guard = STORE_LOCK.lock().await;
    let drafts = read_store().await?;
    Ok(drafts
        .into_iter()
        .find(|stored| matches_context(stored, context))
        .map(|stored| stored.draft)
        .unwrap_or_default())
}

pub async fn save(
    context: &super::workbench_context::ForecastWorkbenchContext,
    draft: &ForecastWorkbenchDraft,
) -> Result<(), String> {
    validate(draft)?;
    let _guard = STORE_LOCK.lock().await;
    let mut drafts = read_store().await?;
    drafts.retain(|stored| !matches_context(stored, context));
    drafts.push(StoredDraft {
        session_id: context.session_id.clone(),
        analysis_id: context.analysis_id.clone(),
        draft: draft.clone(),
    });
    if drafts.len() > MAX_DRAFTS {
        drafts.drain(0..drafts.len() - MAX_DRAFTS);
    }
    write_store(&drafts).await
}

pub fn next(
    current: &ForecastWorkbenchDraft,
    section: String,
) -> Result<ForecastWorkbenchDraft, String> {
    let next = ForecastWorkbenchDraft {
        section,
        revision: current.revision.checked_add(1).unwrap_or(1),
    };
    validate(&next)?;
    Ok(next)
}

fn validate(draft: &ForecastWorkbenchDraft) -> Result<(), String> {
    if draft.revision == 0 || !SECTIONS.contains(&draft.section.as_str()) {
        return Err("Brouillon Forecast invalide".into());
    }
    Ok(())
}

fn matches_context(
    stored: &StoredDraft,
    context: &super::workbench_context::ForecastWorkbenchContext,
) -> bool {
    stored.session_id == context.session_id && stored.analysis_id == context.analysis_id
}

async fn read_store() -> Result<Vec<StoredDraft>, String> {
    let path = match crate::services::paths::data_file_for_read("forecast-workbench", "drafts.json")
        .await
    {
        Ok(path) => path,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(_) => return Err(storage_error()),
    };
    let bytes = super::storage_io::read_bounded(&path, MAX_DRAFT_STORE_BYTES)
        .await
        .map_err(|_| storage_error())?;
    let drafts: Vec<StoredDraft> = serde_json::from_slice(&bytes).map_err(|_| storage_error())?;
    if drafts.len() > MAX_DRAFTS || drafts.iter().any(|stored| validate(&stored.draft).is_err()) {
        return Err(storage_error());
    }
    Ok(drafts)
}

async fn write_store(drafts: &[StoredDraft]) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(drafts).map_err(|_| storage_error())?;
    if bytes.len() > MAX_DRAFT_STORE_BYTES {
        return Err(storage_error());
    }
    let path = crate::services::paths::data_file_for_write("forecast-workbench", "drafts.json")
        .await
        .map_err(|_| storage_error())?;
    crate::services::private_store::atomic_write_async(path, bytes)
        .await
        .map_err(|_| storage_error())
}

fn storage_error() -> String {
    "Impossible de sauvegarder le brouillon Forecast".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_and_revision_are_strictly_validated() {
        assert!(next(&ForecastWorkbenchDraft::default(), "forecast".into()).is_ok());
        assert!(next(&ForecastWorkbenchDraft::default(), "notes".into()).is_ok());
        assert!(next(&ForecastWorkbenchDraft::default(), "unknown".into()).is_err());
    }

    #[tokio::test]
    async fn draft_round_trip_is_scoped_to_its_context() {
        let context = super::super::workbench_context::ForecastWorkbenchContext {
            session_id: uuid::Uuid::new_v4().to_string(),
            analysis_id: Some(uuid::Uuid::new_v4().to_string()),
            revision: 1,
        };
        let draft = next(&ForecastWorkbenchDraft::default(), "forecast".into()).unwrap();

        save(&context, &draft).await.unwrap();

        assert_eq!(load(&context).await.unwrap(), draft);
    }
}
