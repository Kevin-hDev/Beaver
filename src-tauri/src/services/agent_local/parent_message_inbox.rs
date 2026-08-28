use tokio::sync::watch;

pub struct ParentMessageInbox {
    closed: watch::Sender<bool>,
}

impl ParentMessageInbox {
    pub fn new() -> Self {
        let (closed, _) = watch::channel(false);
        Self { closed }
    }

    pub fn subscribe(&self) -> watch::Receiver<bool> {
        self.closed.subscribe()
    }

    pub async fn close(&self) {
        self.closed.send_replace(true);
    }

    #[cfg(test)]
    pub fn is_closed(&self) -> bool {
        *self.closed.borrow()
    }
}

#[cfg(test)]
mod tests {
    use super::ParentMessageInbox;

    #[tokio::test]
    async fn close_notifies_waiters_once() {
        let inbox = ParentMessageInbox::new();
        let mut signal = inbox.subscribe();

        inbox.close().await;

        signal.changed().await.unwrap();
        assert!(*signal.borrow());
        assert!(inbox.is_closed());
    }
}
