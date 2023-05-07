pub mod count;
pub mod hashxor;
pub mod sum;

use std::fmt::Debug;

pub trait LiftingMonoid: Clone + Debug + Eq {
    type Item: Clone + Debug + Ord;

    fn neutral() -> Self;
    fn lift(item: &Self::Item) -> Self;
    fn combine(&self, other: &Self) -> Self;
}

pub trait FormattingMonoid: LiftingMonoid {
    fn item_to_string(item: &Self::Item) -> String;
}

pub trait Monoid2: Clone + Debug + Eq {
    type Item: Item;

    fn neutral() -> Self;
    fn lift(item: &Self::Item) -> Self;
    fn combine(&self, other: &Self) -> Self;
}

pub trait FormatMonoid2: Monoid2
where
    Self::Item: FormatItem,
{
}

pub trait Item: Clone + Ord + Debug {
    fn zero() -> Self;
}

pub trait FormatItem: Item + ToString {}
