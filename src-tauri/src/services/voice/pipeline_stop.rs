use super::capture::activity::CaptureStopReason;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StopAction {
    Discard,
    Validate,
}

pub fn stop_action(reason: CaptureStopReason) -> StopAction {
    match reason {
        CaptureStopReason::NoSpeech => StopAction::Discard,
        CaptureStopReason::Silence
        | CaptureStopReason::DurationLimit
        | CaptureStopReason::Disconnected
        | CaptureStopReason::HiddenOrLocked => StopAction::Validate,
    }
}
