use crate::{monoid::Monoid, tree::ChildId, Node};

impl<M: Monoid> std::fmt::Debug for Node<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tree = self.debug_tree();
        let style = sise::SerializerStyle {
            line_break: "\n",
            indentation: "  ",
        };

        let mut out = String::new();
        let mut serializer = sise::Serializer::new(style, &mut out);
        sise::serialize_tree(&mut serializer, &tree, 48);

        write!(f, "{out}")
    }
}

impl<M: Monoid> std::fmt::Display for Node<M>
where
    M::Item: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tree = self.display_tree();
        let style = sise::SerializerStyle {
            line_break: "\n",
            indentation: "  ",
        };

        let mut out = String::new();
        let mut serializer = sise::Serializer::new(style, &mut out);
        sise::serialize_tree(&mut serializer, &tree, 48);

        write!(f, "{out}")
    }
}

impl<M: Monoid> Node<M>
where
    M::Item: std::fmt::Display,
{
    fn display_tree(&self) -> sise::TreeNode {
        match self {
            Node::Node2(node_data) => sise::TreeNode::List(vec![
                sise::TreeNode::Atom(format!("{:}", node_data.get_item(0).unwrap())),
                node_data
                    .child_by_child_id(ChildId::Normal(0))
                    .unwrap()
                    .as_ref()
                    .debug_tree(),
                node_data
                    .child_by_child_id(ChildId::Last)
                    .unwrap()
                    .as_ref()
                    .debug_tree(),
            ]),
            Node::Node3(node_data) => sise::TreeNode::List(vec![
                sise::TreeNode::Atom(format!("{:}", node_data.get_item(0).unwrap())),
                sise::TreeNode::Atom(format!("{:}", node_data.get_item(1).unwrap())),
                node_data
                    .child_by_child_id(ChildId::Normal(0))
                    .unwrap()
                    .as_ref()
                    .debug_tree(),
                node_data
                    .child_by_child_id(ChildId::Normal(1))
                    .unwrap()
                    .as_ref()
                    .debug_tree(),
                node_data
                    .child_by_child_id(ChildId::Last)
                    .unwrap()
                    .as_ref()
                    .debug_tree(),
            ]),
            Node::Nil(_) => sise::TreeNode::Atom("nil".to_string()),
        }
    }
}

impl<M: Monoid> Node<M> {
    fn debug_tree(&self) -> sise::TreeNode {
        match self {
            Node::Node2(node_data) => sise::TreeNode::List(vec![
                sise::TreeNode::Atom(format!("{:?}", node_data.get_item(0).unwrap())),
                node_data
                    .child_by_child_id(ChildId::Normal(0))
                    .unwrap()
                    .as_ref()
                    .debug_tree(),
                node_data
                    .child_by_child_id(ChildId::Last)
                    .unwrap()
                    .as_ref()
                    .debug_tree(),
            ]),
            Node::Node3(node_data) => sise::TreeNode::List(vec![
                sise::TreeNode::Atom(format!("{:?}", node_data.get_item(0).unwrap())),
                sise::TreeNode::Atom(format!("{:?}", node_data.get_item(1).unwrap())),
                node_data
                    .child_by_child_id(ChildId::Normal(0))
                    .unwrap()
                    .as_ref()
                    .debug_tree(),
                node_data
                    .child_by_child_id(ChildId::Normal(1))
                    .unwrap()
                    .as_ref()
                    .debug_tree(),
                node_data
                    .child_by_child_id(ChildId::Last)
                    .unwrap()
                    .as_ref()
                    .debug_tree(),
            ]),
            Node::Nil(_) => sise::TreeNode::Atom("nil".to_string()),
        }
    }
}
