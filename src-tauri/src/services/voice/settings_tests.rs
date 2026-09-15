use super::settings::{read_voice_settings_at_path, update_voice_settings_at_path};
use super::types::{
    VoiceInputGain, VoiceMaxDuration, VoiceSettings, VoiceSettingsPatch, VoiceSilenceTimeout,
};

#[test]
fn defaults_match_the_product_decisions_and_invalid_input_fails_closed() {
    let settings = VoiceSettings::default();
    assert!(settings.enabled);
    assert_eq!(settings.input_gain, VoiceInputGain::Six);
    assert_eq!(settings.silence_timeout, VoiceSilenceTimeout::FiveSeconds);
    assert_eq!(settings.max_duration, VoiceMaxDuration::Ten);
    assert!(VoiceSettingsPatch {
        shortcut: Some(Some("bad\nshortcut".into())),
        ..Default::default()
    }
    .validate()
    .is_err());
    let clear_shortcut: VoiceSettingsPatch =
        serde_json::from_value(serde_json::json!({ "shortcut": null })).unwrap();
    assert_eq!(clear_shortcut.shortcut, Some(None));
    assert!(
        serde_json::from_value::<VoiceSettingsPatch>(serde_json::json!({
            "model_path": "/tmp/untrusted"
        }))
        .is_err()
    );
}

#[test]
fn legacy_settings_receive_the_default_gain_without_losing_other_values() {
    let legacy = serde_json::json!({
        "version": 1,
        "enabled": true,
        "model": "qwen3-asr06b",
        "input_device": { "kind": "system-default" },
        "silence_timeout": "ten-seconds",
        "max_duration": "10-minutes",
        "language": { "kind": "follow-interface" },
        "shortcut": null,
        "unload_delay": "two-minutes",
        "explanation_accepted": true
    });
    let settings: VoiceSettings = serde_json::from_value(legacy).unwrap();
    assert_eq!(settings.input_gain, VoiceInputGain::Six);
    assert_eq!(settings.silence_timeout, VoiceSilenceTimeout::TenSeconds);
}

#[test]
fn tolerant_read_and_serialized_update_preserve_other_config_sections() {
    let directory = tempfile::tempdir().expect("temp dir");
    let path = directory.path().join("config.json");
    std::fs::write(
        &path,
        r#"{"advanced":{"default_model":"keep-me"},"voice":{"silence_timeout":"unknown"}}"#,
    )
    .expect("fixture");
    assert_eq!(
        read_voice_settings_at_path(&path, directory.path()).expect("tolerant read"),
        VoiceSettings::default()
    );

    let updated = update_voice_settings_at_path(
        &path,
        directory.path(),
        VoiceSettingsPatch {
            max_duration: Some(VoiceMaxDuration::Thirty),
            ..Default::default()
        },
    )
    .expect("settings update");
    assert_eq!(updated.max_duration, VoiceMaxDuration::Thirty);
    let json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).expect("updated config")).expect("json");
    assert_eq!(json["advanced"]["default_model"], "keep-me");
}
