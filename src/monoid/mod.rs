pub mod count;
pub mod hashxor;
pub mod sum;

use std::fmt::Debug;

pub trait Monoid: Clone + Debug + Eq {
    type Item: Item;

    fn neutral() -> Self;
    fn lift(item: &Self::Item) -> Self;
    fn combine(&self, other: &Self) -> Self;
}

pub trait Item: Clone + Debug + Ord {}
