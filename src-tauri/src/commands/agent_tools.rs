use crate::services::agent_local::tool_skill_loader;
use crate::services::agent_local::types_tools::SkillInfo;

#[tauri::command]
pub async fn list_skills() -> Result<Vec<SkillInfo>, String> {
    tool_skill_loader::list_skills().await
}

#[tauri::command]
pub async fn load_skill(skill_id: String) -> Result<String, String> {
    tool_skill_loader::load_skill(&skill_id).await
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn tauri_loader_without_a_session_refuses_extension_qualified_skills() {
        let error = super::load_skill("extension:example.plugin:guide".into())
            .await
            .unwrap_err();

        assert_eq!(error, "Identifiant de skill invalide");
    }
}
