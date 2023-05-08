use std::rc::Rc;

use crate::{monoid::Monoid, range::Range, tree::ChildId, Node};

#[derive(Debug, Clone)]
pub struct RangedRcNode<'a, M>
where
    M: Monoid,
{
    node: &'a Rc<Node<M>>,
    range: Range<M::Item>,
}

impl<'a, M> RangedRcNode<'a, M>
where
    M: Monoid,
{
    pub fn new(node: &'a Rc<Node<M>>, range: Range<M::Item>) -> Self {
        Self { node, range }
    }

    pub fn rc_node(&self) -> &Rc<Node<M>> {
        self.node
    }

    pub fn node(&self) -> &Node<M> {
        &self.node
    }

    pub fn range(&self) -> &Range<M::Item> {
        &self.range
    }

    pub fn ranged_node(&self) -> RangedNode<'a, M> {
        let RangedRcNode { node, range } = self;
        RangedNode {
            node,
            range: range.clone(),
        }
    }
}

impl<'a, M> From<RangedRcNode<'a, M>> for RangedNode<'a, M>
where
    M: Monoid,
{
    fn from(value: RangedRcNode<'a, M>) -> Self {
        let RangedRcNode { node, range } = value;
        RangedNode { node, range }
    }
}

#[derive(Clone, Debug)]
pub struct RangedNode<'a, M>
where
    M: Monoid,
{
    node: &'a Node<M>,
    range: Range<M::Item>,
}

impl<'a, M> RangedNode<'a, M>
where
    M: Monoid,
{
    pub fn new(node: &'a Node<M>, range: Range<M::Item>) -> Self {
        RangedNode { node, range }
    }

    pub fn node(&self) -> &Node<M> {
        self.node
    }

    pub fn range(&self) -> &Range<M::Item> {
        &self.range
    }

    pub fn rc_children(&self) -> RcChildren<'a, M> {
        RcChildren {
            child_id: ChildId::Normal(0),
            node: self.clone(),
        }
    }

    pub fn children(&self) -> Children<'a, M> {
        Children {
            iter: self.rc_children(),
        }
    }

    pub fn rc_last_child(&self) -> RangedRcNode<'a, M> {
        let Range(_, to) = &self.range;
        match &self.node {
            Node::Node2(_) | Node::Node3(_) => RangedRcNode {
                node: self.node.last_child(),
                range: Range(self.node.last_item().clone(), to.clone()),
            },
            Node::Nil(_) => panic!("nil node doesn't have last child"),
        }
    }

    pub fn last_child(&self) -> RangedNode<'a, M> {
        self.rc_last_child().ranged_node()
    }

    pub fn get_child(&self, child_id: ChildId) -> Option<RangedRcNode<'a, M>> {
        let Range(from, to) = &self.range;
        match (&self.node, child_id) {
            // failure cases first, for visibility
            (Node::Node2(_node_data), ChildId::Normal(offs)) if offs > 0 => None,
            (Node::Node3(_node_data), ChildId::Normal(offs)) if offs > 1 => None,
            (Node::Nil(_), _) => None,

            (Node::Node2(node_data), ChildId::Normal(offs)) => {
                let item = &node_data.items()[offs];
                let child = &node_data.children().0[offs];
                Some(RangedRcNode {
                    node: child,
                    range: Range(from.clone(), item.clone()),
                })
            }
            (Node::Node2(node_data), ChildId::Last) => {
                let item = &node_data.items()[0];
                let child = node_data.children().1;
                Some(RangedRcNode {
                    node: child,
                    range: Range(item.clone(), to.clone()),
                })
            }
            (Node::Node3(node_data), ChildId::Normal(offs)) => {
                let from_item = if offs == 0 {
                    from
                } else {
                    &node_data.items()[offs - 1]
                };
                let to_item = &node_data.items()[offs];
                let child = &node_data.children().0[offs];
                Some(RangedRcNode {
                    node: child,
                    range: Range(from_item.clone(), to_item.clone()),
                })
            }
            (Node::Node3(node_data), ChildId::Last) => {
                let item = &node_data.items()[1];
                let child = node_data.children().1;
                Some(RangedRcNode {
                    node: child,
                    range: Range(item.clone(), to.clone()),
                })
            }
        }
    }
}

pub struct Children<'a, M: Monoid> {
    iter: RcChildren<'a, M>,
}

impl<'a, M: Monoid> Iterator for Children<'a, M> {
    type Item = (RangedNode<'a, M>, M::Item);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(rc_node, item)| (rc_node.ranged_node(), item))
    }
}

pub struct RcChildren<'a, M>
where
    M: Monoid,
{
    child_id: ChildId,
    node: RangedNode<'a, M>,
}

impl<'a, M> Iterator for RcChildren<'a, M>
where
    M: Monoid,
{
    type Item = (RangedRcNode<'a, M>, M::Item);

    fn next(&mut self) -> Option<Self::Item> {
        match self.child_id {
            ChildId::Normal(idx) => {
                let child = self.node.get_child(self.child_id);
                let item = self.node.node().get_item(idx);

                // either they are both none or both some
                assert_eq!(child.is_none(), item.is_none());

                if child.is_none() {
                    self.child_id = ChildId::Last;
                    let child = self.node.get_child(ChildId::Last);
                    child.zip(item)
                } else {
                    self.child_id = ChildId::Normal(idx + 1);
                    child.zip(item)
                }
            }
            ChildId::Last => None,
        }
    }
}

// macro_rules! impl_Node_on_RangedNode {
// ($func_name:ident . $($arg_name:ident: $arg_type:ty),*) => {
//   pub fn $func_name(&self, $($arg_name: $arg_type),+) {
// self.node().$func_name($($arg_name),*)
//   }
// };
// ($func_name:ident . $($arg_name:ident: $arg_type:ty),* => $ret_type:ty) => {
//   pub fn $func_name(&self, $($arg_name: $arg_type),*) -> $ret_type{
// self.node().$func_name($($arg_name),*)
//   }
// };
// }
