use super::data_quality::DataProfile;
use super::limits::MAX_INLINE_DATA_BYTES;
use super::types::ForecastRequest;
use crate::services::workspace_scope::WorkspaceScope;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

#[derive(Serialize, Deserialize)]
pub(super) struct StoredDataProfile {
    pub profile: DataProfile,
    pub data: String,
    #[serde(default)]
    pub workspace: WorkspaceScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataProfileLoadError {
    InvalidId,
    NotFound,
    Unavailable,
    Corrupt,
}

impl DataProfileLoadError {
    pub fn message(self) -> &'static str {
        match self {
            Self::InvalidId => "Identifiant de profil invalide",
            Self::NotFound => "Profil de données introuvable",
            Self::Unavailable => "Accès au profil de données impossible",
            Self::Corrupt => "Profil de données invalide",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataProfileHydrateError {
    Ambiguous,
    Incompatible,
    Load(DataProfileLoadError),
}

impl DataProfileHydrateError {
    pub fn message(self) -> &'static str {
        match self {
            Self::Ambiguous => "Référence de profil ambiguë",
            Self::Incompatible => "Profil de données incompatible",
            Self::Load(error) => error.message(),
        }
    }
}

pub async fn hydrate_request(
    session_id: &str,
    request: &mut ForecastRequest,
) -> Result<(), String> {
    hydrate_request_classified(session_id, request)
        .await
        .map_err(|error| error.message().to_string())
}

pub async fn hydrate_request_classified(
    session_id: &str,
    request: &mut ForecastRequest,
) -> Result<(), DataProfileHydrateError> {
    let Some(id) = request.data_profile_id.as_deref() else {
        return Ok(());
    };
    if request.data.is_some() || request.file_path.is_some() {
        return Err(DataProfileHydrateError::Ambiguous);
    }
    let stored = load(session_id, id)
        .await
        .map_err(DataProfileHydrateError::Load)?;
    if !matches_request(&stored.profile, request) {
        return Err(DataProfileHydrateError::Incompatible);
    }
    request.data = Some(stored.data);
    Ok(())
}

pub async fn load_profile_classified(
    session_id: &str,
    id: &str,
) -> Result<DataProfile, DataProfileLoadError> {
    load(session_id, id).await.map(|stored| stored.profile)
}

async fn load(session_id: &str, id: &str) -> Result<StoredDataProfile, DataProfileLoadError> {
    uuid::Uuid::parse_str(id).map_err(|_| DataProfileLoadError::InvalidId)?;
    let workspace = crate::services::workspace_scope::resolve(session_id)
        .await
        .map_err(|_| DataProfileLoadError::Unavailable)?;
    let max_bytes = MAX_INLINE_DATA_BYTES.saturating_add(64 * 1024);
    let path = super::data_profiles::profile_path_for_read(&workspace, id)
        .await
        .map_err(classify_io)?;
    let file = tokio::fs::File::open(path).await.map_err(classify_io)?;
    let mut data = Vec::with_capacity(max_bytes.min(64 * 1024));
    file.take((max_bytes + 1) as u64)
        .read_to_end(&mut data)
        .await
        .map_err(classify_io)?;
    if data.len() > max_bytes {
        return Err(DataProfileLoadError::Corrupt);
    }
    let mut stored: StoredDataProfile =
        serde_json::from_slice(&data).map_err(|_| DataProfileLoadError::Corrupt)?;
    if stored.workspace != workspace
        || stored.profile.id != id
        || !stored.profile.valid
        || stored.data.len() > MAX_INLINE_DATA_BYTES
    {
        return Err(DataProfileLoadError::Corrupt);
    }
    super::data_profile_migration::ensure_fingerprint(&mut stored.profile, &stored.data);
    Ok(stored)
}

fn classify_io(error: std::io::Error) -> DataProfileLoadError {
    if error.kind() == std::io::ErrorKind::NotFound {
        DataProfileLoadError::NotFound
    } else {
        DataProfileLoadError::Unavailable
    }
}

fn matches_request(profile: &DataProfile, request: &ForecastRequest) -> bool {
    profile.target_column == request.target_column
        && profile.date_column == request.date_column
        && profile.series_column == request.series_column
        && profile.covariate_columns == request.covariate_columns
        && profile.frequency == request.frequency
        && profile.horizon == request.horizon
        && profile
            .confidence_level
            .is_some_and(|confidence| (confidence - request.confidence_level).abs() < 0.000_001)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn invalid_and_missing_profiles_have_distinct_failures() {
        assert_eq!(
            load_profile_classified("not-a-session", "not-a-uuid")
                .await
                .unwrap_err(),
            DataProfileLoadError::InvalidId
        );
        assert_eq!(
            classify_io(std::io::Error::from(std::io::ErrorKind::NotFound)),
            DataProfileLoadError::NotFound
        );
    }
}
