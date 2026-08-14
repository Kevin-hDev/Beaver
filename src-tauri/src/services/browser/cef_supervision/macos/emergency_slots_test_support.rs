use super::super::{CefAuthorityTable, CefIpcNames, CefProcessRole, CefPublication};
use super::emergency_slots::MacEmergencySlots;
use super::{MacProcessIdentity, MacPublicationObjects};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct AdmittedKey {
    pub(super) slot: usize,
    pub(super) generation: u64,
    pub(super) pid: u32,
}

pub(super) struct EmergencySlotsFixture {
    pub(super) slots: MacEmergencySlots,
    table: CefAuthorityTable,
    root: tempfile::TempDir,
    children: Vec<Child>,
}

impl EmergencySlotsFixture {
    pub(super) fn new() -> Self {
        Self {
            slots: MacEmergencySlots::new(),
            table: CefAuthorityTable::new(),
            root: tempfile::tempdir().expect("temporary CEF root"),
            children: Vec::new(),
        }
    }

    pub(super) fn install_grouped_sleep(&mut self) -> AdmittedKey {
        let child = grouped_sleep();
        let pid = child.id();
        let identity = MacProcessIdentity::read(pid).expect("child identity");
        let reservation = self
            .table
            .try_reserve(CefProcessRole::Helper)
            .expect("CEF reservation");
        let slot = reservation.marker().slot();
        let generation = reservation.marker().generation();
        let publication =
            CefPublication::from_marker(reservation.marker(), pid).expect("CEF publication");
        let claim = self.table.claim(&publication).expect("CEF claim");
        let admission = claim.admit().expect("CEF admission");
        let names = CefIpcNames::from_marker(reservation.marker()).expect("CEF names");
        let objects = Arc::new(
            MacPublicationObjects::create(self.root.path(), &names, generation)
                .expect("macOS CEF objects"),
        );
        self.slots
            .install(slot, generation, identity, objects, admission)
            .expect("emergency entry");
        self.children.push(child);
        AdmittedKey {
            slot,
            generation,
            pid,
        }
    }
}

impl Drop for EmergencySlotsFixture {
    fn drop(&mut self) {
        for child in &mut self.children {
            if child.try_wait().ok().flatten().is_none() {
                let _ = child.kill();
            }
            let _ = child.wait();
        }
    }
}

fn grouped_sleep() -> Child {
    let mut command = Command::new("/bin/sleep");
    command.arg("30");
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) == 0 {
                Ok(())
            } else {
                Err(std::io::Error::last_os_error())
            }
        });
    }
    command.spawn().expect("grouped child")
}
