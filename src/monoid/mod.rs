pub mod count;
pub mod hashxor;
pub mod mulhash_xs233;
pub mod sum;
pub mod timestamped;

use core::fmt::Debug;

use crate::item::Item;

/// In math, monoids are things you can combine, and you'll get something of the same type.
/// We use this property for fingerprints, and through an accident in history this is now
/// the name of this type. Might change to Fingerprint before v1.
pub trait Monoid: Clone + Debug + Eq {
    type Item: Item;

    /// Returns a value that, when combined with some other value x, will return x.
    fn neutral() -> Self;

    /// Returns a the value corresponding to the item. Ideally, collisions should be rare,
    /// i.e. they should at least not happen by accident, and probably also be hard to find on
    /// purpose.
    fn lift(item: &Self::Item) -> Self;

    /// Returns the combination of two items. Not necessarily commutative.
    /// A good fingerprint monoid will make it difficult to produce the same output with different
    /// inputs (commutativity is allowed).
    fn combine(&self, other: &Self) -> Self;
}
