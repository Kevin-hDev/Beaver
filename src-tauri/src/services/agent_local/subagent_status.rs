pub const RUNNING: &str = "running";
pub const COMPLETED: &str = "completed";
pub const FAILED: &str = "failed";
pub const CANCELLED: &str = "cancelled";
pub const INTERRUPTED: &str = "interrupted";

const ALL: [&str; 5] = [RUNNING, COMPLETED, FAILED, CANCELLED, INTERRUPTED];

pub fn is_valid(status: &str) -> bool {
    ALL.contains(&status)
}
