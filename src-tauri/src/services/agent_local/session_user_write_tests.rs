use super::session_user_write::{ensure_allowed, SUBAGENT_READ_ONLY};

#[tokio::test]
async fn root_session_accepts_user_writes() {
    let session = super::session_store::create_full("Root", "model", "provider", false, None)
        .await
        .expect("session");

    assert_eq!(ensure_allowed(&session.id).await, Ok(()));
    super::session_store::delete_one(&session.id)
        .await
        .expect("cleanup");
}

#[tokio::test]
async fn child_session_rejects_user_writes_with_a_fixed_code() {
    let mut child = super::session_store::create_full("Child", "model", "provider", false, None)
        .await
        .expect("session");
    child.parent_session_id = Some(uuid::Uuid::new_v4().to_string());
    super::session_store::save(&child)
        .await
        .expect("save child");

    assert_eq!(
        ensure_allowed(&child.id).await,
        Err(SUBAGENT_READ_ONLY.to_string()),
    );
    super::session_store::delete_one(&child.id)
        .await
        .expect("cleanup");
}

#[tokio::test]
async fn unknown_session_is_refused() {
    let unknown = uuid::Uuid::new_v4().to_string();

    assert!(ensure_allowed(&unknown).await.is_err());
}
