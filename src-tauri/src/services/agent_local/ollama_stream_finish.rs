use super::types_stream::{StreamOutcome, StreamResult};

pub(super) fn into_outcome(result: StreamResult, interrupted: bool) -> StreamOutcome {
    if interrupted {
        StreamOutcome::InterruptedForCompression(result)
    } else {
        StreamOutcome::Completed(result)
    }
}
