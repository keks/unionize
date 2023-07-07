use super::{Node, NodeData};
use crate::monoid::Monoid;
use crate::Node as NodeTrait;

impl<M> NodeTrait<M> for Node<M>
where
    M: Monoid,
{
    type NonNilNodeRef<'a> = NonNilNodeRef<'a, M> where M: 'a;

    fn monoid(&self) -> &M {
        match self {
            Node::Node2(node_data) => &node_data.total,
            Node::Node3(node_data) => &node_data.total,
            Node::Nil(m) => m,
        }
    }

    fn is_nil(&self) -> bool {
        matches!(self, Node::Nil(_))
    }

    fn node_contents<'a>(&'a self) -> Option<NonNilNodeRef<'a, M>> {
        match self {
            Node::Node2(node_data) => Some(NonNilNodeRef::Node2(&node_data)),
            Node::Node3(node_data) => Some(NonNilNodeRef::Node3(&node_data)),
            Node::Nil(_) => None,
        }
    }
}

#[derive(Clone)]
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
        let opt_res = match &self.node {
            NonNilNodeRef::Node2(node_data) => (
                node_data.children.get(self.offs),
                node_data.items.get(self.offs),
            ),
            NonNilNodeRef::Node3(node_data) => (
                node_data.children.get(self.offs),
                node_data.items.get(self.offs),
            ),
        };

        let res = match opt_res {
            (Some(child), Some(item)) => Some((child.as_ref(), item)),
            (None, None) => None,
            _ => unreachable!(),
        };
        self.offs += 1;

        res
    }
}

#[derive(Debug, Clone)]
pub enum NonNilNodeRef<'a, M: Monoid> {
    Node2(&'a NodeData<M, 1>),
    Node3(&'a NodeData<M, 2>),
}

impl<'a, M> crate::NonNilNodeRef<'a, M, Node<M>> for NonNilNodeRef<'a, M>
where
    M: Monoid + 'a,
{
    type ChildIter<'b> = ChildIter<'b, M>
    where
        M::Item: 'b,
        Self: 'b;

    fn min(&self) -> &<M as Monoid>::Item {
        match self {
            NonNilNodeRef::Node2(node_data) => &node_data.min,
            NonNilNodeRef::Node3(node_data) => &node_data.min,
        }
    }

    fn max(&self) -> &<M as Monoid>::Item {
        match self {
            NonNilNodeRef::Node2(node_data) => &node_data.max,
            NonNilNodeRef::Node3(node_data) => &node_data.max,
        }
    }

    fn bounds(&self) -> (&<M as Monoid>::Item, &<M as Monoid>::Item) {
        match self {
            NonNilNodeRef::Node2(node_data) => (&node_data.min, &node_data.max),
            NonNilNodeRef::Node3(node_data) => (&node_data.min, &node_data.max),
        }
    }

    fn children<'b>(&'b self) -> Self::ChildIter<'b> {
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
}
