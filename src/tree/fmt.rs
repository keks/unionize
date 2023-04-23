use crate::{tree::ChildId, Node, monoid::{FormattingMonoid}};

impl<M: FormattingMonoid> Into<sise::TreeNode> for Node<M> {
    fn into(self) -> sise::TreeNode {
        match self {
            Node::Node2(node_data) => sise::TreeNode::List(vec![
                sise::TreeNode::Atom(
                    M::item_to_string(
                        &node_data.get_item(0).unwrap())),
                node_data
                    .child_by_child_id(ChildId::Normal(0))
                    .unwrap()
                    .as_ref()
                    .clone()
                    .into(),
                node_data
                    .child_by_child_id(ChildId::Last)
                    .unwrap()
                    .as_ref()
                    .clone()
                    .into(),
            ]),
            Node::Node3(node_data) => sise::TreeNode::List(vec![
                sise::TreeNode::Atom(
                    M::item_to_string(
                        &node_data.get_item(0).unwrap())),
                sise::TreeNode::Atom(
                    M::item_to_string(
                        &node_data.get_item(1).unwrap())),
                node_data
                    .child_by_child_id(ChildId::Normal(0))
                    .unwrap()
                    .as_ref()
                    .clone()
                    .into(),
                node_data
                    .child_by_child_id(ChildId::Normal(1))
                    .unwrap()
                    .as_ref()
                    .clone()
                    .into(),
                node_data
                    .child_by_child_id(ChildId::Last)
                    .unwrap()
                    .as_ref()
                    .clone()
                    .into(),
            ]),
            Node::Nil(_) => sise::TreeNode::Atom("nil".to_string()),
        }
    }
}

impl<M: FormattingMonoid> std::fmt::Display for Node<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let style = sise::SerializerStyle {
            line_break: "\n",
            indentation: "  ",
        };

        let mut out = String::new();
        let mut serializer = sise::Serializer::new(style, &mut out);
        sise::serialize_tree(&mut serializer, &self.clone().into(), 48);

        write!(f, "{out}")
    }
}