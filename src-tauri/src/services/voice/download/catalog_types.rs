use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoiceCatalog {
    pub version: u8,
    pub entries: Vec<VoiceCatalogEntry>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoiceCatalogEntry {
    pub id: String,
    pub role: VoiceModelRole,
    pub engine: VoiceEngine,
    pub revision: String,
    pub archive: VoiceArchive,
    pub manifest_url: String,
    pub installed_bytes: u64,
    pub files: Vec<VoiceModelFile>,
    pub languages: Vec<String>,
    pub dialects: Vec<String>,
    pub language_mode: VoiceLanguageMode,
    pub license: VoiceLicense,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum VoiceModelRole {
    Asr,
    Vad,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum VoiceEngine {
    NemoTransducer,
    CohereTranscribe,
    Qwen3Asr,
    SileroVad,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum VoiceLanguageMode {
    AutomaticOnly,
    ExplicitOrAutomatic,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoiceArchive {
    pub url: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoiceModelFile {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoiceLicense {
    pub spdx: String,
    pub url: String,
}
