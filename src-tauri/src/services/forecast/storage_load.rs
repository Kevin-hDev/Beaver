use super::limits::MAX_STORED_ANALYSIS_BYTES;
use super::types::ForecastResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForecastLoadError {
    InvalidId,
    NotFound,
    Unavailable,
    Corrupt,
}

impl ForecastLoadError {
    pub fn message(self) -> &'static str {
        match self {
            Self::InvalidId => "Identifiant d'analyse invalide",
            Self::NotFound => "Analyse introuvable",
            Self::Unavailable => "Accès à l'analyse impossible",
            Self::Corrupt => "Données d'analyse corrompues",
        }
    }
}

pub async fn load(id: &str) -> Result<ForecastResult, String> {
    load_classified(id)
        .await
        .map_err(|error| error.message().to_string())
}

pub async fn load_classified(id: &str) -> Result<ForecastResult, ForecastLoadError> {
    super::storage_paths::validate_analysis_id(id).map_err(|_| ForecastLoadError::InvalidId)?;
    let path = super::storage_paths::analysis_path_for_read(id)
        .await
        .map_err(classify_io)?;
    let data = super::storage_io::read_bounded(&path, MAX_STORED_ANALYSIS_BYTES)
        .await
        .map_err(classify_io)?;
    decode(&data)
}

fn classify_io(error: std::io::Error) -> ForecastLoadError {
    if error.kind() == std::io::ErrorKind::NotFound {
        ForecastLoadError::NotFound
    } else {
        ForecastLoadError::Unavailable
    }
}

fn decode(data: &[u8]) -> Result<ForecastResult, ForecastLoadError> {
    serde_json::from_slice(data).map_err(|_| ForecastLoadError::Corrupt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn invalid_and_missing_analyses_have_distinct_failures() {
        assert_eq!(
            load_classified("../analysis").await.unwrap_err(),
            ForecastLoadError::InvalidId
        );
        let missing = uuid::Uuid::new_v4().to_string();
        assert_eq!(
            load_classified(&missing).await.unwrap_err(),
            ForecastLoadError::NotFound
        );
    }

    #[test]
    fn invalid_json_is_reported_as_corrupt() {
        assert_eq!(decode(b"not json").unwrap_err(), ForecastLoadError::Corrupt);
    }
}
