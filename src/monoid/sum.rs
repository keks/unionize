use super::{Item, Monoid};

pub trait SumItem: Item + std::ops::Add<Output = Self> {
    fn zero() -> Self;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SumMonoid<I: SumItem>(pub I);

impl<I: SumItem> SumMonoid<I> {
    pub fn sum(&self) -> &I {
        &self.0
    }
}

impl<I: SumItem> Monoid for SumMonoid<I> {
    type Item = I;

    fn neutral() -> Self {
        SumMonoid(I::zero())
    }

    fn lift(item: &Self::Item) -> Self {
        SumMonoid(item.clone())
    }

    fn combine(&self, other: &Self) -> Self {
        let (SumMonoid(lhs), SumMonoid(rhs)) = (self, other);
        SumMonoid(lhs.clone() + rhs.clone())
    }
}
