use super::{Node, NodeData};
use crate::monoid::Monoid;

use std::rc::Rc;

enum InsertUpstreamData<M: Monoid> {
    Update2Child(NodeData<M, 1>),
    Update3Child(NodeData<M, 2>),
    Split(M::Item, NodeData<M, 1>, NodeData<M, 1>),
}

impl<M: Monoid> Node<M> {
    pub fn insert(&self, item: M::Item) -> Node<M> {
        // if the tree is empty, replace it with a 2-node
        if let Node::Nil(_) = self {
            let items = [item.clone()];
            let nil = Rc::new(Node::Nil(M::neutral()));
            let children = [nil.clone()];
            return Node::Node2(NodeData::new(items, children, nil));
        }

        // otherwise, do a recusing insert and match the result
        match self.insert_inner(item) {
            InsertUpstreamData::Update2Child(node_data) => Node::Node2(node_data),
            InsertUpstreamData::Update3Child(node_data) => Node::Node3(node_data),
            InsertUpstreamData::Split(middle, left, right) => {
                let items = [middle];
                let children = [Rc::new(Node::Node2(left))];
                let last_child = Rc::new(Node::Node2(right));
                Node::Node2(NodeData::new(items, children, last_child))
            }
        }
    }

    fn insert_inner(&self, item: M::Item) -> InsertUpstreamData<M> {
        // find the leaf where the value belongs
        // when we found it, update it
        if self.is_leaf() {
            match self {
                Node::Node2(node_data) => InsertUpstreamData::Update3Child(
                    node_data.grow(item, Rc::new(Node::Nil(M::neutral()))),
                ),
                Node::Node3(node_data) => {
                    let big_node_data = node_data.grow(item, Rc::new(Node::Nil(M::neutral())));
                    let (middle, left, right) = big_node_data.split();
                    InsertUpstreamData::Split(middle, left, right)
                }
                Node::Nil(_) => unreachable!(),
            }
        } else {
            let (child_id, next) = &self.find_child(&item);
            match (self, next.insert_inner(item)) {
                (Node::Node2(node_data), InsertUpstreamData::Update2Child(new_child)) => {
                    let rc_new_child = Rc::new(Node::Node2(new_child));
                    InsertUpstreamData::Update2Child(
                        node_data.update_child(*child_id, rc_new_child),
                    )
                }
                (Node::Node2(node_data), InsertUpstreamData::Update3Child(new_child)) => {
                    let rc_new_child = Rc::new(Node::Node3(new_child));
                    InsertUpstreamData::Update2Child(
                        node_data.update_child(*child_id, rc_new_child),
                    )
                }
                (Node::Node3(node_data), InsertUpstreamData::Update2Child(new_child)) => {
                    let rc_new_child = Rc::new(Node::Node2(new_child));
                    InsertUpstreamData::Update3Child(
                        node_data.update_child(*child_id, rc_new_child),
                    )
                }
                (Node::Node3(node_data), InsertUpstreamData::Update3Child(new_child)) => {
                    let rc_new_child = Rc::new(Node::Node3(new_child));
                    InsertUpstreamData::Update3Child(
                        node_data.update_child(*child_id, rc_new_child),
                    )
                }

                (Node::Node2(node_data), InsertUpstreamData::Split(middle, left, right)) => {
                    let new_node_data = node_data.merge(*child_id, middle, left, right);
                    InsertUpstreamData::Update3Child(new_node_data)
                }
                (Node::Node3(node_data), InsertUpstreamData::Split(middle, left, right)) => {
                    let big_node_data = node_data.merge(*child_id, middle, left, right);
                    let (middle, left, right) = big_node_data.split();
                    InsertUpstreamData::Split(middle, left, right)
                }
                (Node::Nil(_), _) => unreachable!(),
            }
        }
    }
}
