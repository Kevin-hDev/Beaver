use super::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

struct FakeEntry {
    name: &'static str,
    enabled: Cell<bool>,
    fail_read: Cell<bool>,
    fail_enable: Cell<bool>,
    fail_disable: Cell<bool>,
    calls: Rc<RefCell<Vec<String>>>,
}

impl FakeEntry {
    fn new(name: &'static str, enabled: bool, calls: Rc<RefCell<Vec<String>>>) -> Self {
        Self {
            name,
            enabled: Cell::new(enabled),
            fail_read: Cell::new(false),
            fail_enable: Cell::new(false),
            fail_disable: Cell::new(false),
            calls,
        }
    }

    fn record(&self, action: &str) {
        self.calls
            .borrow_mut()
            .push(format!("{}:{action}", self.name));
    }
}

impl LaunchEntry for FakeEntry {
    fn is_enabled(&self) -> Result<bool, MigrationError> {
        self.record("read");
        if self.fail_read.get() {
            Err(MigrationError::State)
        } else {
            Ok(self.enabled.get())
        }
    }

    fn enable(&self) -> Result<(), MigrationError> {
        self.record("enable");
        if self.fail_enable.get() {
            Err(MigrationError::State)
        } else {
            self.enabled.set(true);
            Ok(())
        }
    }

    fn disable(&self) -> Result<(), MigrationError> {
        self.record("disable");
        if self.fail_disable.get() {
            Err(MigrationError::State)
        } else {
            self.enabled.set(false);
            Ok(())
        }
    }
}

#[test]
fn migration_enables_beaver_before_removing_legacy_entry() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let active = FakeEntry::new("beaver", false, calls.clone());
    let legacy = FakeEntry::new("legacy", true, calls.clone());
    let marked = Cell::new(false);

    migrate_and_mark(&active, &legacy, true, || {
        marked.set(true);
        Ok(())
    })
    .unwrap();

    assert!(active.enabled.get());
    assert!(!legacy.enabled.get());
    assert!(marked.get());
    let calls = calls.borrow();
    let enable = calls
        .iter()
        .position(|call| call == "beaver:enable")
        .unwrap();
    let disable = calls
        .iter()
        .position(|call| call == "legacy:disable")
        .unwrap();
    assert!(enable < disable);
}

#[test]
fn failed_beaver_enable_preserves_legacy_entry_and_marker() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let active = FakeEntry::new("beaver", false, calls.clone());
    active.fail_enable.set(true);
    let legacy = FakeEntry::new("legacy", true, calls.clone());
    let marked = Cell::new(false);

    assert!(migrate_and_mark(&active, &legacy, true, || {
        marked.set(true);
        Ok(())
    })
    .is_err());

    assert!(!active.enabled.get());
    assert!(legacy.enabled.get());
    assert!(!marked.get());
    assert!(!calls.borrow().iter().any(|call| call == "legacy:disable"));
}

#[test]
fn failed_legacy_cleanup_rolls_back_beaver_for_retry() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let active = FakeEntry::new("beaver", false, calls.clone());
    let legacy = FakeEntry::new("legacy", true, calls.clone());
    legacy.fail_disable.set(true);
    let marked = Cell::new(false);

    assert!(migrate_and_mark(&active, &legacy, true, || {
        marked.set(true);
        Ok(())
    })
    .is_err());

    assert!(!active.enabled.get());
    assert!(legacy.enabled.get());
    assert!(!marked.get());
    assert!(calls.borrow().iter().any(|call| call == "beaver:disable"));
}

#[test]
fn disabled_setting_attempts_to_remove_both_entries() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let active = FakeEntry::new("beaver", false, calls.clone());
    let legacy = FakeEntry::new("legacy", false, calls.clone());

    migrate_and_mark(&active, &legacy, false, || Ok(())).unwrap();

    let calls = calls.borrow();
    assert!(calls.iter().any(|call| call == "beaver:disable"));
    assert!(calls.iter().any(|call| call == "legacy:disable"));
}

#[test]
fn both_states_are_read_before_the_first_mutation() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let active = FakeEntry::new("beaver", false, calls.clone());
    let legacy = FakeEntry::new("legacy", true, calls.clone());

    migrate_and_mark(&active, &legacy, true, || Ok(())).unwrap();

    let calls = calls.borrow();
    assert_eq!(&calls[..2], ["beaver:read", "legacy:read"]);
}

#[test]
fn read_failure_prevents_all_mutations_and_marker() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let active = FakeEntry::new("beaver", false, calls.clone());
    let legacy = FakeEntry::new("legacy", true, calls.clone());
    legacy.fail_read.set(true);
    let marked = Cell::new(false);

    assert!(migrate_and_mark(&active, &legacy, true, || {
        marked.set(true);
        Ok(())
    })
    .is_err());

    assert!(!marked.get());
    assert!(!calls
        .borrow()
        .iter()
        .any(|call| call.ends_with(":enable") || call.ends_with(":disable")));
}

#[test]
fn marker_failure_is_reported_after_verified_migration() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let active = FakeEntry::new("beaver", false, calls.clone());
    let legacy = FakeEntry::new("legacy", true, calls);

    let result = migrate_and_mark(&active, &legacy, true, || Err(MigrationError::Marker));

    assert_eq!(result, Err(MigrationError::Marker));
    assert!(active.enabled.get());
    assert!(!legacy.enabled.get());
}

#[test]
fn public_and_legacy_contracts_are_exact() {
    assert_eq!(ACTIVE_ENTRY_NAME, "Beaver");
    assert_eq!(LEGACY_ENTRY_NAME, "CL-GO");
    assert_eq!(AUTOSTART_ARG, "--clgo-autostart");
    assert_eq!(MARKER_FILE, "autostart-beaver-v1");
}

struct ExactFakeEntry {
    state: Cell<ExactEntryState>,
    installs: Cell<usize>,
    removals: Cell<usize>,
}

impl ExactFakeEntry {
    fn new(state: ExactEntryState) -> Self {
        Self {
            state: Cell::new(state),
            installs: Cell::new(0),
            removals: Cell::new(0),
        }
    }
}

impl ExactLaunchEntry for ExactFakeEntry {
    fn state(&self) -> Result<ExactEntryState, MigrationError> {
        Ok(self.state.get())
    }

    fn install(&self) -> Result<(), MigrationError> {
        self.installs.set(self.installs.get() + 1);
        self.state.set(ExactEntryState::Exact);
        Ok(())
    }

    fn remove(&self) -> Result<(), MigrationError> {
        self.removals.set(self.removals.get() + 1);
        self.state.set(ExactEntryState::Absent);
        Ok(())
    }
}

#[test]
fn stale_active_entry_is_replaced_and_verified() {
    let entry = ExactFakeEntry::new(ExactEntryState::Stale);

    synchronize_exact_entry(&entry, true).unwrap();

    assert_eq!(entry.state.get(), ExactEntryState::Exact);
    assert_eq!(entry.installs.get(), 1);
    assert_eq!(entry.removals.get(), 0);
}

#[test]
fn exact_active_entry_is_not_rewritten() {
    let entry = ExactFakeEntry::new(ExactEntryState::Exact);

    synchronize_exact_entry(&entry, true).unwrap();

    assert_eq!(entry.installs.get(), 0);
    assert_eq!(entry.removals.get(), 0);
}

#[test]
fn absent_active_entry_is_already_disabled() {
    let entry = ExactFakeEntry::new(ExactEntryState::Absent);

    synchronize_exact_entry(&entry, false).unwrap();

    assert_eq!(entry.installs.get(), 0);
    assert_eq!(entry.removals.get(), 0);
}
