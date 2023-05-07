use crate::LiftingMonoid;

use super::FormattingMonoid;

pub trait SumItem: Clone + std::fmt::Debug + Eq + Ord + std::ops::Add<Output = Self> {
    fn zero() -> Self;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SumMonoid<T: SumItem>(pub T);

impl<T: SumItem> LiftingMonoid for SumMonoid<T> {
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

impl<T: SumItem> SumMonoid<T> {
    pub fn sum(&self) -> &T {
        let SumMonoid(sum) = self;
        sum
    }
}

impl<T: SumItem> FormattingMonoid for SumMonoid<T> {
    fn item_to_string(item: &Self::Item) -> String {
        format!("{item:?}")
    }
}
