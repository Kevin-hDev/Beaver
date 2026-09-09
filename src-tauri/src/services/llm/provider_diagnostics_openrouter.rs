//! One bounded routing observation per response; never changes request admission or retries.
//! Source (2026-09-09): https://openrouter.ai/docs/guides/features/router-metadata
use serde::Serialize;
use serde_json::Value;

#[path = "openrouter_routing_metadata.rs"]
mod metadata;
pub(super) use metadata::project;

const MAX_SNAPSHOTS: usize = 4;
const FILE_NAME: &str = "openrouter-routing.jsonl";

pub(super) fn log_path() -> std::path::PathBuf {
    crate::services::paths::data_dir()
        .join("logs")
        .join(FILE_NAME)
}

#[derive(Clone, Serialize)]
struct ResponseContext {
    model: String,
    request_id: Option<String>,
    http_status: u16,
    generation_id: Option<String>,
    retry_after_seconds: Option<u64>,
}

pub(crate) struct Observation {
    context: ResponseContext,
    routing: Option<metadata::RoutingMetadata>,
    snapshots: usize,
    snapshots_truncated: bool,
}

#[derive(Serialize)]
struct Entry<'a> {
    version: u8,
    timestamp: String,
    transport: &'static str,
    provider: &'static str,
    #[serde(flatten)]
    context: &'a ResponseContext,
    routing: &'a Option<metadata::RoutingMetadata>,
    snapshots_truncated: bool,
}

pub(crate) fn attach(
    response: &mut reqwest::Response,
    provider: &str,
    model: &str,
    request_id: Option<&str>,
) {
    if !super::super::openrouter_model_metadata::owns_catalog_metadata(provider) {
        return;
    }
    let context = ResponseContext {
        model: metadata::safe_label(model).unwrap_or_else(|| "unknown".into()),
        request_id: request_id.and_then(super::safe_request_id),
        http_status: response.status().as_u16(),
        generation_id: response
            .headers()
            .get("x-generation-id")
            .and_then(|value| value.to_str().ok())
            .and_then(metadata::generation_id),
        retry_after_seconds: super::super::provider_error::retry_after_seconds(response.headers()),
    };
    response.extensions_mut().insert(context);
}

pub(crate) fn take(response: &mut reqwest::Response) -> Option<Observation> {
    response
        .extensions_mut()
        .remove::<ResponseContext>()
        .map(|context| Observation {
            context,
            routing: None,
            snapshots: 0,
            snapshots_truncated: false,
        })
}

impl Observation {
    pub(crate) fn observe(&mut self, value: &Value) {
        // Prefer a body ID observed before routing metadata over the header fallback.
        if self.snapshots == 0 {
            if let Some(id) = value
                .get("id")
                .and_then(Value::as_str)
                .and_then(metadata::generation_id)
            {
                self.context.generation_id = Some(id);
            }
        }
        if value.get("openrouter_metadata").is_none() {
            return;
        }
        if self.snapshots >= MAX_SNAPSHOTS {
            self.snapshots_truncated = true;
            return;
        }
        self.snapshots += 1;
        if let Some(routing) = project(value) {
            self.routing = Some(routing);
        }
    }
}

impl Drop for Observation {
    fn drop(&mut self) {
        // Flush on error/cancellation too. Missing metadata stays null, never a diagnosis.
        let entry = Entry {
            version: 1,
            timestamp: chrono::Utc::now().to_rfc3339(),
            transport: "openrouter_routing",
            provider: "openrouter",
            context: &self.context,
            routing: &self.routing,
            snapshots_truncated: self.snapshots_truncated,
        };
        // Successful routing observations have their own bounded file so they
        // cannot evict provider failure evidence from provider-errors.jsonl.
        if super::write_at(&log_path(), &entry).is_err() {
            log::warn!("[llm] routing diagnostic log unavailable");
        }
    }
}
