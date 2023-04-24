use crate::LiftingMonoid;
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountingMonoid<T: Clone + std::fmt::Debug + Ord + Eq>(usize, PhantomData<T>);

impl<T: Clone + std::fmt::Debug + Ord + Eq> LiftingMonoid for CountingMonoid<T> {
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

impl<T: Clone + std::fmt::Debug + Ord + Eq> CountingMonoid<T> {
    pub fn count(&self) -> usize {
        self.0
    }
}
