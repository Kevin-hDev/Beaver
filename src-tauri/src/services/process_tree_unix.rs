use sysinfo::{Pid, System};

const MAX_CHILDREN: usize = 256;
const MAX_DEPTH: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct UnixProcessIdentity {
    pid: Pid,
    start_time: u64,
}

impl UnixProcessIdentity {
    pub(super) fn new(pid: Pid, start_time: u64) -> Self {
        Self { pid, start_time }
    }

    pub(super) fn matches(self, pid: Pid, start_time: u64) -> bool {
        self.pid == pid && self.start_time == start_time
    }

    pub(super) fn pid(self) -> Pid {
        self.pid
    }
}

pub(super) fn collect_children(pid: u32) -> Vec<UnixProcessIdentity> {
    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let mut result = Vec::new();
    collect_children_inner(&system, Pid::from_u32(pid), &mut result, 0);
    result
}

pub(super) fn collect_group_members(group: u32) -> (Vec<UnixProcessIdentity>, bool) {
    let Ok(raw_group) = i32::try_from(group) else {
        return (Vec::new(), false);
    };
    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let mut result = Vec::new();
    let mut complete = true;
    for (pid, process) in system.processes() {
        if pid.as_u32() == group {
            continue;
        }
        let Ok(raw_pid) = i32::try_from(pid.as_u32()) else {
            continue;
        };
        if unsafe { libc::getpgid(raw_pid) } != raw_group {
            continue;
        }
        if result.len() == MAX_CHILDREN {
            complete = false;
            break;
        }
        result.push(UnixProcessIdentity::new(*pid, process.start_time()));
    }
    (result, complete)
}

pub(super) fn is_current(identity: UnixProcessIdentity) -> bool {
    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[identity.pid()]), true);
    system
        .process(identity.pid())
        .is_some_and(|process| identity.matches(identity.pid(), process.start_time()))
}

pub(super) fn is_current_group_member(identity: UnixProcessIdentity, group: u32) -> bool {
    let Ok(raw_pid) = i32::try_from(identity.pid().as_u32()) else {
        return false;
    };
    let Ok(raw_group) = i32::try_from(group) else {
        return false;
    };
    is_current(identity) && unsafe { libc::getpgid(raw_pid) } == raw_group
}

fn collect_children_inner(
    system: &System,
    parent: Pid,
    result: &mut Vec<UnixProcessIdentity>,
    depth: u32,
) {
    if depth >= MAX_DEPTH || result.len() >= MAX_CHILDREN {
        return;
    }
    for (pid, process) in system.processes() {
        if result.len() >= MAX_CHILDREN {
            return;
        }
        if process.parent() == Some(parent) {
            result.push(UnixProcessIdentity::new(*pid, process.start_time()));
            collect_children_inner(system, *pid, result, depth + 1);
        }
    }
}
