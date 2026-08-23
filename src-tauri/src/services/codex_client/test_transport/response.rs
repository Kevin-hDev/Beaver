use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use reqwest::{Response, StatusCode};

pub(in crate::services::codex_client) struct ObservedResponse {
    response: Option<Response>,
    dropped: Option<Arc<AtomicBool>>,
}

impl ObservedResponse {
    pub(in crate::services::codex_client) fn status(&self) -> StatusCode {
        self.response.as_ref().expect("response present").status()
    }

    pub(in crate::services::codex_client) fn into_inner(mut self) -> Response {
        self.dropped = None;
        self.response.take().expect("response present")
    }
}

impl Drop for ObservedResponse {
    fn drop(&mut self) {
        drop(self.response.take());
        if let Some(observer) = &self.dropped {
            observer.store(true, Ordering::SeqCst);
        }
    }
}

pub(super) fn observe(response: Response) -> ObservedResponse {
    let dropped = super::ACTIVE_SCENARIO
        .try_with(|context| Arc::clone(&context.initial_response_dropped))
        .ok();
    ObservedResponse {
        response: Some(response),
        dropped,
    }
}
