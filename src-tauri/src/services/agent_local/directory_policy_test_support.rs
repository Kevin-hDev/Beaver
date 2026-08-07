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
