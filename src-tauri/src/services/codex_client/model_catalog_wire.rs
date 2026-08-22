use std::fmt;
use std::marker::PhantomData;

use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

pub(super) const MAX_CATALOG_MODELS: usize = 64;
const MAX_REASONING_MODES: usize = 8;
const MAX_INPUT_MODALITIES: usize = 8;
const MAX_SERVICE_TIERS: usize = 8;

#[derive(Debug, Clone)]
pub(super) struct BoundedVec<T, const LIMIT: usize>(pub Vec<T>);

impl<T, const LIMIT: usize> Default for BoundedVec<T, LIMIT> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<'de, T, const LIMIT: usize> Deserialize<'de> for BoundedVec<T, LIMIT>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BoundedVisitor<T, const LIMIT: usize>(PhantomData<T>);

        impl<'de, T, const LIMIT: usize> Visitor<'de> for BoundedVisitor<T, LIMIT>
        where
            T: Deserialize<'de>,
        {
            type Value = BoundedVec<T, LIMIT>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "une liste de {LIMIT} éléments maximum")
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = Vec::with_capacity(sequence.size_hint().unwrap_or(0).min(LIMIT));
                while let Some(value) = sequence.next_element()? {
                    if values.len() >= LIMIT {
                        return Err(serde::de::Error::custom("collection trop grande"));
                    }
                    values.push(value);
                }
                Ok(BoundedVec(values))
            }
        }

        deserializer.deserialize_seq(BoundedVisitor::<T, LIMIT>(PhantomData))
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct ModelsResponse {
    pub models: BoundedVec<WireModel, MAX_CATALOG_MODELS>,
}

#[derive(Debug, Deserialize)]
pub(super) struct WireModel {
    pub slug: String,
    pub display_name: String,
    #[serde(default)]
    pub supported_reasoning_levels: BoundedVec<ReasoningLevel, MAX_REASONING_MODES>,
    pub visibility: Option<String>,
    pub context_window: Option<u64>,
    pub max_context_window: Option<u64>,
    pub effective_context_window_percent: Option<u64>,
    #[serde(default)]
    pub input_modalities: BoundedVec<String, MAX_INPUT_MODALITIES>,
    #[serde(default)]
    pub service_tiers: BoundedVec<WireServiceTier, MAX_SERVICE_TIERS>,
}

#[derive(Debug, Deserialize)]
pub(super) struct WireServiceTier {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct ReasoningLevel {
    pub effort: String,
}
