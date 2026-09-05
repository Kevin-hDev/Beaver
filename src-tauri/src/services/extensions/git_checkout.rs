use git2::build::CheckoutBuilder;
use git2::CheckoutNotificationType;
use std::collections::HashSet;
use std::path::Component;

use super::install_signal::InstallSignal;

pub fn bounded(cancellation: impl InstallSignal) -> CheckoutBuilder<'static> {
    let mut checkout = CheckoutBuilder::new();
    let mut paths = HashSet::new();
    let mut bytes = 0_u64;
    checkout.disable_filters(true);
    checkout.notify_on(CheckoutNotificationType::all());
    checkout.notify(move |_, path, _, target, _| {
        if cancellation.is_cancelled() {
            return false;
        }
        let Some(path) = path else {
            return false;
        };
        if path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        {
            return false;
        }
        if !paths.insert(path.to_path_buf()) {
            return true;
        }
        let size = target.map(|file| file.size()).unwrap_or_default();
        if paths.len() > super::managed_tree::MAX_ENTRIES
            || size > super::managed_tree::MAX_FILE_BYTES
        {
            return false;
        }
        let Some(total) = bytes.checked_add(size) else {
            return false;
        };
        bytes = total;
        bytes <= super::managed_tree::MAX_TOTAL_BYTES
    });
    checkout
}
