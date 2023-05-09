use serde::Serialize;

use crate::{monoid::Monoid, proto::ProtocolMonoid};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountingMonoid<M: Monoid>(usize, M);

impl<M: Monoid> CountingMonoid<M> {
    pub fn inner(&self) -> &M {
        &self.1
    }
}

impl<M: Monoid> ProtocolMonoid for CountingMonoid<M>
where
    M::Item: Serialize,
{
    type ProtocolItem = M::Item;

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
