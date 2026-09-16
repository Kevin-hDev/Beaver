use super::{contracts::VoiceDestination, recovery::Recovery};

fn recovery_ready_fixture(ready_at_ms: u64) -> Recovery {
    let mut recovery = Recovery::preparing(
        "recovery-1".into(),
        &VoiceDestination::Draft {
            draft_key: "draft-1".into(),
        },
        160_000,
    );
    assert!(recovery.mark_ready("texte".into(), ready_at_ms));
    recovery
}

#[test]
fn recovery_expiry_starts_when_the_text_is_ready() {
    let ready = recovery_ready_fixture(120_000);
    assert!(!ready.is_expired_at(719_999));
    assert!(ready.is_expired_at(720_000));
}
