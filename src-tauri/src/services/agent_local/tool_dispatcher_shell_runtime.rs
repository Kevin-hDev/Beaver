use super::agent_work_supervision::ShellWork;
use tauri::Manager;

pub(super) fn shell_work() -> Result<ShellWork, String> {
    #[cfg(test)]
    if let Some(work) = test_support::current() {
        return Ok(work);
    }

    super::app_handle_global::get()
        .map(|app| app.state::<super::agent_work_supervision::AgentWorkServices>().shells())
        .ok_or_else(|| "application-context-unavailable".to_string())
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::ShellWork;
    use std::cell::RefCell;

    thread_local! {
        static WORK: RefCell<Option<ShellWork>> = const { RefCell::new(None) };
    }

    pub(super) fn current() -> Option<ShellWork> {
        WORK.with(|slot| slot.borrow().clone())
    }

    pub(crate) fn with<T>(work: ShellWork, action: impl FnOnce() -> T) -> T {
        struct Restore(Option<ShellWork>);

        impl Drop for Restore {
            fn drop(&mut self) {
                WORK.with(|slot| *slot.borrow_mut() = self.0.take());
            }
        }

        let previous = WORK.with(|slot| slot.replace(Some(work)));
        let _restore = Restore(previous);
        action()
    }
}
