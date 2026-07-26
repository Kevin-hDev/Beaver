use std::fmt;
use std::marker::PhantomData;

use serde::de::{Error, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

#[derive(Debug)]
pub struct BoundedVec<T, const MAX: usize>(Vec<T>);

impl<T, const MAX: usize> BoundedVec<T, MAX> {
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }
}

impl<T, const MAX: usize> TryFrom<Vec<T>> for BoundedVec<T, MAX> {
    type Error = &'static str;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        if value.len() > MAX {
            Err("collection trop grande")
        } else {
            Ok(Self(value))
        }
    }
}

impl<'a, T, const MAX: usize> IntoIterator for &'a BoundedVec<T, MAX> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'de, T, const MAX: usize> Deserialize<'de> for BoundedVec<T, MAX>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(BoundedVisitor::<T, MAX>(PhantomData))
    }
}

struct BoundedVisitor<T, const MAX: usize>(PhantomData<T>);

impl<'de, T, const MAX: usize> Visitor<'de> for BoundedVisitor<T, MAX>
where
    T: Deserialize<'de>,
{
    type Value = BoundedVec<T, MAX>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "une liste de {MAX} éléments maximum")
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let capacity = sequence.size_hint().unwrap_or(0).min(MAX);
        let mut values = Vec::with_capacity(capacity);
        while let Some(value) = sequence.next_element()? {
            if values.len() == MAX {
                return Err(A::Error::custom("collection trop grande"));
            }
            values.push(value);
        }
        Ok(BoundedVec(values))
    }
}
