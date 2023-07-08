use crate::{
    monoid::Monoid,
    protocol::{DecodeError, Encodable, EncodeError, ProtocolMonoid},
};

use serde::{Deserialize, Serialize};

/// Wraps another monoid and attaches an item counter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountingMonoid<M: Monoid>(usize, M);

impl<M: Monoid> CountingMonoid<M> {
    #[allow(dead_code)] // is actually used, but in tests
    pub(crate) fn new(count: usize, fp: M) -> Self {
        CountingMonoid(count, fp)
    }

    pub fn inner(&self) -> &M {
        &self.1
    }
}

impl<M: Monoid + Default> Default for CountingMonoid<M> {
    fn default() -> Self {
        CountingMonoid(0, M::default())
    }
}

impl<M> ProtocolMonoid for CountingMonoid<M>
where
    M: Monoid + Encodable,
    M::Item: Serialize,
{
    fn count(&self) -> usize {
        self.0
    }
}

impl<M: Monoid> Monoid for CountingMonoid<M> {
    type Item = M::Item;

    fn neutral() -> Self {
        CountingMonoid(0, M::neutral())
    }

    fn lift(item: &Self::Item) -> Self {
        CountingMonoid(1, M::lift(item))
    }

    fn combine(&self, other: &Self) -> Self {
        CountingMonoid(self.0 + other.0, M::combine(&self.1, &other.1))
    }
}

impl<M: Monoid + Encodable> Encodable for CountingMonoid<M> {
    type Encoded = EncodedCountingMonoid<M>;

    type EncodeError = M::EncodeError;
    type DecodeError = M::DecodeError;

    fn encode(&self, encoded: &mut Self::Encoded) -> Result<(), EncodeError<Self::EncodeError>> {
        encoded.0 = self.0;
        M::encode(&self.1, &mut encoded.1)
    }

    fn decode(&mut self, encoded: &Self::Encoded) -> Result<(), DecodeError<Self::DecodeError>> {
        self.0 = encoded.0;
        M::decode(&mut self.1, &encoded.1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncodedCountingMonoid<M: Monoid + Encodable>(usize, M::Encoded);

impl<M> Default for EncodedCountingMonoid<M>
where
    M: Monoid + Encodable,
{
    fn default() -> Self {
        EncodedCountingMonoid(0, M::Encoded::default())
    }
}

// impl<M> Serialize for CountingMonoid<M>
// where
//     M: ProtocolMonoid + Serialize,
// {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let mut tup = serializer.serialize_tuple(2)?;
//         tup.serialize_element(&self.0)?;
//         tup.serialize_element(&self.1)?;
//         tup.end()
//     }
// }
