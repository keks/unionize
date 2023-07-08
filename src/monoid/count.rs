use crate::{
    monoid::Monoid,
    proto::{DecodeError, Encodable, EncodeError, ProtocolMonoid},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountingMonoid<M: Monoid>(usize, M);

impl<M: Monoid> CountingMonoid<M> {
    pub fn inner(&self) -> &M {
        &self.1
    }
}

impl<M: Monoid + Default> Default for CountingMonoid<M> {
    fn default() -> Self {
        CountingMonoid(0, M::default())
    }
}

impl<M: Monoid + Encodable> ProtocolMonoid for CountingMonoid<M> {
    fn count(&self) -> usize {
        self.0
    }
}

impl<M: Monoid + Encodable> Encodable for CountingMonoid<M> {
    type Encoded = (usize, M::Encoded);

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
