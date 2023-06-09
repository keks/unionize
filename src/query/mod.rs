pub mod item_filter;
pub mod items;
pub mod simple;
pub mod split;

use crate::{item::Item, monoid::Monoid, Node};

pub trait Accumulator<M>: core::fmt::Debug
where
    M: Monoid,
    M::Item: Item,
{
    fn add_node<'a, N: Node<M>>(&mut self, node: &'a N);

    fn add_item(&mut self, item: &M::Item);

    fn finalize(&mut self) {}
}
