use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

const SHARED_SCHEMA: u32 = 1;
const MAILBOX_EMPTY: u32 = 0;
const MAILBOX_WRITING: u32 = 1;
const MAILBOX_PUBLISHED: u32 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::services::browser) enum CefSharedLayoutError {
    Invalid,
    AlreadyPublished,
    Unpublished,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CefMailboxSnapshot {
    pub(super) generation: u64,
    pub(super) pid: u32,
    pub(super) started_at: u64,
    pub(super) native_group: u32,
}

#[repr(C, align(64))]
pub(in crate::services::browser) struct CefMailboxPage {
    schema: AtomicU32,
    published: AtomicU32,
    generation: AtomicU64,
    pub(super) pid: AtomicU32,
    pub(super) native_group: AtomicU32,
    pub(super) started_at: AtomicU64,
}

impl CefMailboxPage {
    pub(super) fn new() -> Self {
        Self {
            schema: AtomicU32::new(SHARED_SCHEMA),
            published: AtomicU32::new(MAILBOX_EMPTY),
            generation: AtomicU64::new(0),
            pid: AtomicU32::new(0),
            native_group: AtomicU32::new(0),
            started_at: AtomicU64::new(0),
        }
    }

    pub(super) fn publish(
        &self,
        generation: u64,
        pid: u32,
        started_at: u64,
        native_group: u32,
    ) -> Result<(), CefSharedLayoutError> {
        if generation == 0 || pid == 0 || started_at == 0 {
            return Err(CefSharedLayoutError::Invalid);
        }
        self.published
            .compare_exchange(
                MAILBOX_EMPTY,
                MAILBOX_WRITING,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map_err(|_| CefSharedLayoutError::AlreadyPublished)?;
        self.generation.store(generation, Ordering::Relaxed);
        self.pid.store(pid, Ordering::Relaxed);
        self.started_at.store(started_at, Ordering::Relaxed);
        self.native_group.store(native_group, Ordering::Relaxed);
        self.published.store(MAILBOX_PUBLISHED, Ordering::Release);
        Ok(())
    }

    pub(super) fn snapshot(&self) -> Result<CefMailboxSnapshot, CefSharedLayoutError> {
        if self.schema.load(Ordering::Acquire) != SHARED_SCHEMA
            || self.published.load(Ordering::Acquire) != MAILBOX_PUBLISHED
        {
            return Err(CefSharedLayoutError::Unpublished);
        }
        let snapshot = CefMailboxSnapshot {
            generation: self.generation.load(Ordering::Relaxed),
            pid: self.pid.load(Ordering::Relaxed),
            started_at: self.started_at.load(Ordering::Relaxed),
            native_group: self.native_group.load(Ordering::Relaxed),
        };
        if snapshot.generation == 0 || snapshot.pid == 0 || snapshot.started_at == 0 {
            Err(CefSharedLayoutError::Invalid)
        } else {
            Ok(snapshot)
        }
    }
}

impl Default for CefMailboxPage {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CefControlSnapshot {
    pub(super) generation: u64,
    pub(super) closing: bool,
    pub(super) deadline_ticks: u64,
}

#[repr(C, align(64))]
pub(in crate::services::browser) struct CefControlPage {
    schema: AtomicU32,
    closing: AtomicU32,
    generation: AtomicU64,
    deadline_ticks: AtomicU64,
}

impl CefControlPage {
    pub(super) fn new(generation: u64) -> Result<Self, CefSharedLayoutError> {
        if generation == 0 {
            return Err(CefSharedLayoutError::Invalid);
        }
        Ok(Self {
            schema: AtomicU32::new(SHARED_SCHEMA),
            closing: AtomicU32::new(0),
            generation: AtomicU64::new(generation),
            deadline_ticks: AtomicU64::new(0),
        })
    }

    pub(super) fn begin_closing(&self, deadline_ticks: u64) -> Result<(), CefSharedLayoutError> {
        if deadline_ticks == 0 {
            return Err(CefSharedLayoutError::Invalid);
        }
        self.deadline_ticks.store(deadline_ticks, Ordering::Relaxed);
        self.closing.store(1, Ordering::Release);
        Ok(())
    }

    pub(super) fn snapshot(&self) -> Result<CefControlSnapshot, CefSharedLayoutError> {
        if self.schema.load(Ordering::Acquire) != SHARED_SCHEMA {
            return Err(CefSharedLayoutError::Invalid);
        }
        let closing = self.closing.load(Ordering::Acquire) == 1;
        let snapshot = CefControlSnapshot {
            generation: self.generation.load(Ordering::Relaxed),
            closing,
            deadline_ticks: self.deadline_ticks.load(Ordering::Relaxed),
        };
        if snapshot.generation == 0 || (snapshot.closing && snapshot.deadline_ticks == 0) {
            Err(CefSharedLayoutError::Invalid)
        } else {
            Ok(snapshot)
        }
    }
}

#[repr(C, align(64))]
pub(in crate::services::browser) struct CefEventPage {
    pub(super) schema: AtomicU32,
    signaled: AtomicU32,
}

impl CefEventPage {
    pub(super) fn new() -> Self {
        Self {
            schema: AtomicU32::new(SHARED_SCHEMA),
            signaled: AtomicU32::new(0),
        }
    }

    pub(super) fn signal(&self) {
        self.signaled.store(1, Ordering::Release);
    }

    pub(super) fn is_signaled(&self) -> Result<bool, CefSharedLayoutError> {
        if self.schema.load(Ordering::Acquire) != SHARED_SCHEMA {
            Err(CefSharedLayoutError::Invalid)
        } else {
            Ok(self.signaled.load(Ordering::Acquire) == 1)
        }
    }
}

impl Default for CefEventPage {
    fn default() -> Self {
        Self::new()
    }
}
