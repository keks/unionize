use super::Node;
use crate::monoid::Monoid;
use crate::Node as NodeTrait;

impl<'a, M> NodeTrait<'a, M> for Node<M>
where
    M: Monoid + 'a,
    Self: 'a,
{
    type ChildIter = ChildIter<'a, M>;

    fn monoid(&self) -> &M {
        self.monoid()
    }

    fn is_nil(&self) -> bool {
        matches!(self, Node::Nil(_))
    }

    fn min_item(&self) -> Option<&M::Item> {
        match self {
            Node::Node2(node_data) if self.is_leaf() => Some(&node_data.items[0]),
            Node::Node3(node_data) if self.is_leaf() => Some(&node_data.items[0]),
            Node::Node2(node_data) => Some(&node_data.min),
            Node::Node3(node_data) => Some(&node_data.min),
            Node::Nil(_) => None,
        }
    }

    fn max_item(&self) -> Option<&M::Item> {
        match self {
            Node::Node2(node_data) if self.is_leaf() => Some(&node_data.items[0]),
            Node::Node3(node_data) if self.is_leaf() => Some(&node_data.items[1]),
            Node::Node2(node_data) => Some(&node_data.max),
            Node::Node3(node_data) => Some(&node_data.max),
            Node::Nil(_) => None,
        }
    }

    fn children(&'a self) -> Option<Self::ChildIter> {
        Some(ChildIter(self, 0))
    }

    fn last_child(&self) -> Option<&Self> {
        match self {
            Node::Node2(node_data) => Some(&node_data.last_child),
            Node::Node3(node_data) => Some(&node_data.last_child),
            Node::Nil(_) => None,
        }
    }
}

pub struct ChildIter<'a, M: Monoid>(&'a Node<M>, usize);

impl<'a, M> Iterator for ChildIter<'a, M>
where
    M: Monoid + 'a,
{
    type Item = (&'a Node<M>, &'a M::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let ChildIter(node, ref mut offs) = self;

        let res = match node {
            Node::Node2(node_data) => {
                Some((node_data.children.get(*offs), node_data.items.get(*offs)))
            }
            Node::Node3(node_data) => {
                Some((node_data.children.get(*offs), node_data.items.get(*offs)))
            }
            Node::Nil(_) => None,
        }
        .map(|x| match x {
            (Some(child), Some(item)) => Some((child.as_ref(), item)),
            (None, None) => None,
            _ => unreachable!(),
        })
        .flatten();

        *offs += 1;

        res
    }
}
