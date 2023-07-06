pub mod generic;
pub mod items;
pub mod simple;
pub mod split;

use crate::{
    monoid::{Item, Monoid},
    Node,
};

pub trait Accumulator<M>: std::fmt::Debug
where
    M: Monoid,
    M::Item: Item,
{
    fn add_xnode<'a, N: Node<'a, M>>(&mut self, node: &'a N)
    where
        M: 'a;
    fn add_item(&mut self, item: &M::Item);
}

#[cfg(test)]
pub mod test {
    use crate::{
        monoid::count::CountingMonoid,
        monoid::sum::{SumItem, SumMonoid},
    };
    pub type TestMonoid<T> = CountingMonoid<SumMonoid<T>>;

    impl SumItem for u64 {
        fn zero() -> u64 {
            0
        }
    }
}
