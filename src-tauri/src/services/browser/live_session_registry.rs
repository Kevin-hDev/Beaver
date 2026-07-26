use std::collections::VecDeque;

const MAX_LIVE_BROWSER_SESSIONS: usize = 64;

#[derive(Default)]
pub(super) struct LiveSessionRegistry {
    entries: VecDeque<String>,
}

impl LiveSessionRegistry {
    pub(super) fn activate(&mut self, session_id: &str) -> bool {
        if let Some(index) = self.entries.iter().position(|entry| entry == session_id) {
            if let Some(existing) = self.entries.remove(index) {
                self.entries.push_back(existing);
            }
            return false;
        }
        if self.entries.len() == MAX_LIVE_BROWSER_SESSIONS {
            self.entries.pop_front();
        }
        self.entries.push_back(session_id.to_owned());
        true
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }
}
