use futures_util::future::{BoxFuture, FutureExt, Shared};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::oneshot;

pub struct OAuthCompletion<T>
where
    T: Clone + Send + 'static,
{
    future: Shared<BoxFuture<'static, Option<T>>>,
    finished: Arc<AtomicBool>,
}

impl<T> Clone for OAuthCompletion<T>
where
    T: Clone + Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            future: self.future.clone(),
            finished: Arc::clone(&self.finished),
        }
    }
}

pub struct OAuthCompletionOwner<T> {
    sender: Option<oneshot::Sender<T>>,
    finished: Arc<AtomicBool>,
}

impl<T> OAuthCompletion<T>
where
    T: Clone + Send + 'static,
{
    pub fn channel() -> (OAuthCompletionOwner<T>, Self) {
        let (sender, receiver) = oneshot::channel();
        let finished = Arc::new(AtomicBool::new(false));
        let future = async move { receiver.await.ok() }.boxed().shared();
        (
            OAuthCompletionOwner {
                sender: Some(sender),
                finished: Arc::clone(&finished),
            },
            Self { future, finished },
        )
    }

    pub async fn wait(&self) -> Option<T> {
        self.future.clone().await
    }

    pub fn is_finished(&self) -> bool {
        self.finished.load(Ordering::Acquire)
    }
}

impl<T> OAuthCompletionOwner<T> {
    pub fn complete(mut self, value: T) {
        self.finished.store(true, Ordering::Release);
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(value);
        }
    }
}

impl<T> Drop for OAuthCompletionOwner<T> {
    fn drop(&mut self) {
        self.finished.store(true, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::OAuthCompletion;
    use std::time::Duration;

    #[tokio::test]
    async fn owned_work_drop_completes_without_polling() {
        let (owner, completion) = OAuthCompletion::<()>::channel();
        let work = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(20)).await;
            drop(owner);
        });

        tokio::time::timeout(Duration::from_millis(200), completion.wait())
            .await
            .expect("completion must follow the owned work");
        work.await.unwrap();
    }
}
