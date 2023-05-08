use super::{Item, Monoid};

pub trait SumItem: Item + std::ops::Add<Output = Self> {
    fn zero() -> Self;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SumMonoid<T: SumItem>(pub T);

impl<T: SumItem> SumMonoid<T> {
    pub fn sum(&self) -> &T {
        &self.0
    }
}

impl<T: SumItem> Monoid for SumMonoid<T> {
    type Item = T;

    fn neutral() -> Self {
        SumMonoid(T::zero())
    }

    fn lift(item: &Self::Item) -> Self {
        SumMonoid(item.clone())
    }

    fn combine(&self, other: &Self) -> Self {
        let (SumMonoid(lhs), SumMonoid(rhs)) = (self, other);
        SumMonoid(lhs.clone() + rhs.clone())
    }
}
