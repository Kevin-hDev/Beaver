use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, ts(rename_all = "snake_case"))]
pub enum CompressionWindowBand {
    #[serde(rename = "under_64k")]
    #[cfg_attr(test, ts(rename = "under_64k"))]
    Under64K,
    Compact,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, ts(rename_all = "snake_case"))]
pub enum CompressionTrigger {
    Automatic,
    Explicit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CompressionBandSettings {
    pub recent_message_count: u8,
    pub summary_max_tokens: u32,
    pub tool_result_count: u16,
    pub recent_file_count: u16,
    pub image_count: u16,
    pub include_work_state: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CompressionProfile {
    pub id: String,
    pub name: String,
    #[cfg_attr(test, ts(type = "number"))]
    pub revision: u64,
    pub threshold_percent: u8,
    pub allow_under_64k: bool,
    pub system_prompt: String,
    pub handoff_prompt: String,
    pub under_64k: CompressionBandSettings,
    pub compact: CompressionBandSettings,
    pub large: CompressionBandSettings,
}

impl CompressionProfile {
    pub fn band_settings(&self, band: CompressionWindowBand) -> &CompressionBandSettings {
        match band {
            CompressionWindowBand::Under64K => &self.under_64k,
            CompressionWindowBand::Compact => &self.compact,
            CompressionWindowBand::Large => &self.large,
        }
    }
}
