use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

const MAX_TREE_PROCESSES: usize = 256;

struct ProcessSnapshot(HANDLE);

impl ProcessSnapshot {
    fn capture() -> Option<Self> {
        let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        (handle != INVALID_HANDLE_VALUE).then_some(Self(handle))
    }

    fn for_each(&self, mut visit: impl FnMut(&PROCESSENTRY32W)) {
        // SAFETY: PROCESSENTRY32W is a plain Windows API record initialized as required by ToolHelp.
        let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        if unsafe { Process32FirstW(self.0, &mut entry) } == 0 {
            return;
        }
        loop {
            visit(&entry);
            if unsafe { Process32NextW(self.0, &mut entry) } == 0 {
                break;
            }
        }
    }
}

impl Drop for ProcessSnapshot {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}

pub(super) fn terminate_tree(root_pid: u32, deadline: std::time::Instant) {
    if root_pid < 2 || !crate::services::owned_process::is_confined(root_pid) {
        return;
    }
    let mut members = Vec::with_capacity(MAX_TREE_PROCESSES);
    members.push(root_pid);
    if let Some(snapshot) = ProcessSnapshot::capture() {
        collect_descendants(&snapshot, &mut members);
    }
    // Children go first so they cannot keep inherited pipes open while their parent is reaped.
    members.reverse();
    let expected = members.len();
    let reaped = crate::services::owned_process::terminate_confined(&members, deadline);
    if reaped != expected {
        ::log::warn!("[process] balayage Windows incomplet: {reaped}/{expected}");
    }
}

fn collect_descendants(snapshot: &ProcessSnapshot, members: &mut Vec<u32>) {
    loop {
        let before = members.len();
        snapshot.for_each(|entry| {
            if members.len() == MAX_TREE_PROCESSES {
                return;
            }
            let pid = entry.th32ProcessID;
            let parent_pid = entry.th32ParentProcessID;
            if pid >= 2
                && members.contains(&parent_pid)
                && !members.contains(&pid)
                && crate::services::owned_process::is_confined(pid)
            {
                members.push(pid);
            }
        });
        if members.len() == before || members.len() == MAX_TREE_PROCESSES {
            break;
        }
    }
    if members.len() == MAX_TREE_PROCESSES {
        ::log::warn!("[process] arbre Windows tronque a {MAX_TREE_PROCESSES} processus");
    }
}
