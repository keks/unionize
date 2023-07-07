use super::Node;
use crate::monoid::Monoid;

use crate::{Node as NodeTrait, NonNilNodeRef as NonNilNodeRefTrait};

impl<M: Monoid> NodeTrait<M> for Node<M> {
    fn monoid(&self) -> &M {
        self.monoid()
    }

    fn is_nil(&self) -> bool {
        matches!(self, Node::Nil(_))
    }

    type NonNilNodeRef<'a> = NonNilNodeRef<'a, M> where M: 'a;

    fn node_contents<'a>(&'a self) -> Option<Self::NonNilNodeRef<'a>> {
        match self {
            Node::Node2(node_data) => Some(NonNilNodeRef::Node2(node_data)),
            Node::Node3(node_data) => Some(NonNilNodeRef::Node3(node_data)),
            Node::Nil(_) => None,
        }
    }
}

pub struct ChildIter<'a, M: Monoid> {
    node: NonNilNodeRef<'a, M>,
    offs: usize,
}

impl<'a, M> Iterator for ChildIter<'a, M>
where
    M: Monoid + 'a,
{
    type Item = (&'a Node<M>, &'a M::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let res_opt = match self.node {
            NonNilNodeRef::Node2(node_data) => (
                node_data.children.get(self.offs),
                node_data.items.get(self.offs),
            ),
            NonNilNodeRef::Node3(node_data) => (
                node_data.children.get(self.offs),
                node_data.items.get(self.offs),
            ),
        };
        let res = match res_opt {
            (Some(child), Some(item)) => Some((child.as_ref(), item)),
            (None, None) => None,
            _ => unreachable!(),
        };

        self.offs += 1;

        res
    }
}

#[derive(Clone, Debug)]
pub enum NonNilNodeRef<'a, M: Monoid> {
    Node2(&'a super::NodeData<M, 1>),
    Node3(&'a super::NodeData<M, 2>),
}

impl<'a, M> NonNilNodeRefTrait<'a, M, Node<M>> for NonNilNodeRef<'a, M>
where
    M: Monoid + 'a,
{
    type ChildIter<'b> = ChildIter<'b, M> where M: 'b, Self: 'b;

    fn min(&self) -> &'a <M as Monoid>::Item {
        match self {
            NonNilNodeRef::Node2(node_data) => node_data.min_item(),
            NonNilNodeRef::Node3(node_data) => node_data.min_item(),
        }
    }

    fn max(&self) -> &<M as Monoid>::Item {
        match self {
            NonNilNodeRef::Node2(node_data) => node_data.max_item(),
            NonNilNodeRef::Node3(node_data) => node_data.max_item(),
        }
    }

    fn children(&self) -> Self::ChildIter<'a> {
        ChildIter {
            node: self.clone(),
            offs: 0,
        }
    }

    fn last_child<'b>(&'b self) -> &'b Node<M> {
        match self {
            NonNilNodeRef::Node2(node_data) => &node_data.last_child,
            NonNilNodeRef::Node3(node_data) => &node_data.last_child,
        }
    }

    fn bounds(&self) -> (&<M as Monoid>::Item, &<M as Monoid>::Item) {
        match self {
            NonNilNodeRef::Node2(node_data) => node_data.bounds(),
            NonNilNodeRef::Node3(node_data) => node_data.bounds(),
        }
    }
}
