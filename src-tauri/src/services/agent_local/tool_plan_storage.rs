use std::path::{Path, PathBuf};

use uuid::Uuid;

use super::types_plan::AgentPlanRun;

pub(crate) fn upsert_run(runs: &mut Vec<AgentPlanRun>, run: AgentPlanRun) {
    runs.retain(|existing| existing.id != run.id);
    runs.insert(0, run);
}

pub(crate) fn plan_path(session_id: &str, plan_id: &str) -> Result<PathBuf, String> {
    super::session_store::validate_session_id(session_id)?;
    super::session_store::validate_session_id(plan_id)?;
    Ok(crate::services::paths::data_dir()
        .join("plans")
        .join(session_id)
        .join(format!("{plan_id}.md")))
}

pub(crate) async fn write_markdown(path: &Path, title: &str, content: &str) -> Result<(), String> {
    let dir = path
        .parent()
        .ok_or_else(|| super::tool_plan_messages::INVALID_PLAN.to_string())?;
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(|_| super::tool_plan_messages::PLAN_UNAVAILABLE.to_string())?;
    let body = format!("# {title}\n\n{content}\n");
    let tmp = path.with_extension(format!("{}.tmp", Uuid::new_v4()));
    tokio::fs::write(&tmp, body)
        .await
        .map_err(|_| super::tool_plan_messages::PLAN_UNAVAILABLE.to_string())?;
    tokio::fs::rename(&tmp, path)
        .await
        .map_err(|_| super::tool_plan_messages::PLAN_UNAVAILABLE.to_string())
}
