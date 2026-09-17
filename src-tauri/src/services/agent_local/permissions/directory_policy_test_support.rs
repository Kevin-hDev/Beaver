use std::cell::RefCell;
use std::path::PathBuf;

thread_local! {
    // L'isolation par thread évite de modifier la politique des tests parallèles.
    static ROOTS: RefCell<Option<Vec<PathBuf>>> = const { RefCell::new(None) };
}

pub(crate) fn current_roots() -> Option<Vec<PathBuf>> {
    ROOTS.with(|value| value.borrow().clone())
}

pub(crate) fn with_roots<T>(roots: Vec<PathBuf>, action: impl FnOnce() -> T) -> T {
    assert!(!roots.is_empty());
    assert!(roots.len() <= super::super::directory_access::MAX_ALLOWED_PATHS);
    assert!(
        roots.iter().all(|root| {
            root.is_absolute()
                && root.is_dir()
                && dunce::canonicalize(root).is_ok_and(|canonical| canonical == *root)
        }),
        "test roots must be canonical directories"
    );

    struct Restore(Option<Vec<PathBuf>>);

    impl Drop for Restore {
        fn drop(&mut self) {
            ROOTS.with(|value| *value.borrow_mut() = self.0.take());
        }
    }

    let previous = ROOTS.with(|value| value.replace(Some(roots)));
    let _restore = Restore(previous);
    action()
}

#[cfg(test)]
mod tests {
    use super::with_roots;
    use std::path::PathBuf;

    #[test]
    #[should_panic(expected = "test roots must be canonical directories")]
    fn rejects_a_relative_root() {
        with_roots(vec![PathBuf::from("relative")], || ());
    }

    #[test]
    #[should_panic(expected = "test roots must be canonical directories")]
    fn rejects_a_noncanonical_root() {
        let base = tempfile::tempdir().expect("test base");
        let nested = base.path().join("nested");
        std::fs::create_dir(&nested).expect("nested directory");

        with_roots(vec![nested.join("..")], || ());
    }
}
