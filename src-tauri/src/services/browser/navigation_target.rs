#[derive(Default)]
pub(super) struct NavigationTarget {
    current: Option<String>,
    pending: Option<String>,
}

impl NavigationTarget {
    pub(super) fn request(&mut self, url: &str) {
        self.pending = Some(url.to_owned());
    }

    pub(super) fn observe(&mut self, url: &str) {
        self.current = Some(url.to_owned());
        if self.pending.as_deref() == Some(url) {
            self.pending = None;
        }
    }

    pub(super) fn take_pending(&mut self) -> Option<String> {
        self.pending.take()
    }
}
