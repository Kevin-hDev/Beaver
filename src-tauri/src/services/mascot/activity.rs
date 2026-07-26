use serde::Serialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};

const MAX_ACTIVE_SESSIONS: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MascotAnimation {
    Idle,
    Thinking,
    ExploreBook,
    WorkLaptop,
    Waiting,
    Success,
    Failed,
    Alert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MascotStatePayload {
    pub animation: MascotAnimation,
    pub revision: u64,
}

#[derive(Debug, Clone, Copy)]
struct SessionActivity {
    animation: MascotAnimation,
    updated_at: Instant,
    expires_at: Option<Instant>,
    fallback: Option<MascotAnimation>,
}

pub struct ActivityArbiter {
    sessions: HashMap<String, SessionActivity>,
    current: MascotAnimation,
    revision: u64,
}

impl Default for ActivityArbiter {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
            current: MascotAnimation::Idle,
            revision: 0,
        }
    }
}

impl ActivityArbiter {
    pub fn update(
        &mut self,
        session_id: &str,
        animation: MascotAnimation,
        ttl: Option<Duration>,
        resume_previous: bool,
        now: Instant,
    ) -> Option<MascotStatePayload> {
        if session_id.is_empty() || session_id.len() > 128 {
            return None;
        }
        if !self.sessions.contains_key(session_id) && self.sessions.len() >= MAX_ACTIVE_SESSIONS {
            self.evict_oldest();
        }
        let fallback = if resume_previous {
            self.sessions
                .get(session_id)
                .map(|activity| activity.animation)
        } else {
            None
        };
        self.sessions.insert(
            session_id.to_string(),
            SessionActivity {
                animation,
                updated_at: now,
                expires_at: ttl.and_then(|duration| now.checked_add(duration)),
                fallback,
            },
        );
        self.recompute(now)
    }

    pub fn remove(&mut self, session_id: &str, now: Instant) -> Option<MascotStatePayload> {
        self.sessions.remove(session_id);
        self.recompute(now)
    }

    pub fn refresh(&mut self, now: Instant) -> Option<MascotStatePayload> {
        self.recompute(now)
    }

    pub fn state(&self) -> MascotStatePayload {
        MascotStatePayload {
            animation: self.current,
            revision: self.revision,
        }
    }

    fn recompute(&mut self, now: Instant) -> Option<MascotStatePayload> {
        self.sessions.retain(|_, activity| {
            if activity.expires_at.is_none_or(|expiry| expiry > now) {
                return true;
            }
            if let Some(fallback) = activity.fallback.take() {
                activity.animation = fallback;
                activity.expires_at = None;
                return true;
            }
            false
        });
        let next = self
            .sessions
            .values()
            .max_by_key(|activity| (priority(activity.animation), activity.updated_at))
            .map(|activity| activity.animation)
            .unwrap_or(MascotAnimation::Idle);
        if next == self.current {
            return None;
        }
        self.current = next;
        self.revision = self.revision.saturating_add(1);
        Some(self.state())
    }

    fn evict_oldest(&mut self) {
        let oldest = self
            .sessions
            .iter()
            .min_by_key(|(session_id, activity)| {
                (
                    priority(activity.animation),
                    activity.updated_at,
                    session_id.as_str(),
                )
            })
            .map(|(session_id, _)| session_id.clone());
        if let Some(session_id) = oldest {
            self.sessions.remove(&session_id);
        }
    }

    #[cfg(test)]
    fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

fn priority(animation: MascotAnimation) -> u8 {
    match animation {
        MascotAnimation::Failed => 6,
        MascotAnimation::Waiting => 5,
        MascotAnimation::Success => 4,
        MascotAnimation::Alert => 3,
        MascotAnimation::ExploreBook | MascotAnimation::WorkLaptop => 2,
        MascotAnimation::Thinking => 1,
        MascotAnimation::Idle => 0,
    }
}

#[cfg(test)]
#[path = "activity_tests.rs"]
mod tests;
