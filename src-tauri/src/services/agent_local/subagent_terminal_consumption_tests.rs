use super::subagent_registry::{self, SubagentTerminalKind};
use tokio_util::sync::CancellationToken;

fn uid() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[tokio::test]
async fn failed_terminal_cannot_be_consumed() {
    let parent = uid();
    let child = uid();
    subagent_registry::register(&parent, &child, CancellationToken::new())
        .await
        .expect("register child");
    subagent_registry::complete_child(
        &child,
        SubagentTerminalKind::ReportPersistenceFailed,
    )
    .await
    .expect("complete child");
    let failed = subagent_registry::terminal_state_for_parent(&parent)
        .await
        .expect("failed terminal");

    assert!(subagent_registry::register(&parent, &uid(), CancellationToken::new())
        .await
        .is_err());
    assert!(!subagent_registry::consume_terminal(
        &parent,
        failed.generation,
        failed.sequence,
    )
    .await);
    assert_eq!(
        subagent_registry::terminal_state_for_parent(&parent).await,
        Some(failed)
    );
}

#[tokio::test]
async fn exact_consumption_allows_a_new_generation() {
    let parent = uid();
    let first_child = uid();
    subagent_registry::register(&parent, &first_child, CancellationToken::new())
        .await
        .expect("register first child");
    subagent_registry::complete_child(&first_child, SubagentTerminalKind::ReportPersisted)
        .await
        .expect("complete first child");
    let first = subagent_registry::terminal_state_for_parent(&parent)
        .await
        .expect("first terminal");
    assert!(subagent_registry::consume_terminal(
        &parent,
        first.generation,
        first.sequence,
    )
    .await);

    let second_child = uid();
    subagent_registry::register(&parent, &second_child, CancellationToken::new())
        .await
        .expect("register second child");
    let second = subagent_registry::terminal_state_for_parent(&parent)
        .await
        .expect("second signal");
    assert_ne!(first.generation, second.generation);
    subagent_registry::unregister(&second_child).await;
}

#[tokio::test]
async fn advanced_sequence_is_not_consumed_by_stale_acknowledgement() {
    let parent = uid();
    let first_child = uid();
    let second_child = uid();
    subagent_registry::register(&parent, &first_child, CancellationToken::new())
        .await
        .expect("register first child");
    subagent_registry::register(&parent, &second_child, CancellationToken::new())
        .await
        .expect("register second child");
    subagent_registry::complete_child(&first_child, SubagentTerminalKind::ReportPersisted)
        .await
        .expect("complete first child");
    let stale = subagent_registry::terminal_state_for_parent(&parent)
        .await
        .expect("first terminal");
    subagent_registry::complete_child(
        &second_child,
        SubagentTerminalKind::ReportPersistenceFailed,
    )
    .await
    .expect("complete second child");

    assert!(!subagent_registry::consume_terminal(
        &parent,
        stale.generation,
        stale.sequence,
    )
    .await);
    let advanced = subagent_registry::terminal_state_for_parent(&parent)
        .await
        .expect("advanced terminal");
    assert_eq!(advanced.generation, stale.generation);
    assert_eq!(advanced.sequence, stale.sequence + 1);
    assert!(advanced.report_persistence_failed);
}
