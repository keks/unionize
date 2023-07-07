pub mod count;
pub mod hashxor;
pub mod mulhash_xs233;
pub mod sum;

use core::fmt::Debug;

use crate::item::Item;

pub trait Monoid: Clone + Debug + Eq {
    type Item: Item;

    fn neutral() -> Self;
    fn lift(item: &Self::Item) -> Self;
    fn combine(&self, other: &Self) -> Self;
}
