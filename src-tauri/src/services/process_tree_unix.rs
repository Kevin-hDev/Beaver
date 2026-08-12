use sysinfo::{Pid, System};

const MAX_CHILDREN: usize = 256;
const MAX_DEPTH: u32 = 10;

pub(super) fn collect_children(pid: u32) -> Vec<Pid> {
    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let mut result = Vec::new();
    collect_children_inner(&system, Pid::from_u32(pid), &mut result, 0);
    result
}

fn collect_children_inner(system: &System, parent: Pid, result: &mut Vec<Pid>, depth: u32) {
    if depth >= MAX_DEPTH || result.len() >= MAX_CHILDREN {
        return;
    }
    for (pid, process) in system.processes() {
        if result.len() >= MAX_CHILDREN {
            return;
        }
        if process.parent() == Some(parent) {
            result.push(*pid);
            collect_children_inner(system, *pid, result, depth + 1);
        }
    }
}
