use std::fmt;
use std::marker::PhantomData;

use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

use super::agent_turn_contract::{
    NewUserTurnInput, ResumeTurnInput, SkillReference, TurnAttachmentInput, TurnStart,
    MAX_ATTACHMENT_GRANT_BYTES, MAX_ATTACHMENT_MIME_BYTES, MAX_ATTACHMENT_NAME_BYTES,
    MAX_ATTACHMENT_PATH_BYTES, MAX_ATTACHMENT_THUMBNAIL_BYTES, MAX_RESUME_MESSAGE_ID_BYTES,
    MAX_SKILLS_PER_TURN, MAX_SKILL_ID_BYTES, MAX_SKILL_NAME_BYTES, MAX_TURN_ATTACHMENTS,
    MAX_TURN_CONTENT_BYTES,
};

pub(super) fn turn_start<'de, D>(deserializer: D) -> Result<TurnStart, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(
        tag = "type",
        content = "input",
        rename_all = "camelCase",
        deny_unknown_fields
    )]
    enum Wire {
        New(NewUserTurnInput),
        Resume(ResumeTurnInput),
    }
    Ok(match Wire::deserialize(deserializer)? {
        Wire::New(input) => TurnStart::New(input),
        Wire::Resume(input) => TurnStart::Resume(input),
    })
}

pub(super) fn new_turn<'de, D>(deserializer: D) -> Result<NewUserTurnInput, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Wire {
        content: BoundedString<MAX_TURN_CONTENT_BYTES>,
        files: BoundedVec<TurnAttachmentInput, MAX_TURN_ATTACHMENTS>,
        skills: BoundedVec<SkillReference, MAX_SKILLS_PER_TURN>,
    }
    let wire = Wire::deserialize(deserializer)?;
    if wire.content.0.contains('\0') {
        return Err(serde::de::Error::custom("invalid turn content"));
    }
    Ok(NewUserTurnInput {
        content: wire.content.0,
        files: wire.files.0,
        skills: wire.skills.0,
    })
}

pub(super) fn attachment<'de, D>(deserializer: D) -> Result<TurnAttachmentInput, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Wire {
        name: BoundedString<MAX_ATTACHMENT_NAME_BYTES>,
        path: BoundedString<MAX_ATTACHMENT_PATH_BYTES>,
        mime_type: BoundedString<MAX_ATTACHMENT_MIME_BYTES>,
        size: u64,
        #[serde(default)]
        thumbnail: Option<BoundedString<MAX_ATTACHMENT_THUMBNAIL_BYTES>>,
        #[serde(default)]
        access_grant: Option<BoundedString<MAX_ATTACHMENT_GRANT_BYTES>>,
    }
    let wire = Wire::deserialize(deserializer)?;
    if [&wire.name.0, &wire.path.0, &wire.mime_type.0]
        .into_iter()
        .any(|value| value.chars().any(char::is_control))
        || wire
            .thumbnail
            .as_ref()
            .is_some_and(|value| value.0.chars().any(char::is_control))
        || wire
            .access_grant
            .as_ref()
            .is_some_and(|value| value.0.chars().any(char::is_control))
    {
        return Err(serde::de::Error::custom("invalid attachment input"));
    }
    Ok(TurnAttachmentInput {
        name: wire.name.0,
        path: wire.path.0,
        mime_type: wire.mime_type.0,
        size: wire.size,
        thumbnail: wire.thumbnail.map(|value| value.0),
        access_grant: wire.access_grant.map(|value| value.0),
    })
}

pub(super) fn skill<'de, D>(deserializer: D) -> Result<SkillReference, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Wire {
        id: BoundedString<MAX_SKILL_ID_BYTES>,
        #[serde(default)]
        name: Option<BoundedString<MAX_SKILL_NAME_BYTES>>,
    }
    let wire = Wire::deserialize(deserializer)?;
    if wire.id.0.chars().any(char::is_control)
        || wire
            .name
            .as_ref()
            .is_some_and(|value| value.0.chars().any(char::is_control))
    {
        return Err(serde::de::Error::custom("invalid skill reference"));
    }
    Ok(SkillReference {
        id: wire.id.0,
        name: wire.name.map(|value| value.0),
    })
}

#[allow(dead_code, reason = "adopted by resume admission in Task 8")]
pub(super) fn resume<'de, D>(deserializer: D) -> Result<ResumeTurnInput, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Wire {
        message_id: BoundedString<MAX_RESUME_MESSAGE_ID_BYTES>,
    }
    let wire = Wire::deserialize(deserializer)?;
    if wire.message_id.0.chars().any(char::is_control) {
        return Err(serde::de::Error::custom("invalid resume identifier"));
    }
    Ok(ResumeTurnInput {
        message_id: wire.message_id.0,
    })
}

struct BoundedString<const MAX: usize>(String);

impl<'de, const MAX: usize> Deserialize<'de> for BoundedString<MAX> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StringVisitor<const MAX: usize>;
        impl<'de, const MAX: usize> Visitor<'de> for StringVisitor<MAX> {
            type Value = BoundedString<MAX>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "a string of at most {MAX} bytes")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                bounded_string(value.len(), || value.to_owned())
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                bounded_string(value.len(), || value)
            }
        }
        deserializer.deserialize_string(StringVisitor::<MAX>)
    }
}

fn bounded_string<E, F, const MAX: usize>(length: usize, value: F) -> Result<BoundedString<MAX>, E>
where
    E: serde::de::Error,
    F: FnOnce() -> String,
{
    if length > MAX {
        Err(E::custom("string limit exceeded"))
    } else {
        Ok(BoundedString(value()))
    }
}

struct BoundedVec<T, const MAX: usize>(Vec<T>);

impl<'de, T, const MAX: usize> Deserialize<'de> for BoundedVec<T, MAX>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VecVisitor<T, const MAX: usize>(PhantomData<T>);
        impl<'de, T, const MAX: usize> Visitor<'de> for VecVisitor<T, MAX>
        where
            T: Deserialize<'de>,
        {
            type Value = BoundedVec<T, MAX>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "a list of at most {MAX} items")
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = Vec::with_capacity(sequence.size_hint().unwrap_or(0).min(MAX));
                while let Some(value) = sequence.next_element()? {
                    if values.len() == MAX {
                        return Err(serde::de::Error::custom("collection limit exceeded"));
                    }
                    values.push(value);
                }
                Ok(BoundedVec(values))
            }
        }
        deserializer.deserialize_seq(VecVisitor::<T, MAX>(PhantomData))
    }
}
