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

    fn bounds(&self) -> Option<(&M::Item, &M::Item)> {
        match self {
            Node::Node2(node_data) if self.is_leaf() => {
                Some((&node_data.items[0], &node_data.items[0]))
            }
            Node::Node3(node_data) if self.is_leaf() => {
                Some((&node_data.items[0], &node_data.items[1]))
            }
            Node::Node2(node_data) => Some((
                node_data.children[0].min_item()?,
                node_data.last_child.max_item()?,
            )),
            Node::Node3(node_data) => Some((
                node_data.children[0].min_item()?,
                node_data.last_child.max_item()?,
            )),
            Node::Nil(_) => None,
        }
    }

    fn min_item(&self) -> Option<&M::Item> {
        match self {
            Node::Node2(node_data) if self.is_leaf() => Some(&node_data.items[0]),
            Node::Node3(node_data) if self.is_leaf() => Some(&node_data.items[0]),
            Node::Node2(node_data) => node_data.children[0].min_item(),
            Node::Node3(node_data) => node_data.children[0].min_item(),
            Node::Nil(_) => None,
        }
    }

    fn max_item(&self) -> Option<&M::Item> {
        match self {
            Node::Node2(node_data) if self.is_leaf() => Some(&node_data.items[0]),
            Node::Node3(node_data) if self.is_leaf() => Some(&node_data.items[1]),
            Node::Node2(node_data) => node_data.last_child.max_item(),
            Node::Node3(node_data) => node_data.last_child.max_item(),
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

    type NonNilNode = NonNilNode<'a, M>;

    fn node_contents(&'a self) -> Option<Self::NonNilNode> {
        match self {
            Node::Node2(node_data) => Some(NonNilNode::Node2(node_data)),
            Node::Node3(node_data) => Some(NonNilNode::Node3(node_data)),
            Node::Nil(_) => todo!(),
        }
    }
}

pub struct ChildIter<'a, M: Monoid> {
    node: &'a Node<M>,
    offs: usize,
}

impl<'a, M> Iterator for ChildIter<'a, M>
where
    M: Monoid + 'a,
{
    type Item = (&'a Node<M>, &'a M::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let res = match self.node {
            Node::Node2(node_data) => Some((
                node_data.children.get(self.offs),
                node_data.items.get(self.offs),
            )),
            Node::Node3(node_data) => Some((
                node_data.children.get(self.offs),
                node_data.items.get(self.offs),
            )),
            Node::Nil(_) => None,
        }
        .map(|x| match x {
            (Some(child), Some(item)) => Some((child.as_ref(), item)),
            (None, None) => None,
            _ => unreachable!(),
        })
        .flatten();

        self.offs += 1;

        res
    }
}

#[derive(Clone, Debug)]
pub enum NonNilNode<'a, M: Monoid> {
    Node2(&'a super::NodeData<M, 1>),
    Node3(&'a super::NodeData<M, 2>),
}

impl<'a, M> crate::NonNilNode<'a, M, Node<M>> for NonNilNode<'a, M>
where
    M: Monoid + 'a,
{
    type ChildIter = ChildIter<'a, M>;

    fn min(&self) -> &<M as Monoid>::Item {
        todo!()
    }

    fn max(&self) -> &<M as Monoid>::Item {
        todo!()
    }

    fn children(&self) -> Self::ChildIter {
        todo!()
    }

    fn last_child(&self) -> &Self {
        todo!()
    }
}
