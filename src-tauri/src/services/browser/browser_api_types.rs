use super::surface_bounds::BrowserSurfaceBounds;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserSurfaceRequest {
    pub conversation_id: String,
    pub tab_id: String,
    pub url: Option<String>,
    pub bounds: BrowserSurfaceBounds,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BrowserCommandError {
    InvalidInput,
    Unavailable,
    Internal,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BrowserNavigationAction {
    Back,
    Forward,
    ReloadOrStop,
}
