use super::{contracts::VoiceDestination, start_guards::validate_start, types::VoiceLanguage};

#[test]
fn start_requires_a_live_context_and_foreground_window() {
    let destination = VoiceDestination::Draft {
        draft_key: "draft-1".into(),
    };
    assert!(validate_start(&destination, 1, &None, true).is_ok());
    assert!(validate_start(&destination, 0, &None, true).is_err());
    assert!(validate_start(&destination, 1, &None, false).is_err());
    assert!(validate_start(
        &destination,
        1,
        &Some(VoiceLanguage::Language("fr\n".into())),
        true
    )
    .is_err());
}
