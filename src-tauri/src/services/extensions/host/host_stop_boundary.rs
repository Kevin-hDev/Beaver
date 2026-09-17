use std::future::Future;

pub(super) async fn after_confirmed_stop<T, E>(
    stopped: bool,
    failure: E,
    followup: impl Future<Output = Result<T, E>>,
) -> Result<T, E> {
    if !stopped {
        return Err(failure);
    }
    followup.await
}
