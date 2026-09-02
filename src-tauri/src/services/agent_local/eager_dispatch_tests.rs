use super::collect_eager_results_with;
use crate::services::agent_local::types_tools::ToolResult;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

struct DropMarker(Arc<AtomicBool>);

impl Drop for DropMarker {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

#[test]
fn eager_capacity_uses_the_parallel_batch_authority() {
    assert_eq!(
        super::MAX_EAGER,
        super::super::tool_executor_parallel_batch::MAX_PARALLEL
    );
}

#[tokio::test]
async fn eager_children_are_aborted_with_their_collector() {
    let (tx, rx) = mpsc::unbounded_channel();
    tx.send((0, "read_file".to_string(), serde_json::json!({"path": "x"})))
        .unwrap();
    drop(tx);
    let dropped = Arc::new(AtomicBool::new(false));
    let task_dropped = Arc::clone(&dropped);
    let (started_tx, started_rx) = oneshot::channel();
    let started = Arc::new(std::sync::Mutex::new(Some(started_tx)));

    let collector = tokio::spawn(collect_eager_results_with(
        rx,
        std::path::PathBuf::from("."),
        "session".to_string(),
        "request".to_string(),
        false,
        CancellationToken::new(),
        move |_, _, _, _, _, _, _| {
            let marker = DropMarker(Arc::clone(&task_dropped));
            let signal = Arc::clone(&started);
            async move {
                if let Some(sender) = signal.lock().unwrap().take() {
                    let _ = sender.send(());
                }
                let _marker = marker;
                std::future::pending::<ToolResult>().await
            }
        },
    ));
    started_rx.await.expect("eager child started");

    collector.abort();
    let _ = collector.await;
    tokio::task::yield_now().await;

    assert!(dropped.load(Ordering::SeqCst));
}
