use serde::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};

const MAX_SCAN_DEPTH: usize = 32;
const MAX_SCAN_ELEMENTS: usize = 16_384;
const MAX_KEY_BYTES: usize = 256;

pub(super) fn forbidden_field_present(body: &[u8]) -> Result<bool, serde_json::Error> {
    let mut state = ScanState::default();
    let mut deserializer = serde_json::Deserializer::from_slice(body);
    ScanSeed {
        state: &mut state,
        depth: 0,
        count_element: false,
    }
    .deserialize(&mut deserializer)?;
    deserializer.end()?;
    Ok(state.forbidden_field_present)
}

#[derive(Default)]
struct ScanState {
    elements: usize,
    forbidden_field_present: bool,
}

impl ScanState {
    fn count<E: Error>(&mut self) -> Result<(), E> {
        if self.elements >= MAX_SCAN_ELEMENTS {
            return Err(E::custom("too many JSON elements"));
        }
        self.elements += 1;
        Ok(())
    }
}

struct ScanSeed<'state> {
    state: &'state mut ScanState,
    depth: usize,
    count_element: bool,
}

impl<'de> DeserializeSeed<'de> for ScanSeed<'_> {
    type Value = ();

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        if self.depth > MAX_SCAN_DEPTH {
            return Err(D::Error::custom("JSON nesting is too deep"));
        }
        if self.count_element {
            self.state.count()?;
        }
        deserializer.deserialize_any(ScanVisitor {
            state: self.state,
            depth: self.depth,
        })
    }
}

struct ScanVisitor<'state> {
    state: &'state mut ScanState,
    depth: usize,
}

impl<'de> Visitor<'de> for ScanVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("bounded JSON")
    }

    fn visit_bool<E: Error>(self, _value: bool) -> Result<(), E> {
        Ok(())
    }

    fn visit_i64<E: Error>(self, _value: i64) -> Result<(), E> {
        Ok(())
    }

    fn visit_u64<E: Error>(self, _value: u64) -> Result<(), E> {
        Ok(())
    }

    fn visit_f64<E: Error>(self, _value: f64) -> Result<(), E> {
        Ok(())
    }

    fn visit_str<E: Error>(self, _value: &str) -> Result<(), E> {
        Ok(())
    }

    fn visit_string<E: Error>(self, _value: String) -> Result<(), E> {
        Ok(())
    }

    fn visit_none<E: Error>(self) -> Result<(), E> {
        Ok(())
    }

    fn visit_unit<E: Error>(self) -> Result<(), E> {
        Ok(())
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<(), A::Error> {
        let child_depth = self.depth.saturating_add(1);
        while sequence
            .next_element_seed(ScanSeed {
                state: &mut *self.state,
                depth: child_depth,
                count_element: true,
            })?
            .is_some()
        {}
        Ok(())
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
        let child_depth = self.depth.saturating_add(1);
        while let Some(forbidden) = map.next_key_seed(KeySeed)? {
            self.state.forbidden_field_present |= forbidden;
            map.next_value_seed(ScanSeed {
                state: &mut *self.state,
                depth: child_depth,
                count_element: true,
            })?;
        }
        Ok(())
    }
}

struct KeySeed;

impl<'de> DeserializeSeed<'de> for KeySeed {
    type Value = bool;

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_identifier(KeyVisitor)
    }
}

struct KeyVisitor;

impl Visitor<'_> for KeyVisitor {
    type Value = bool;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a bounded JSON key")
    }

    fn visit_str<E: Error>(self, value: &str) -> Result<bool, E> {
        classify_key(value)
    }

    fn visit_string<E: Error>(self, value: String) -> Result<bool, E> {
        classify_key(&value)
    }
}

fn classify_key<E: Error>(key: &str) -> Result<bool, E> {
    if key.len() > MAX_KEY_BYTES {
        return Err(E::custom("JSON key is too long"));
    }
    Ok(matches!(
        key,
        "access_token" | "refresh_token" | "authorization"
    ))
}
