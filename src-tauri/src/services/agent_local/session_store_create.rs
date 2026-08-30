use super::types_session::AgentSession;
use chrono::Utc;
use uuid::Uuid;

pub async fn create_gateway(
    name: &str,
    model: &str,
    provider: &str,
    gateway_channel_key: String,
) -> Result<AgentSession, String> {
    let mut session = create_full(name, model, provider, false, None).await?;
    session.is_gateway = true;
    session.gateway_channel_key = Some(gateway_channel_key);
    super::session_store::save(&session).await?;
    Ok(session)
}

pub async fn create_full(
    name: &str,
    model: &str,
    provider: &str,
    is_heartbeat: bool,
    project_id: Option<String>,
) -> Result<AgentSession, String> {
    create_full_with_fast_mode(name, model, provider, is_heartbeat, project_id, false).await
}

pub async fn create_with_project(
    name: &str,
    model: &str,
    provider: &str,
    is_heartbeat: bool,
    project_id: Option<String>,
) -> Result<AgentSession, String> {
    create_with_project_inner(name, model, provider, is_heartbeat, project_id, false).await
}

pub async fn create_with_project_and_fast_mode(
    name: &str,
    model: &str,
    provider: &str,
    project_id: Option<String>,
    fast_mode_enabled: bool,
) -> Result<AgentSession, String> {
    create_with_project_inner(name, model, provider, false, project_id, fast_mode_enabled).await
}

async fn create_full_with_fast_mode(
    name: &str,
    model: &str,
    provider: &str,
    is_heartbeat: bool,
    project_id: Option<String>,
    fast_mode_enabled: bool,
) -> Result<AgentSession, String> {
    let reasoning_mode = crate::services::reasoning::default_mode(provider, model);
    let now = Utc::now();
    let session = AgentSession {
        schema_version: super::session_limits::CURRENT_SESSION_SCHEMA_VERSION,
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        created_at: now,
        updated_at: Some(now),
        archived_at: None,
        pinned_at: None,
        model: model.to_string(),
        provider: provider.to_string(),
        thinking_enabled: crate::services::reasoning::enabled(reasoning_mode.as_deref(), false),
        fast_mode_enabled,
        reasoning_mode,
        preserve_reasoning: Default::default(),
        accumulated_tokens: 0,
        context_tokens: None,
        compression_profile_selection: None,
        compression_count: 0,
        messages: Vec::new(),
        todos: Vec::new(),
        todo_neglect_count: 0,
        todo_runs: Vec::new(),
        active_todo_run_id: None,
        stream_failures: Vec::new(),
        diagnostic_runs: Vec::new(),
        plan_mode_enabled: false,
        plan_runs: Vec::new(),
        active_plan_id: None,
        plan_workflow_status: Default::default(),
        is_heartbeat,
        is_gateway: false,
        gateway_channel_key: None,
        project_id,
        working_dir: String::new(),
        working_dir_managed: false,
        parent_session_id: None,
        subagent_type: None,
        subagent_worktree: None,
        subagent_prompt: None,
        subagent_status: None,
        subagent_run_id: None,
        subagent_description: None,
        subagent_color_key: None,
        subagent_summary: None,
        subagent_last_activity: None,
        subagent_queued_prompts: Vec::new(),
        subagent_hidden_reports: Vec::new(),
        clone_parent_session_id: None,
        clone_parent_message_id: None,
        clone_mode: None,
        clone_summary: None,
        clone_read_files: Vec::new(),
        clone_modified_files: Vec::new(),
        clone_root_session_id: None,
        git_branch: None,
    };
    super::session_store::save(&session).await?;
    Ok(session)
}

async fn create_with_project_inner(
    name: &str,
    model: &str,
    provider: &str,
    is_heartbeat: bool,
    project_id: Option<String>,
    fast_mode_enabled: bool,
) -> Result<AgentSession, String> {
    let project_path = match project_id.as_deref() {
        Some(project_id) => Some(super::directory_access::project_path(project_id).await?),
        None => None,
    };
    let mut session = create_full_with_fast_mode(
        name,
        model,
        provider,
        is_heartbeat,
        project_id,
        fast_mode_enabled,
    )
    .await?;
    if let Some(path) = project_path {
        session.working_dir = path.to_string_lossy().to_string();
        if let Err(error) = super::session_store::save(&session).await {
            let _ = super::session_store::delete_one(&session.id).await;
            return Err(error);
        }
    }
    Ok(session)
}
