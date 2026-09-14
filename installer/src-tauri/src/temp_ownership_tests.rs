use super::temp_ownership::{purge_orphans, OwnedTempRun, OWNER_MARKER};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

const CURRENT: &str = "0123456789abcdef0123456789abcdef";
static NEXT: AtomicU64 = AtomicU64::new(1);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "beaver-owned-temp-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }

    fn run(&self, id: &str) -> PathBuf {
        let path = self.0.join(format!("beaver-install-{id}"));
        fs::create_dir(&path).unwrap();
        fs::write(
            path.join(OWNER_MARKER),
            format!(r#"{{"schema":1,"runId":"{id}"}}"#),
        )
        .unwrap();
        path
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn adopts_and_cleans_only_a_proven_owned_run() {
    let fixture = Fixture::new();
    let path = fixture.run(CURRENT);
    let owned = OwnedTempRun::adopt(&fixture.0, &path, CURRENT).unwrap();
    assert_eq!(owned.path(), path);
    drop(owned);
    assert!(!path.exists());
}

#[test]
fn explicit_cleanup_removes_a_proven_owned_run_before_process_exit() {
    let fixture = Fixture::new();
    let path = fixture.run(CURRENT);
    let owned = OwnedTempRun::adopt(&fixture.0, &path, CURRENT).unwrap();

    owned.cleanup().unwrap();

    assert!(!path.exists());
}

#[test]
fn disarmed_cleanup_preserves_the_run_for_the_next_launch() {
    let fixture = Fixture::new();
    let path = fixture.run(CURRENT);
    let owned = OwnedTempRun::adopt(&fixture.0, &path, CURRENT).unwrap();

    owned.disarm_cleanup();
    drop(owned);

    assert!(path.exists());
    assert!(path.join(OWNER_MARKER).exists());
}

#[test]
fn rejects_absent_forged_and_divergent_markers() {
    let fixture = Fixture::new();
    let absent = fixture.0.join(format!("beaver-install-{CURRENT}"));
    fs::create_dir(&absent).unwrap();
    assert!(OwnedTempRun::adopt(&fixture.0, &absent, CURRENT).is_err());

    fs::write(absent.join(OWNER_MARKER), "not-json").unwrap();
    assert!(OwnedTempRun::adopt(&fixture.0, &absent, CURRENT).is_err());

    fs::write(
        absent.join(OWNER_MARKER),
        r#"{"schema":1,"runId":"ffffffffffffffffffffffffffffffff"}"#,
    )
    .unwrap();
    assert!(OwnedTempRun::adopt(&fixture.0, &absent, CURRENT).is_err());
}

#[cfg(unix)]
#[test]
fn rejects_symbolic_candidates_outside_paths_and_symbolic_children() {
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new();
    let outside = Fixture::new();
    let link = fixture.0.join(format!("beaver-install-{CURRENT}"));
    symlink(&outside.0, &link).unwrap();
    assert!(OwnedTempRun::adopt(&fixture.0, &link, CURRENT).is_err());
    assert!(OwnedTempRun::adopt(&fixture.0, &outside.0, CURRENT).is_err());

    fs::remove_file(&link).unwrap();
    let linked_marker_run = fixture.run(CURRENT);
    let outside_marker = outside.0.join("owner.json");
    fs::write(
        &outside_marker,
        format!(r#"{{"schema":1,"runId":"{CURRENT}"}}"#),
    )
    .unwrap();
    fs::remove_file(linked_marker_run.join(OWNER_MARKER)).unwrap();
    symlink(&outside_marker, linked_marker_run.join(OWNER_MARKER)).unwrap();
    assert!(OwnedTempRun::adopt(&fixture.0, &linked_marker_run, CURRENT).is_err());
    fs::remove_dir_all(&linked_marker_run).unwrap();

    let run = fixture.run(CURRENT);
    let sentinel = outside.0.join("sentinel");
    fs::write(&sentinel, "keep").unwrap();
    symlink(&sentinel, run.join("child-link")).unwrap();
    let owned = OwnedTempRun::adopt(&fixture.0, &run, CURRENT).unwrap();
    drop(owned);
    assert!(run.exists());
    assert!(sentinel.exists());
}

#[test]
fn five_crashed_runs_are_purged_without_touching_the_current_run() {
    let fixture = Fixture::new();
    let current = fixture.run(CURRENT);
    for index in 0..5 {
        fixture.run(&format!("{index:032x}"));
    }

    purge_orphans(&fixture.0, CURRENT);

    let remaining: Vec<_> = fs::read_dir(&fixture.0)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(remaining, vec![current]);
}

#[test]
fn ordinary_temp_entries_do_not_hide_an_owned_orphan() {
    let fixture = Fixture::new();
    for index in 0..4_096 {
        fs::write(fixture.0.join(format!("ordinary-{index}")), "keep").unwrap();
    }
    let orphan = fixture.run("ffffffffffffffffffffffffffffffff");

    purge_orphans(&fixture.0, CURRENT);

    assert!(!orphan.exists());
}

#[test]
fn active_run_is_not_purged() {
    let fixture = Fixture::new();
    let path = fixture.run(CURRENT);
    let active = OwnedTempRun::adopt(&fixture.0, &path, CURRENT).unwrap();
    active.mark_active().unwrap();

    purge_orphans(&fixture.0, "ffffffffffffffffffffffffffffffff");

    assert!(path.exists());
}
