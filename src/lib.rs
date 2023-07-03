pub mod proto;
pub mod range;

pub mod hash_item;
pub mod monoid;
pub mod query;
mod tree;

pub trait XNode<'a, M>: std::fmt::Debug + Clone
where
    M: monoid::Monoid + 'a,
    Self: 'a,
{
    type ChildIter: Iterator<Item = (&'a Self, &'a M::Item)>;

    fn monoid(&self) -> &M;
    fn is_nil(&self) -> bool;

    fn min_item(&self) -> Option<&M::Item>;
    fn max_item(&self) -> Option<&M::Item>;

    fn children(&'a self) -> Option<Self::ChildIter>;
    fn last_child(&self) -> Option<&Self>;
}

mod ranged_node;

pub use tree::{Node, Tree};
