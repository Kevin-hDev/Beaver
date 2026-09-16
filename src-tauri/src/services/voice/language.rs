use super::types::{VoiceLanguage, VoiceModel};

pub fn normalize_for_model(model: VoiceModel, language: VoiceLanguage) -> VoiceLanguage {
    // Keep persisted settings honest: automatic-only engines cannot consume a forced language.
    match model {
        VoiceModel::ParakeetTdtV3 | VoiceModel::Qwen3Asr06b => VoiceLanguage::Automatic,
        VoiceModel::CohereTranscribe => match language {
            VoiceLanguage::Automatic => VoiceLanguage::FollowInterface,
            language => language,
        },
    }
}
