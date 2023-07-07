use crate::proto::Encodable;

use super::{Item, Monoid};

pub trait SumItem: Item + core::ops::Add<Output = Self> {}
impl<I> SumItem for I where I: Item + core::ops::Add<Output = Self> {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SumMonoid<I: SumItem>(pub I);

impl<I: SumItem> SumMonoid<I> {
    pub fn sum(&self) -> &I {
        &self.0
    }
}

impl<I: SumItem> Default for SumMonoid<I> {
    fn default() -> Self {
        SumMonoid(<I as Item>::zero())
    }
}

impl<I: SumItem> Encodable for SumMonoid<I> {
    type Encoded = Self;
    type Error = ();

    fn encode(&self, encoded: &mut Self::Encoded) -> Result<(), Self::Error> {
        *encoded = self.clone();
        Ok(())
    }

    fn decode(&mut self, encoded: &Self::Encoded) -> Result<(), Self::Error> {
        *self = encoded.clone();
        Ok(())
    }
}

impl<I: SumItem> Monoid for SumMonoid<I> {
    type Item = I;

    fn neutral() -> Self {
        SumMonoid(<I as Item>::zero())
    }

    fn lift(item: &Self::Item) -> Self {
        SumMonoid(item.clone())
    }

    fn combine(&self, other: &Self) -> Self {
        let (SumMonoid(lhs), SumMonoid(rhs)) = (self, other);
        SumMonoid(lhs.clone() + rhs.clone())
    }
}
