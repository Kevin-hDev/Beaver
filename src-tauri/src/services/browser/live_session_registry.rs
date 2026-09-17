use super::session_model::SessionModel;
use std::collections::VecDeque;

const MAX_LIVE_BROWSER_SESSIONS: usize = 64;

#[derive(Default)]
pub(super) struct LiveSessionRegistry {
    entries: VecDeque<LiveSession>,
}

struct LiveSession {
    id: String,
    model: SessionModel,
}

impl LiveSessionRegistry {
    pub(super) fn contains(&self, session_id: &str) -> bool {
        self.entries.iter().any(|entry| entry.id == session_id)
    }

    pub(super) fn get_mut(&mut self, session_id: &str) -> Option<&mut SessionModel> {
        if let Some(index) = self.entries.iter().position(|entry| entry.id == session_id) {
            if let Some(existing) = self.entries.remove(index) {
                self.entries.push_back(existing);
            }
            return self.entries.back_mut().map(|entry| &mut entry.model);
        }
        None
    }

    pub(super) fn insert(&mut self, session_id: String, model: SessionModel) {
        if self.entries.len() == MAX_LIVE_BROWSER_SESSIONS {
            self.entries.pop_front();
        }
        self.entries.push_back(LiveSession {
            id: session_id,
            model,
        });
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }
}
