use super::types_ollama::StreamResult;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use tokio_util::sync::CancellationToken;

const MAX_SCRIPTS: usize = 8;
const MAX_RESPONSES: usize = 256;

struct Script {
    request_id: String,
    responses: VecDeque<StreamResult>,
    calls: Arc<AtomicUsize>,
}

static SCRIPTS: LazyLock<Mutex<VecDeque<Script>>> = LazyLock::new(|| Mutex::new(VecDeque::new()));

pub struct ScriptHandle {
    request_id: String,
    calls: Arc<AtomicUsize>,
}

impl ScriptHandle {
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl Drop for ScriptHandle {
    fn drop(&mut self) {
        lock_scripts().retain(|script| script.request_id != self.request_id);
    }
}

pub fn install(request_id: &str, responses: Vec<StreamResult>) -> ScriptHandle {
    assert!(responses.len() <= MAX_RESPONSES);
    let calls = Arc::new(AtomicUsize::new(0));
    let mut scripts = lock_scripts();
    assert!(scripts.len() < MAX_SCRIPTS);
    assert!(!scripts.iter().any(|script| script.request_id == request_id));
    scripts.push_back(Script {
        request_id: request_id.to_string(),
        responses: responses.into(),
        calls: Arc::clone(&calls),
    });
    ScriptHandle {
        request_id: request_id.to_string(),
        calls,
    }
}

pub fn next(request_id: &str, cancel: &CancellationToken) -> Option<Result<StreamResult, String>> {
    let mut scripts = lock_scripts();
    let position = scripts
        .iter()
        .position(|script| script.request_id == request_id)?;
    let script = scripts.get_mut(position)?;
    if let Some(result) = script.responses.pop_front() {
        script.calls.fetch_add(1, Ordering::SeqCst);
        return Some(Ok(result));
    }
    scripts.remove(position);
    cancel.cancel();
    Some(Err("Annulé".to_string()))
}

fn lock_scripts() -> std::sync::MutexGuard<'static, VecDeque<Script>> {
    SCRIPTS.lock().unwrap_or_else(|error| error.into_inner())
}
