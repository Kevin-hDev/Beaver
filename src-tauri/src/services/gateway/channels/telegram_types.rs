use serde::Deserialize;

use super::bounded_vec::BoundedVec;

const MAX_TELEGRAM_ENTITIES: usize = 100;
const MAX_TELEGRAM_UPDATES: usize = 100;

#[derive(Debug, Deserialize)]
pub struct TgResponse<T> {
    pub ok: bool,
    pub result: Option<T>,
}

#[derive(Debug, Deserialize)]
pub struct TgUpdate {
    pub update_id: i64,
    pub message: Option<TgMessage>,
}

#[derive(Debug, Deserialize)]
pub struct TgMessage {
    pub message_id: i64,
    #[serde(default)]
    pub message_thread_id: Option<i64>,
    pub from: Option<TgUser>,
    pub chat: TgChat,
    pub text: Option<String>,
    pub entities: Option<BoundedVec<TgEntity, MAX_TELEGRAM_ENTITIES>>,
}

#[derive(Debug, Deserialize)]
pub struct TgUser {
    pub id: i64,
    pub username: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TgChat {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: String,
}

#[derive(Debug, Deserialize)]
pub struct TgEntity {
    #[serde(rename = "type")]
    pub entity_type: String,
    pub offset: u32,
    pub length: u32,
}

#[derive(Debug, Deserialize)]
pub struct TgSentMessage {}

pub type TgUpdates = BoundedVec<TgUpdate, MAX_TELEGRAM_UPDATES>;

impl TgChat {
    pub fn is_group(&self) -> bool {
        self.chat_type == "group" || self.chat_type == "supergroup"
    }
}

impl TgMessage {
    pub fn has_bot_mention(&self, bot_username: &str) -> bool {
        let Some(entities) = &self.entities else {
            return false;
        };
        let Some(text) = &self.text else { return false };
        let lower_bot = bot_username.to_lowercase();
        for e in entities {
            if e.entity_type == "mention" {
                if let Some(mention) = utf16_slice(text, e.offset, e.length) {
                    let clean = mention.trim_start_matches('@').to_lowercase();
                    if clean == lower_bot {
                        return true;
                    }
                }
            }
        }
        false
    }
}

fn utf16_slice(text: &str, offset: u32, length: u32) -> Option<&str> {
    let start_units = usize::try_from(offset).ok()?;
    let end_units = start_units.checked_add(usize::try_from(length).ok()?)?;
    let mut units = 0usize;
    let mut start = None;
    let mut end = None;
    for (index, ch) in text.char_indices() {
        if units == start_units {
            start = Some(index);
        }
        if units == end_units {
            end = Some(index);
            break;
        }
        units = units.checked_add(ch.len_utf16())?;
    }
    if start.is_none() && units == start_units {
        start = Some(text.len());
    }
    if end.is_none() && units == end_units {
        end = Some(text.len());
    }
    text.get(start?..end?)
}

#[cfg(test)]
#[path = "telegram_types_tests.rs"]
mod tests;
