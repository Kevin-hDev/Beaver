use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

use super::{AppReleaseNotesByLocale, MAX_BULLETS, MAX_VERSION_ENTRIES};

pub(super) fn parse(bytes: &[u8], version: &str) -> Option<AppReleaseNotesByLocale> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let notes = NotesRootSeed { version }
        .deserialize(&mut deserializer)
        .ok()??;
    deserializer.end().ok()?;
    Some(notes)
}

struct NotesRootSeed<'a> {
    version: &'a str,
}

impl<'de> DeserializeSeed<'de> for NotesRootSeed<'_> {
    type Value = Option<AppReleaseNotesByLocale>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(NotesRootVisitor {
            version: self.version,
        })
    }
}

struct NotesRootVisitor<'a> {
    version: &'a str,
}

impl<'de> Visitor<'de> for NotesRootVisitor<'_> {
    type Value = Option<AppReleaseNotesByLocale>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("un objet de notes borné")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let expected_prefixed = format!("v{}", self.version);
        let mut seen = BTreeSet::new();
        let mut selected = None;
        while let Some(key) = map.next_key::<String>()? {
            if seen.len() >= MAX_VERSION_ENTRIES
                || key.len() > 64
                || key.chars().any(|character| character.is_control())
                || !seen.insert(key.clone())
            {
                return Err(serde::de::Error::custom("clé de version invalide"));
            }
            let notes = map.next_value::<BoundedLocales>()?.0;
            if (key == self.version || key == expected_prefixed)
                && selected.replace(notes).is_some()
            {
                return Err(serde::de::Error::custom("version dupliquée"));
            }
        }
        Ok(selected)
    }
}

struct BoundedLocales(AppReleaseNotesByLocale);

impl<'de> Deserialize<'de> for BoundedLocales {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(LocalesVisitor)
    }
}

struct LocalesVisitor;

impl<'de> Visitor<'de> for LocalesVisitor {
    type Value = BoundedLocales;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("sept listes de notes")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut locales = BTreeMap::new();
        while let Some(locale) = map.next_key::<String>()? {
            if locales.len() >= 7 || locale.len() > 8 {
                return Err(serde::de::Error::custom("trop de langues"));
            }
            let items = map.next_value::<BoundedBullets>()?.0;
            if locales.insert(locale, items).is_some() {
                return Err(serde::de::Error::custom("langue dupliquée"));
            }
        }
        Ok(BoundedLocales(locales))
    }
}

struct BoundedBullets(Vec<String>);

impl<'de> Deserialize<'de> for BoundedBullets {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(BulletsVisitor)
    }
}

struct BulletsVisitor;

impl<'de> Visitor<'de> for BulletsVisitor {
    type Value = BoundedBullets;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("une liste bornée de notes")
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut items = Vec::with_capacity(MAX_BULLETS);
        while items.len() < MAX_BULLETS {
            let Some(item) = sequence.next_element()? else {
                return Ok(BoundedBullets(items));
            };
            items.push(item);
        }
        if sequence.next_element::<serde::de::IgnoredAny>()?.is_some() {
            return Err(serde::de::Error::custom("trop de notes"));
        }
        Ok(BoundedBullets(items))
    }
}
