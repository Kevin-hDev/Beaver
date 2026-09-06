pub(super) const MAX_ACTIVE: usize = 8;
pub(super) const MAX_RECENT: usize = 32;
// Revisions cross the IPC boundary as JavaScript integers.
pub(super) const MAX_REVISION: u64 = (1_u64 << 53) - 1;
pub(super) const INVALID: &str = "extension-install-invalid";
pub(super) const BUSY: &str = "extension-install-busy";
pub(super) const UNAVAILABLE: &str = "extension-install-unavailable";
pub(super) const FAILED: &str = "extension-install-failed";
pub(super) const INSUFFICIENT_SPACE: &str = "extension-install-insufficient-space";
pub(crate) const CHANGED_EVENT: &str = "extension-installs-changed";
pub(super) const RECOVERY_UNAVAILABLE: &str = "extension-install-recovery-unavailable";
