pub mod count;
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
