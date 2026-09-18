use super::types_session::{AgentSession, SubagentExtensionOwner, SubagentExtensionOwnership};
use super::{session_store, subagent_status, tool_delegate_child};

fn owner(id: &str) -> SubagentExtensionOwner {
    SubagentExtensionOwner {
        extension_id: id.into(),
        extension_version: "1.0.0".into(),
        extension_fingerprint: "a".repeat(64),
    }
}

async fn sessions(ownership: Option<SubagentExtensionOwnership>) -> (AgentSession, AgentSession) {
    let parent = session_store::create_full("Parent", "model", "ollama", false, None)
        .await
        .unwrap();
    let mut child = tool_delegate_child::create_child(
        &parent,
        &parent.id,
        "explorer",
        "original mission",
        "Child",
        "original description",
        "explorer",
        "original-run",
    )
    .await
    .unwrap();
    child.subagent_status = Some(subagent_status::COMPLETED.into());
    child.subagent_extension_owner = ownership;
    session_store::save(&child).await.unwrap();
    (parent, child)
}

#[tokio::test]
async fn native_redeployment_cannot_reuse_or_mutate_an_extension_child() {
    rejected_without_mutation(
        Some(SubagentExtensionOwnership::Valid(owner("test.owner"))),
        None,
    )
    .await;
}

async fn rejected_without_mutation(
    ownership: Option<SubagentExtensionOwnership>,
    requester: Option<SubagentExtensionOwner>,
) {
    let (parent, child) = sessions(ownership).await;
    let before = serde_json::to_value(session_store::get(&child.id).await.unwrap()).unwrap();
    let result = tool_delegate_child::prepare_existing_child(
        &child.id,
        &parent.id,
        requester.as_ref(),
        "explorer",
        "new mission",
        "Renamed",
        "new description",
        "explorer",
        "new-run",
    )
    .await;
    assert!(result.is_err(), "a different owner cannot redeploy a child");
    let after = serde_json::to_value(session_store::get(&child.id).await.unwrap()).unwrap();
    assert_eq!(
        after, before,
        "ownership must be checked before any durable mutation"
    );
    session_store::delete_one(&child.id).await.unwrap();
    session_store::delete_one(&parent.id).await.unwrap();
}

#[tokio::test]
async fn extension_redeployment_rejects_other_native_stale_and_invalid_owners() {
    let expected = owner("test.owner");
    let mut stale = expected.clone();
    stale.extension_fingerprint = "b".repeat(64);
    for ownership in [
        None,
        Some(SubagentExtensionOwnership::Valid(owner("test.neighbor"))),
        Some(SubagentExtensionOwnership::Valid(stale)),
        Some(SubagentExtensionOwnership::Invalid(
            serde_json::json!({"broken": true}),
        )),
    ] {
        rejected_without_mutation(ownership, Some(expected.clone())).await;
    }
    rejected_without_mutation(
        Some(SubagentExtensionOwnership::Invalid(
            serde_json::json!({"broken": true}),
        )),
        None,
    )
    .await;
}

#[tokio::test]
async fn native_and_extension_owners_can_redeploy_their_own_children() {
    for requester in [None, Some(owner("test.owner"))] {
        let ownership = requester.clone().map(SubagentExtensionOwnership::Valid);
        let (parent, child) = sessions(ownership.clone()).await;
        let prepared = tool_delegate_child::prepare_existing_child(
            &child.id,
            &parent.id,
            requester.as_ref(),
            "explorer",
            "new mission",
            "Renamed",
            "new description",
            "explorer",
            "new-run",
        )
        .await
        .unwrap();
        assert_eq!(prepared.subagent_extension_owner, ownership);
        let saved = session_store::get(&child.id).await.unwrap();
        assert_eq!(saved.subagent_status.as_deref(), Some("running"));
        assert_eq!(saved.subagent_prompt.as_deref(), Some("new mission"));
        assert_eq!(saved.subagent_run_id.as_deref(), Some("new-run"));
        assert_eq!(saved.subagent_extension_owner, ownership);
        session_store::delete_one(&child.id).await.unwrap();
        session_store::delete_one(&parent.id).await.unwrap();
    }
}
