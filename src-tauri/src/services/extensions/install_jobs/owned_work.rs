//! The blocking producer holds its admission, independently of the awaiting future.
use super::super::work_supervision::ExtensionWorkServices;

pub(super) fn spawn<T: Send + 'static>(
    work: &ExtensionWorkServices,
    operation: impl FnOnce() -> T + Send + 'static,
) -> Result<tokio::sync::oneshot::Receiver<T>, String> {
    spawn_typed(work, operation).map_err(|error| error.public_code().to_string())
}

pub(super) fn spawn_typed<T: Send + 'static>(
    work: &ExtensionWorkServices,
    operation: impl FnOnce() -> T + Send + 'static,
) -> Result<
    tokio::sync::oneshot::Receiver<T>,
    super::super::work_supervision::ExtensionWorkAdmissionError,
> {
    let admission = work.try_admit_operation()?;
    let (sender, receiver) = tokio::sync::oneshot::channel();
    // No abortable async handle owns this admission: deadline expiry must remain
    // unconfirmed until the real thread exits. The app's independent exit guard remains.
    tokio::task::spawn_blocking(move || {
        let result = operation();
        drop(admission);
        let _ = sender.send(result);
    });
    Ok(receiver)
}
