use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

const MAX_GEOMETRY_BYTES: usize = 1_024;
const MAX_POSITION: i32 = 100_000;
const MIN_WIDTH: u32 = 820;
const MIN_HEIGHT: u32 = 600;
const MAX_SIZE: u32 = 16_000;
static GEOMETRY_LOCK: Mutex<()> = Mutex::const_new(());

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForecastWorkbenchGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub async fn get() -> Result<Option<ForecastWorkbenchGeometry>, String> {
    let _guard = GEOMETRY_LOCK.lock().await;
    let path =
        match crate::services::paths::data_file_for_read("forecast-workbench", "geometry.json")
            .await
        {
            Ok(path) => path,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(storage_error()),
        };
    let bytes = super::storage_io::read_bounded(&path, MAX_GEOMETRY_BYTES)
        .await
        .map_err(|_| storage_error())?;
    let geometry = serde_json::from_slice(&bytes).map_err(|_| storage_error())?;
    validate(&geometry)?;
    Ok(Some(geometry))
}

pub async fn save(geometry: ForecastWorkbenchGeometry) -> Result<(), String> {
    validate(&geometry)?;
    let _guard = GEOMETRY_LOCK.lock().await;
    let bytes = serde_json::to_vec(&geometry).map_err(|_| storage_error())?;
    let path = crate::services::paths::data_file_for_write("forecast-workbench", "geometry.json")
        .await
        .map_err(|_| storage_error())?;
    crate::services::private_store::atomic_write_async(path, bytes)
        .await
        .map_err(|_| storage_error())
}

fn validate(geometry: &ForecastWorkbenchGeometry) -> Result<(), String> {
    if geometry.x.unsigned_abs() > MAX_POSITION as u32
        || geometry.y.unsigned_abs() > MAX_POSITION as u32
        || !(MIN_WIDTH..=MAX_SIZE).contains(&geometry.width)
        || !(MIN_HEIGHT..=MAX_SIZE).contains(&geometry.height)
    {
        return Err("Géométrie Forecast invalide".into());
    }
    Ok(())
}

fn storage_error() -> String {
    "Impossible de sauvegarder la fenêtre Forecast".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_is_bounded() {
        let valid = ForecastWorkbenchGeometry {
            x: -1_000,
            y: 20,
            width: MIN_WIDTH,
            height: MIN_HEIGHT,
        };
        assert!(validate(&valid).is_ok());
        assert!(validate(&ForecastWorkbenchGeometry { width: 1, ..valid }).is_err());
        assert!(validate(&ForecastWorkbenchGeometry {
            x: 200_000,
            ..valid
        })
        .is_err());
    }
}
