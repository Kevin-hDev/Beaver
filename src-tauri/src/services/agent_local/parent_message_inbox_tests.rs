use super::parent_message_inbox::ParentMessageInbox;
use crate::models::agent_turn_contract::{NewUserTurnInput, SkillReference};

#[tokio::test]
async fn one_intention_is_admitted_only_after_an_explicit_commit_signal() {
    let inbox = ParentMessageInbox::new();
    inbox.enqueue(user("premier")).await.unwrap();
    inbox.enqueue(user("second")).await.unwrap();

    assert_eq!(inbox.len().await, 2);
    let admitted = inbox
        .admit_one_after_commit(|input| async move { Ok::<_, String>(input.content) })
        .await
        .unwrap();
    assert_eq!(admitted.as_deref(), Some("premier"));
    assert_eq!(inbox.len().await, 1);
}

#[tokio::test]
async fn failed_admission_keeps_the_front_intention_and_never_skips_it() {
    let inbox = ParentMessageInbox::new();
    inbox.enqueue(user("premier")).await.unwrap();
    inbox.enqueue(user("second")).await.unwrap();

    let failed = inbox
        .admit_one_after_commit(|_| async { Err::<String, _>("blocked".to_string()) })
        .await;
    assert!(failed.is_err());
    assert_eq!(inbox.len().await, 2);
    let admitted = inbox
        .admit_one_after_commit(|input| async move { Ok::<_, String>(input.content) })
        .await
        .unwrap();
    assert_eq!(admitted.as_deref(), Some("premier"));
}

#[tokio::test]
async fn inbox_rejects_more_than_eight_waiting_intentions() {
    let inbox = ParentMessageInbox::new();
    for index in 0..8 {
        inbox.enqueue(user(&index.to_string())).await.unwrap();
    }
    assert!(inbox.enqueue(user("neuf")).await.is_err());
    assert_eq!(inbox.len().await, 8);
}

#[tokio::test]
async fn eight_skill_references_remain_one_queue_entry_and_close_is_fail_closed() {
    let inbox = ParentMessageInbox::new();
    let mut input = user("question");
    input.skills = (0..8)
        .map(|index| SkillReference { id: format!("skill-{index}"), name: None })
        .collect();
    assert!(inbox.enqueue(input).await.unwrap());
    assert_eq!(inbox.len().await, 1);
    inbox.close().await;
    assert!(!inbox.enqueue(user("trop tard")).await.unwrap());
    assert_eq!(inbox.len().await, 1);
}

fn user(content: &str) -> NewUserTurnInput {
    NewUserTurnInput { content: content.into(), files: Vec::new(), skills: Vec::new() }
}
