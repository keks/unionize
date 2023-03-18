use crate::{LiftingMonoid, Node, SumMonoid};

// sise impl

pub(crate) trait SiseMonoid: LiftingMonoid {
    fn item_to_sise(item: &Self::Item) -> sise::TreeNode;
}

impl SiseMonoid for SumMonoid {
    fn item_to_sise(item: &Self::Item) -> sise::TreeNode {
        sise::TreeNode::Atom(format!("{item}"))
    }
}

impl<M: SiseMonoid> Into<sise::TreeNode> for Node<M> {
    fn into(self) -> sise::TreeNode {
        match self {
            Node::Node2(node_data) => sise::TreeNode::List(vec![
                    M::item_to_sise(&node_data.items[0]),
                    node_data.children[0].as_ref().clone().into(),
                    node_data.last_child.as_ref().clone().into(),
                ]),
            Node::Node3(node_data) => sise::TreeNode::List(vec![
                M::item_to_sise(&node_data.items[0]),
                M::item_to_sise(&node_data.items[1]),
                node_data.children[0].as_ref().clone().into(),
                node_data.children[1].as_ref().clone().into(),
                node_data.last_child.as_ref().clone().into(),
            ]),
            Node::Nil(_) => sise::TreeNode::Atom("nil".to_string()),
            
        }
    }
}

// sexp impl

pub(crate) trait SexpMonoid: LiftingMonoid {
    fn item_to_sexp(item: &Self::Item) -> sexp::Sexp;
}


impl SexpMonoid for SumMonoid {
    fn item_to_sexp(item: &Self::Item) -> sexp::Sexp {
        sexp::Sexp::Atom(sexp::Atom::I(*item as i64))
    }
}

impl<M: SexpMonoid> Into<sexp::Sexp> for Node<M> {
    fn into(self) -> sexp::Sexp {
        match self {
            Node::Node2(node_data) => {
                let v: Vec<sexp::Sexp> = vec![
                    M::item_to_sexp(&node_data.items[0]),
                    node_data.children[0].as_ref().clone().into(),
                    node_data.last_child.as_ref().clone().into(),
                ];

                sexp::Sexp::List(v)
            }
            
            Node::Node3(node_data) =>{
                let v: Vec<sexp::Sexp> = vec![
                    M::item_to_sexp(&node_data.items[0]),
                    M::item_to_sexp(&node_data.items[1]),
                    node_data.children[0].as_ref().clone().into(),
                    node_data.children[1].as_ref().clone().into(),
                    node_data.last_child.as_ref().clone().into(),
                ];

                sexp::Sexp::List(v)

            },
            Node::Nil(_) => sexp::Sexp::Atom(sexp::Atom::S("nil".to_string())),
        }
    }
}

impl Into<sexp::Atom> for SumMonoid {
    fn into(self) -> sexp::Atom {
        sexp::Atom::I(self.0 as i64)
    }
}

// lexpr impl

impl<M: LiftingMonoid> Into<lexpr::Value> for Node<M> where M::Item: Into<lexpr::Number>{
    fn into(self) -> lexpr::Value {
        match self {
            Node::Node2(node_data) => {
                let v: Vec<lexpr::Value> = vec![
                    node_data.items[0].clone().into().into(),
                    node_data.children[0].as_ref().clone().into(),
                    node_data.last_child.as_ref().clone().into(),
                ];

                lexpr::Value::vector(v.into_iter())
            }
            
            Node::Node3(node_data) =>{
                let v: Vec<lexpr::Value> = vec![
                    node_data.items[0].clone().into().into(),
                    node_data.items[1].clone().into().into(),
                    node_data.children[0].as_ref().clone().into(),
                    node_data.children[1].as_ref().clone().into(),
                    node_data.last_child.as_ref().clone().into(),
                ];

                lexpr::Value::vector(v.into_iter())

            },
            Node::Nil(_) => lexpr::Value::Nil,
        }
    }
}

impl Into<lexpr::Value> for SumMonoid {
    fn into(self) -> lexpr::Value {
        lexpr::Number::from(self.0 as u64).into()
    }
}



#[cfg(test)]
mod test {
    use crate::{SumMonoid, Node};

    #[test]
    fn wtf_lexpr() {
        let mut root = Node::Nil(SumMonoid(0));
        root = root.insert(1);
        root = root.insert(2);
        root = root.insert(4);

        let root_sexp: sexp::Sexp = root.into();
        println!("{}", root_sexp);
    }
}