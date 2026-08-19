use std::path::{Path, PathBuf};

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
    let body = format!("# {title}\n\n{content}\n");
    crate::services::private_store::atomic_write_async(path.to_path_buf(), body.into_bytes())
        .await
        .map_err(|_| super::tool_plan_messages::PLAN_UNAVAILABLE.to_string())
}
