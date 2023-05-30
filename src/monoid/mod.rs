pub mod count;
pub mod hashxor;
pub mod mulhash_xs233;
pub mod sum;

use std::fmt::Debug;

pub trait Monoid: Clone + Debug + Eq {
    type Item: Item;

    fn neutral() -> Self;
    fn lift(item: &Self::Item) -> Self;
    fn combine(&self, other: &Self) -> Self;
}

pub trait Item: Clone + Debug + Ord + Peano {}

pub trait Peano {
    fn zero() -> Self;
    fn next(&self) -> Self;
}

impl Peano for u64 {
    fn zero() -> Self {
        0
    }

    fn next(&self) -> Self {
        self + 1
    }
}
