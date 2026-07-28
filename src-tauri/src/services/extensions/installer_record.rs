use super::types::{ExtensionRecord, ExtensionStatus};

pub fn for_update(current: &ExtensionRecord, mut replacement: ExtensionRecord) -> ExtensionRecord {
    replacement.enabled = false;
    replacement.trusted = false;
    replacement.show_in_chat = current.show_in_chat;
    replacement.status = ExtensionStatus::Inactive;
    replacement.last_error = None;
    replacement.last_activated_at = current.last_activated_at.clone();
    replacement
}

#[cfg(test)]
mod tests {
    use super::for_update;

    #[test]
    fn managed_updates_revoke_trust_and_preserve_user_preferences() {
        let mut current = crate::services::extensions::builtin::records()
            .unwrap()
            .remove(0);
        current.enabled = true;
        current.trusted = true;
        current.show_in_chat = true;
        current.last_activated_at = Some("2026-07-28T00:00:00Z".to_string());
        let mut downloaded = current.clone();
        downloaded.manifest.version = "2.0.0".to_string();

        let replacement = for_update(&current, downloaded);

        assert!(!replacement.enabled);
        assert!(!replacement.trusted);
        assert!(replacement.show_in_chat);
        assert_eq!(replacement.last_activated_at, current.last_activated_at);
    }
}
