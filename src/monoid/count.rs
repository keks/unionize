use std::marker::PhantomData;

use crate::proto::ProtocolMonoid;

use super::{Item, Monoid};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountingMonoid<T: Clone + std::fmt::Debug + Ord + Eq>(usize, PhantomData<T>);

impl<T: Item> ProtocolMonoid for CountingMonoid<T> {
    fn count(&self) -> usize {
        self.0
    }
}

impl<T: Item> Monoid for CountingMonoid<T> {
    type Item = T;

    fn neutral() -> Self {
        CountingMonoid(0, PhantomData)
    }

    fn lift(_item: &Self::Item) -> Self {
        CountingMonoid(1, PhantomData)
    }

    fn combine(&self, other: &Self) -> Self {
        CountingMonoid(self.0 + other.0, PhantomData)
    }
}
