use super::types_session::PreserveReasoningSetting;

pub async fn update(id: &str, setting: PreserveReasoningSetting) -> Result<(), String> {
    super::session_store_updates::update_locked(id, |session| {
        session.preserve_reasoning = setting;
    })
    .await
}
