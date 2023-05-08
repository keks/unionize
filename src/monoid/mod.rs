pub mod count;
pub mod hashxor;
pub mod sum;

use std::fmt::{Debug, Display};

pub trait Monoid: Clone + Debug + Eq {
    type Item: Item;

    fn neutral() -> Self;
    fn lift(item: &Self::Item) -> Self;
    fn combine(&self, other: &Self) -> Self;
}

pub trait FormatMonoid: Monoid
where
    Self::Item: DisplayItem,
{
}

pub trait Item: Clone + Ord + Debug {}

pub trait DisplayItem: Item + Display {}
