use super::types::{ExtensionRecord, ExtensionStatus};

pub fn for_update(current: &ExtensionRecord, mut replacement: ExtensionRecord) -> ExtensionRecord {
    replacement.enabled = false;
    replacement.trusted = false;
    replacement.trusted_at = None;
    replacement.show_in_chat = current.show_in_chat;
    replacement.status = ExtensionStatus::Inactive;
    replacement.last_error = None;
    replacement.last_activated_at = current.last_activated_at.clone();
    replacement.sensitive_access_granted = current.sensitive_access_granted;
    replacement
}

pub fn carry_sensitive_access(
    current: &ExtensionRecord,
    replacement: &mut ExtensionRecord,
) -> bool {
    replacement.sensitive_access_granted |= current.sensitive_access_granted;
    current.sensitive_access_granted
}

#[cfg(test)]
mod tests {
    use super::{carry_sensitive_access, for_update};

    #[test]
    fn managed_updates_revoke_trust_and_preserve_user_preferences() {
        let mut current = crate::services::extensions::builtin::records()
            .unwrap()
            .remove(0);
        current.enabled = true;
        current.trusted = true;
        current.fingerprint = Some("ab".repeat(32));
        current.trusted_at = Some("2026-07-28T00:00:00Z".to_string());
        current.show_in_chat = true;
        current.last_activated_at = Some("2026-07-28T00:00:00Z".to_string());
        current.sensitive_access_granted = true;
        let mut downloaded = current.clone();
        downloaded.manifest.version = "2.0.0".to_string();
        downloaded.fingerprint = Some("cd".repeat(32));

        let replacement = for_update(&current, downloaded);

        assert!(!replacement.enabled);
        assert!(!replacement.trusted);
        assert_eq!(replacement.fingerprint, Some("cd".repeat(32)));
        assert!(replacement.trusted_at.is_none());
        assert!(replacement.show_in_chat);
        assert_eq!(replacement.last_activated_at, current.last_activated_at);
        assert!(replacement.sensitive_access_granted);
        assert_eq!(
            serde_json::to_value(&replacement).unwrap()["sensitiveAccessGranted"],
            true
        );
    }

    #[test]
    fn replacement_keeps_an_access_recorded_after_update_preparation() {
        let mut prepared_from = crate::services::extensions::builtin::records()
            .unwrap()
            .remove(0);
        prepared_from.kind = crate::services::extensions::types::ExtensionKind::Local;
        prepared_from.sensitive_access_granted = false;
        let mut replacement = for_update(&prepared_from, prepared_from.clone());

        let mut current_at_commit = prepared_from;
        current_at_commit.sensitive_access_granted = true;
        let reminder = carry_sensitive_access(&current_at_commit, &mut replacement);

        assert!(reminder);
        assert!(replacement.sensitive_access_granted);
    }
}
