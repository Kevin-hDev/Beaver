use super::process_manager::{McpProcessService, PoolEntry};
use super::work_supervision::McpWorkServices;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

fn service() -> McpProcessService {
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    McpProcessService {
        pool: Mutex::new(HashMap::new()),
        spawn_owner: tokio::sync::Mutex::new(()),
        work: McpWorkServices::new(coordinator.work_supervisor()),
    }
}

async fn entry(service: &McpProcessService) -> (PoolEntry, u32) {
    let program = which::which("node").unwrap();
    let (child, handle) = super::process_spawn::spawn_program(
        &program,
        &["-e".into(), "process.stdin.resume()".into()],
        &[],
    )
    .await
    .unwrap();
    let pid = child.id().unwrap();
    let admission = service.work.try_admit_process().unwrap();
    (
        PoolEntry {
            child,
            handle,
            last_used: Instant::now(),
            _admission: admission,
        },
        pid,
    )
}

#[tokio::test]
async fn published_pool_is_drained_even_while_spawn_owner_is_locked() {
    let service = service();
    let (entry, _pid) = entry(&service).await;
    service.lock_pool().insert("fixture".into(), entry);
    let _owner = service.spawn_owner.lock().await;

    assert!(
        !service
            .stop_and_wait(Instant::now() + Duration::from_millis(20))
            .await
    );
    assert!(service.lock_pool().is_empty());
}

#[tokio::test]
async fn locked_stdin_cannot_extend_the_shared_deadline() {
    let service = service();
    let (entry, _) = entry(&service).await;
    let handle = entry.handle.clone();
    let _stdin = handle.stdin.lock().await;
    let started = Instant::now();

    assert!(
        !super::process_pool::terminate_entry(entry, Instant::now() + Duration::from_millis(20))
            .await
    );
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[tokio::test]
async fn stop_and_wait_closes_admission_terminates_child_and_drains_work() {
    let service = service();
    let (entry, pid) = entry(&service).await;
    service.lock_pool().insert("complete-stop".into(), entry);
    let (drained_tx, drained_rx) = tokio::sync::oneshot::channel();
    service
        .work
        .try_admit()
        .unwrap()
        .spawn(move |cancel| async move {
            cancel.cancelled().await;
            let _ = drained_tx.send(());
        })
        .unwrap();

    assert!(
        service
            .stop_and_wait(Instant::now() + Duration::from_secs(2))
            .await
    );
    drained_rx.await.expect("operation drained");
    assert!(service.work.try_admit().is_err());
    let mut processes = sysinfo::System::new();
    processes.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    assert!(processes.process(sysinfo::Pid::from_u32(pid)).is_none());
}
