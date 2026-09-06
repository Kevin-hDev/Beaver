use super::{disk_policy::*, InstallInterruption};

fn policy() -> DiskPolicy {
    DiskPolicy {
        warning_bytes: 10,
        reserve_bytes: 5,
        poll_interval: POLL_INTERVAL,
    }
}

#[test]
fn consent_is_finite_once_and_never_spends_the_reserve() {
    let mut allowance = StorageAllowance::new(policy());
    assert!(allowance.check(10, 20, policy()).is_ok());
    assert_eq!(
        allowance.check(11, 20, policy()),
        Err(InstallInterruption::Confirmation)
    );
    allowance.approve(11, 20, policy()).unwrap();
    assert_eq!(allowance.approved_total_bytes, 26);
    assert!(allowance.check(26, 6, policy()).is_ok());
    assert_eq!(
        allowance.check(27, 6, policy()),
        Err(InstallInterruption::InsufficientSpace)
    );
    assert_eq!(
        allowance.check(11, 5, policy()),
        Err(InstallInterruption::InsufficientSpace)
    );
    assert!(allowance.approve(11, 200, policy()).is_err());
}

#[test]
fn missing_capacity_reserve_and_overflow_fail_closed() {
    for (occupied, free) in [(0, 0), (0, 5), (u64::MAX, 6)] {
        let mut allowance = StorageAllowance::new(policy());
        assert!(allowance.approve(occupied, free, policy()).is_err());
        assert!(!allowance.confirmation_used);
    }
    assert!(free_bytes(std::path::Path::new("/nonexistent-beaver-fixture-volume")).is_err());
}

#[test]
fn counts_cache_temporary_and_output_with_one_combined_bound() {
    let root = tempfile::tempdir().unwrap();
    for (name, size) in [("cache", 3), ("tmp", 4), ("output", 5)] {
        let directory = root.path().join(name);
        std::fs::create_dir(&directory).unwrap();
        std::fs::write(directory.join("bytes"), vec![0_u8; size]).unwrap();
    }
    assert_eq!(
        super::disk_usage::measure_roots(&[root.path().to_owned()]).unwrap(),
        12
    );
}

#[cfg(unix)]
#[test]
fn refuses_symlink_roots_and_children() {
    let root = tempfile::tempdir().unwrap();
    let link = root.path().join("outside");
    std::os::unix::fs::symlink("/tmp", &link).unwrap();
    assert!(super::disk_usage::measure_roots(&[link]).is_err());
    assert!(super::disk_usage::measure_roots(&[root.path().to_owned()]).is_err());
}
