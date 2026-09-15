use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;

use super::{
    checkpoint::should_checkpoint, checkpoint_path, partial_path, save_checkpoint,
    transfer::download_with_client, Checkpoint, HttpValidator, VoiceArchive, VoiceCatalogEntry,
    VoiceEngine, VoiceLanguageMode, VoiceLicense, VoiceModelFile, VoiceModelRole,
};

const BODY: &[u8] = b"hello resilient voice";

fn entry(url: String) -> VoiceCatalogEntry {
    VoiceCatalogEntry {
        id: "test-model".into(),
        role: VoiceModelRole::Asr,
        engine: VoiceEngine::NemoTransducer,
        revision: "a".repeat(40),
        archive: VoiceArchive {
            url,
            bytes: BODY.len() as u64,
            sha256: "b".repeat(64),
        },
        manifest_url: "https://github.com/test".into(),
        installed_bytes: BODY.len() as u64,
        files: vec![VoiceModelFile {
            path: "model.onnx".into(),
            bytes: BODY.len() as u64,
            sha256: "b".repeat(64),
        }],
        languages: vec![],
        dialects: vec![],
        language_mode: VoiceLanguageMode::AutomaticOnly,
        license: VoiceLicense {
            spdx: "MIT".into(),
            url: "https://github.com/test/license".into(),
        },
    }
}

async fn server(
    responses: Vec<Vec<u8>>,
) -> (String, Arc<Mutex<Vec<String>>>, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&requests);
    let task = tokio::spawn(async move {
        for response in responses {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = vec![0; 4096];
            let count = socket.read(&mut request).await.unwrap();
            seen.lock()
                .unwrap()
                .push(String::from_utf8_lossy(&request[..count]).into_owned());
            socket.write_all(&response).await.unwrap();
        }
    });
    (format!("http://{address}/model"), requests, task)
}

fn response(status: &str, body: &[u8], extra_headers: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\n{extra_headers}Connection: close\r\n\r\n",
        body.len()
    )
    .bytes()
    .chain(body.iter().copied())
    .collect()
}

#[tokio::test]
async fn resumes_from_the_durable_byte_with_if_range() {
    let remaining = &BODY[5..];
    let reply = response(
        "206 Partial Content",
        remaining,
        &format!(
            "Content-Range: bytes 5-{}/{}\r\nETag: \"v1\"\r\n",
            BODY.len() - 1,
            BODY.len()
        ),
    );
    let (url, requests, task) = server(vec![reply]).await;
    let data = tempfile::tempdir().unwrap();
    let entry = entry(url);
    tokio::fs::create_dir_all(partial_path(data.path(), &entry.id).parent().unwrap())
        .await
        .unwrap();
    tokio::fs::write(partial_path(data.path(), &entry.id), &BODY[..5])
        .await
        .unwrap();
    let mut checkpoint = Checkpoint::new(
        entry.id.clone(),
        entry.id.clone(),
        entry.revision.clone(),
        entry.archive.bytes,
        entry.archive.sha256.clone(),
    )
    .unwrap();
    checkpoint.durable_bytes = 5;
    checkpoint.validator = Some(HttpValidator::Etag("\"v1\"".into()));
    save_checkpoint(&checkpoint_path(data.path(), &entry.id), &checkpoint).unwrap();

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let path = download_with_client(
        &entry,
        &entry.id,
        data.path(),
        &CancellationToken::new(),
        &client,
        true,
        &mut |_, _| {},
    )
    .await
    .unwrap();
    task.await.unwrap();

    assert_eq!(tokio::fs::read(path).await.unwrap(), BODY);
    let request = requests.lock().unwrap()[0].to_ascii_lowercase();
    assert!(request.contains("range: bytes=5-"));
    assert!(request.contains("if-range: \"v1\""));
}

#[tokio::test]
async fn ignored_range_restarts_without_appending_duplicate_bytes() {
    let (url, _, task) = server(vec![response("200 OK", BODY, "ETag: \"v2\"\r\n")]).await;
    let data = tempfile::tempdir().unwrap();
    let entry = entry(url);
    tokio::fs::create_dir_all(partial_path(data.path(), &entry.id).parent().unwrap())
        .await
        .unwrap();
    tokio::fs::write(partial_path(data.path(), &entry.id), &BODY[..5])
        .await
        .unwrap();
    let mut checkpoint = Checkpoint::new(
        entry.id.clone(),
        entry.id.clone(),
        entry.revision.clone(),
        entry.archive.bytes,
        entry.archive.sha256.clone(),
    )
    .unwrap();
    checkpoint.durable_bytes = 5;
    checkpoint.validator = Some(HttpValidator::Etag("\"v1\"".into()));
    save_checkpoint(&checkpoint_path(data.path(), &entry.id), &checkpoint).unwrap();
    let client = reqwest::Client::new();

    let path = download_with_client(
        &entry,
        &entry.id,
        data.path(),
        &CancellationToken::new(),
        &client,
        true,
        &mut |_, _| {},
    )
    .await
    .unwrap();
    task.await.unwrap();
    assert_eq!(tokio::fs::read(path).await.unwrap(), BODY);
}

#[tokio::test]
async fn a_complete_durable_partial_is_reused_without_network_access() {
    let data = tempfile::tempdir().unwrap();
    let entry = entry("http://127.0.0.1:9/must-not-be-called".into());
    let partial = partial_path(data.path(), &entry.id);
    tokio::fs::create_dir_all(partial.parent().unwrap())
        .await
        .unwrap();
    tokio::fs::write(&partial, BODY).await.unwrap();
    let mut checkpoint = Checkpoint::new(
        entry.id.clone(),
        entry.id.clone(),
        entry.revision.clone(),
        entry.archive.bytes,
        entry.archive.sha256.clone(),
    )
    .unwrap();
    checkpoint.durable_bytes = entry.archive.bytes;
    save_checkpoint(&checkpoint_path(data.path(), &entry.id), &checkpoint).unwrap();

    let path = download_with_client(
        &entry,
        &entry.id,
        data.path(),
        &CancellationToken::new(),
        &reqwest::Client::new(),
        true,
        &mut |_, _| {},
    )
    .await
    .unwrap();

    assert_eq!(path, partial);
    assert_eq!(tokio::fs::read(path).await.unwrap(), BODY);
}

#[tokio::test(start_paused = true)]
async fn cancellation_interrupts_retry_backoff() {
    let data = tempfile::tempdir().unwrap();
    let entry = entry("http://127.0.0.1:9/unavailable".into());
    let cancel = CancellationToken::new();
    let task_cancel = cancel.clone();
    let client = reqwest::Client::new();
    let mut progress = |_, _| {};
    let future = download_with_client(
        &entry,
        &entry.id,
        data.path(),
        &cancel,
        &client,
        true,
        &mut progress,
    );
    tokio::pin!(future);
    tokio::select! {
        _ = &mut future => panic!("retry returned before cancellation"),
        _ = tokio::time::sleep(std::time::Duration::from_millis(1)) => task_cancel.cancel(),
    }
    assert_eq!(future.await.unwrap_err(), "cancelled");
}

#[test]
fn checkpoint_uses_the_first_byte_or_time_threshold_reached() {
    assert!(!should_checkpoint(
        4 * 1024 * 1024 - 1,
        0,
        std::time::Duration::from_secs(4)
    ));
    assert!(should_checkpoint(
        4 * 1024 * 1024,
        0,
        std::time::Duration::from_secs(1)
    ));
    assert!(should_checkpoint(1, 0, std::time::Duration::from_secs(5)));
    assert!(!should_checkpoint(0, 0, std::time::Duration::from_secs(6)));
}

#[tokio::test(start_paused = true)]
async fn three_failed_attempts_leave_a_manually_resumable_checkpoint() {
    let cutoff = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\nhello",
        BODY.len()
    )
    .into_bytes();
    let (url, requests, task) = server(vec![cutoff.clone(), cutoff.clone(), cutoff]).await;
    let data = tempfile::tempdir().unwrap();
    let entry = entry(url);
    let error = download_with_client(
        &entry,
        &entry.id,
        data.path(),
        &CancellationToken::new(),
        &reqwest::Client::new(),
        true,
        &mut |_, _| {},
    )
    .await
    .unwrap_err();
    task.await.unwrap();

    assert_eq!(error, "model-download-network-failed");
    assert_eq!(requests.lock().unwrap().len(), 3);
    assert_eq!(
        super::discover_checkpoints(data.path())
            .remove(0)
            .durable_bytes,
        5
    );
}

#[tokio::test(start_paused = true)]
async fn changed_validator_discards_the_old_range_before_retrying() {
    let remaining = &BODY[5..];
    let changed = response(
        "206 Partial Content",
        remaining,
        &format!(
            "Content-Range: bytes 5-{}/{}\r\nETag: \"v2\"\r\n",
            BODY.len() - 1,
            BODY.len()
        ),
    );
    let (url, requests, task) =
        server(vec![changed, response("200 OK", BODY, "ETag: \"v2\"\r\n")]).await;
    let data = tempfile::tempdir().unwrap();
    let entry = entry(url);
    let partial = partial_path(data.path(), &entry.id);
    tokio::fs::create_dir_all(partial.parent().unwrap())
        .await
        .unwrap();
    tokio::fs::write(&partial, &BODY[..5]).await.unwrap();
    let mut checkpoint = Checkpoint::new(
        entry.id.clone(),
        entry.id.clone(),
        entry.revision.clone(),
        entry.archive.bytes,
        entry.archive.sha256.clone(),
    )
    .unwrap();
    checkpoint.durable_bytes = 5;
    checkpoint.validator = Some(HttpValidator::Etag("\"v1\"".into()));
    save_checkpoint(&checkpoint_path(data.path(), &entry.id), &checkpoint).unwrap();

    let path = download_with_client(
        &entry,
        &entry.id,
        data.path(),
        &CancellationToken::new(),
        &reqwest::Client::new(),
        true,
        &mut |_, _| {},
    )
    .await
    .unwrap();
    task.await.unwrap();

    assert_eq!(tokio::fs::read(path).await.unwrap(), BODY);
    let seen = requests.lock().unwrap();
    assert!(seen[0].to_ascii_lowercase().contains("range: bytes=5-"));
    assert!(!seen[1].to_ascii_lowercase().contains("range:"));
}
