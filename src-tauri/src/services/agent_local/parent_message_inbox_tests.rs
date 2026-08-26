use super::parent_message_inbox::ParentMessageInbox;
use crate::models::agent_turn_contract::{NewUserTurnInput, SkillReference};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::oneshot;

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

#[tokio::test]
async fn close_before_drain_never_invokes_the_admission_closure() {
    let inbox = ParentMessageInbox::new();
    inbox.enqueue(user("question")).await.unwrap();
    inbox.close().await;
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&calls);

    let admitted = inbox
        .admit_one_after_commit(move |_| {
            observed.fetch_add(1, Ordering::SeqCst);
            async { Ok::<_, String>(()) }
        })
        .await
        .unwrap();

    assert!(admitted.is_none());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(inbox.len().await, 1);
}

#[tokio::test]
async fn concurrent_drain_and_close_have_one_linearized_order() {
    let inbox = Arc::new(ParentMessageInbox::new());
    inbox.enqueue(user("question")).await.unwrap();
    let (entered_tx, entered_rx) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel();
    let draining_inbox = Arc::clone(&inbox);
    let draining = tokio::spawn(async move {
        draining_inbox
            .admit_one_after_commit(move |_| async move {
                let _ = entered_tx.send(());
                let _ = release_rx.await;
                Ok::<_, String>("admitted")
            })
            .await
    });
    entered_rx.await.unwrap();
    let closing_inbox = Arc::clone(&inbox);
    let closing = tokio::spawn(async move { closing_inbox.close().await });
    release_tx.send(()).unwrap();

    assert_eq!(draining.await.unwrap().unwrap(), Some("admitted"));
    closing.await.unwrap();
    assert_eq!(inbox.len().await, 0);
    assert!(!inbox.enqueue(user("late")).await.unwrap());
}

fn user(content: &str) -> NewUserTurnInput {
    NewUserTurnInput { content: content.into(), files: Vec::new(), skills: Vec::new() }
}
