use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

use super::tool_bash_session::{ShellSession, ShellSessionSnapshot};

pub async fn wait(
    session: &Arc<ShellSession>,
    yield_after: Duration,
    caller_cancel: &CancellationToken,
    cancel_process_with_caller: bool,
) -> ShellSessionSnapshot {
    let deadline = tokio::time::sleep(yield_after);
    tokio::pin!(deadline);
    loop {
        let notified = session.notified();
        tokio::pin!(notified);
        if session.is_done() {
            return session.snapshot();
        }
        tokio::select! {
            _ = &mut notified => {}
            _ = &mut deadline => return session.snapshot(),
            _ = caller_cancel.cancelled() => {
                if cancel_process_with_caller {
                    session.cancel();
                }
                return wait_after_cancel(session).await;
            }
        }
    }
}

async fn wait_after_cancel(session: &ShellSession) -> ShellSessionSnapshot {
    let completed = async {
        while !session.is_done() {
            session.notified().await;
        }
    };
    let _ = tokio::time::timeout(Duration::from_secs(3), completed).await;
    session.snapshot()
}
