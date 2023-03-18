use std::fmt::Debug;
use std::rc::Rc;

mod sexpr;

mod range;
mod proto;
mod cursor;
mod iter;


macro_rules! impl_NodeData_on_Node {
  ($func_name:ident . $($arg_name:ident: $arg_type:ty),*) => {
    fn $func_name(&self, $($arg_name: $arg_type),+) {
      match self {
          Node::Nil(_) => panic!("can't call {} on nil node", stringify!($func_name)),
          Node::Node2(node_data) => node_data.$func_name($($arg_name),*),
          Node::Node3(node_data) => node_data.$func_name($($arg_name),*),
      }
    }
  };
  ($func_name:ident . $($arg_name:ident: $arg_type:ty),* => $ret_type:ty) => {
    fn $func_name(&self, $($arg_name: $arg_type),*) -> $ret_type{
      match self {
          Node::Nil(_) => panic!("can't call {} on nil node", stringify!($func_name)),
          Node::Node2(node_data) => node_data.$func_name($($arg_name),*),
          Node::Node3(node_data) => node_data.$func_name($($arg_name),*),
      }
    }
  };
}


#[derive(Debug, Clone)]
struct SumMonoid(u64);

impl LiftingMonoid for SumMonoid {
    type Item = u64;

    fn neutral() -> Self {
        SumMonoid(0)
    }

    fn lift(item: &Self::Item) -> Self {
        SumMonoid(*item)
    }

    fn combine(&self, other: &Self) -> Self {
        let (SumMonoid(lhs), SumMonoid(rhs)) = (self, other);
        SumMonoid(*lhs + *rhs)
    }
}

trait LiftingMonoid: Clone + Debug {
    type Item: Clone + Debug + Ord;

    fn neutral() -> Self;
    fn lift(item: &Self::Item) -> Self;
    fn combine(&self, other: &Self) -> Self;
}

#[derive(Clone, Debug)]
struct NodeData<M: LiftingMonoid, const N: usize> {
    items: [M::Item; N],
    children: [Rc<Node<M>>; N],
    last_child: Rc<Node<M>>,
    total: M,
}

#[derive(Clone, Copy, Debug)]
enum ChildId {
    Normal(usize),
    Last,
}

trait NodeDataFuns {
    fn is_leaf(&self) -> bool;
}

impl<M: LiftingMonoid, const N: usize> NodeData<M, N> {
    fn new(items: [M::Item; N], children: [Rc<Node<M>>; N], last_child: Rc<Node<M>>) -> Self {
        let total = Self::compute_total(&items, &children, &last_child);

        NodeData {
            items: items,
            children: children,
            last_child,
            total,
        }
    }

    fn compute_total(items: &[M::Item; N], children: &[Rc<Node<M>>; N], last_child: &Node<M>) -> M {
        let mut total = M::neutral();
        for i in 0..N {
            total = total.combine(children[i].as_ref().monoid());
            total = total.combine(&M::lift(&items[i]));
        }
        total = total.combine(last_child.monoid());

        total
    }

    fn is_leaf(&self) -> bool {
        matches!(self.last_child.as_ref(), Node::Nil(_))
    }

    fn carries_item(&self, item: &M::Item) -> bool {
      self.items.contains(item)
    }

    fn find_child(&self, item: &M::Item) -> (ChildId, Rc<Node<M>>) {
        let found = self.items.iter().position(|x| item < x);
        match found {
            Some(pos) => (ChildId::Normal(pos), Rc::clone(&self.children[pos])),
            None => (ChildId::Last, Rc::clone(&self.last_child)),
        }
    }

    fn update_child(&self, child_id: ChildId, new_child: Rc<Node<M>>) -> NodeData<M, N> {
        let mut new_node = self.clone();

        match child_id {
            ChildId::Normal(child_offs) => {
                new_node.children[child_offs] = new_child;
                new_node.total =
                    Self::compute_total(&new_node.items, &new_node.children, &new_node.last_child);
            }
            ChildId::Last => {
                new_node.last_child = new_child;
                new_node.total =
                    Self::compute_total(&new_node.items, &new_node.children, &new_node.last_child);
            }
        }

        new_node
    }

    fn grow<const N_PLUS_1: usize>(
        &self,
        item: M::Item,
        child: Rc<Node<M>>,
    ) -> NodeData<M, N_PLUS_1> {
        assert_eq!(N + 1, N_PLUS_1);

        let found = self.items.iter().position(|x| &item < x);
        let (pos, before_range, after_range) = match found {
            Some(pos) => (pos, 0..pos, pos..N),
            None => (N, 0..N, N..N),
        };

        // we may need to add the last_child here somehow? ->

        let mut items = Vec::with_capacity(N_PLUS_1);
        let mut children = Vec::with_capacity(N_PLUS_1);

        items.extend(self.items[before_range.clone()].iter().cloned());
        children.extend(self.children[before_range].iter().cloned());

        items.push(item);
        children.push(child);

        items.extend(self.items[after_range.clone()].iter().cloned());
        children.extend(self.children[after_range].iter().cloned());

        assert_eq!(items.len(), N_PLUS_1);
        assert_eq!(children.len(), N_PLUS_1);

        let items: [M::Item; N_PLUS_1] = items.try_into().unwrap();
        let children: [Rc<Node<M>>; N_PLUS_1] = children.try_into().unwrap();

        let last_child = self.last_child.clone();
        let total = NodeData::<M, N_PLUS_1>::compute_total(&items, &children, &last_child);

        NodeData {
            items,
            children,
            total,
            last_child,
        }
    }


    fn child_by_child_id(&self, id: ChildId) -> Option<Rc<Node<M>>> {
      match id {
        ChildId::Normal(idx) if idx < N => Some(Rc::clone(&self.children[idx])),
        ChildId::Last => Some(Rc::clone(&self.last_child)),
        _ => None,
      }
    }

    fn next_item(&self, item: &M::Item) -> Option<M::Item> {
      self.items.iter().find(|cur_item| item < cur_item).cloned()
    }

    fn find_item<F: FnMut(&&M::Item) -> bool >(&self, mut f: F) -> Option<M::Item> {
      self.items.iter().find(f).cloned()
    }

    fn item_position<F: FnMut(&M::Item) -> bool >(&self, mut f: F) -> Option<usize> {
      self.items.iter().position(f)
    }

    fn get_item(&self, idx: usize) -> Option<M::Item> {
      self.items.get(idx).cloned()
    }

    fn n(&self) -> usize {
        N
    }
}

impl<M: LiftingMonoid> NodeData<M, 3> {
    fn split(&self) -> (M::Item, NodeData<M, 1>, NodeData<M, 1>) {
        let left_items: &[M::Item; 1] = self.items[0..1].try_into().unwrap();
        let right_items: &[M::Item; 1] = self.items[2..3].try_into().unwrap();

        let left_children: &[Rc<Node<M>>; 1] = self.children[0..1].try_into().unwrap();
        let right_children: &[Rc<Node<M>>; 1] = self.children[2..3].try_into().unwrap();

        let left = NodeData::new(
            left_items.clone(),
            left_children.clone(),
            Rc::clone(&self.children[1]),
        );
        let right = NodeData::new(
            right_items.clone(),
            right_children.clone(),
            Rc::clone(&self.last_child),
        );
        let middle = self.items[1].clone();

        (middle, left, right)
    }
}

impl<M: LiftingMonoid> NodeData<M, 1> {
    fn merge(
        &self,
        child_id: ChildId,
        middle: M::Item,
        left: NodeData<M, 1>,
        right: NodeData<M, 1>,
    ) -> NodeData<M, 2> {
        let rc_left = Rc::new(Node::Node2(left));
        let rc_right = Rc::new(Node::Node2(right));

        match child_id {
            ChildId::Normal(offs) if offs == 0 => {
                let items = [middle, self.items[0].clone()];
                let children = [rc_left, rc_right];

                NodeData::new(items, children, self.last_child.clone())
            }
            ChildId::Last => {
                let items = [self.items[0].clone(), middle];
                let children = [self.children[0].clone(), rc_left];

                NodeData::new(items, children, rc_right)
            }
            ChildId::Normal(offs) => unreachable!("{offs}"),
        }
    }
}

impl<M: LiftingMonoid> NodeData<M, 2> {
    fn merge(
        &self,
        child_id: ChildId,
        middle: M::Item,
        left: NodeData<M, 1>,
        right: NodeData<M, 1>,
    ) -> NodeData<M, 3> {
        let rc_left = Rc::new(Node::Node2(left));
        let rc_right = Rc::new(Node::Node2(right));

        match child_id {
            ChildId::Normal(offs) if offs == 0 => {
                let items = [middle, self.items[0].clone(), self.items[1].clone()];
                let children = [rc_left, rc_right, self.children[1].clone()];

                NodeData::new(items, children, self.last_child.clone())
            }
            ChildId::Normal(offs) if offs == 1 => {
                let items = [self.items[0].clone(), middle, self.items[1].clone()];
                let children = [self.children[0].clone(), rc_left, rc_right];

                NodeData::new(items, children, self.last_child.clone())
            }
            ChildId::Last => {
                let items = [self.items[0].clone(), self.items[1].clone(), middle];
                let children = [self.children[0].clone(), self.children[1].clone(), rc_left];

                NodeData::new(items, children, rc_right)
            }
            ChildId::Normal(offs) => unreachable!("{offs}"),
        }
    }
}

impl<M: LiftingMonoid, const N: usize> NodeDataFuns for NodeData<M, N> {
    fn is_leaf(&self) -> bool {
        matches!(self.children[0].as_ref(), Node::Nil(_))
    }
}

#[derive(Clone, Debug)]
enum Node<M: LiftingMonoid> {
    Node2(NodeData<M, 1>),
    Node3(NodeData<M, 2>),
    Nil(M),
}

impl<M: LiftingMonoid> Node<M> {
    fn monoid(&self) -> &M {
        match self {
            Node::Node2(node_data) => &node_data.total,
            Node::Node3(node_data) => &node_data.total,
            Node::Nil(m) => m,
        }
    }

    impl_NodeData_on_Node!(child_by_child_id . id: ChildId => Option<Rc<Node<M>>>);
    impl_NodeData_on_Node!(carries_item . item: &M::Item => bool);
    impl_NodeData_on_Node!(next_item . item: &M::Item => Option<M::Item>);
    impl_NodeData_on_Node!(get_item . idx: usize => Option<M::Item>);
    impl_NodeData_on_Node!(find_child . item: &M::Item => (ChildId, Rc<Node<M>>));
    impl_NodeData_on_Node!(is_leaf . => bool);
    impl_NodeData_on_Node!(n . => usize);

    fn find_item<F: FnMut(&&M::Item) -> bool >(&self, mut f: F) -> Option<M::Item> {
      match self {
        Node::Node2(node_data) => node_data.find_item(f),
        Node::Node3(node_data) => node_data.find_item(f),
        Node::Nil(_) => None,
      }
    }


    fn item_position<F: FnMut(&M::Item) -> bool >(&self, mut f: F) -> Option<usize> {
      match self {
        Node::Node2(node_data) => node_data.item_position(f),
        Node::Node3(node_data) => node_data.item_position(f),
        Node::Nil(_) => None,
      }
    }

}

impl<M: sexpr::SiseMonoid> std::fmt::Display for Node<M> {
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

enum InsertUpstreamData<M: LiftingMonoid> {
    Update2Child(NodeData<M, 1>),
    Update3Child(NodeData<M, 2>),
    Split(M::Item, NodeData<M, 1>, NodeData<M, 1>),
}

impl<M: LiftingMonoid> Node<M> {
    fn insert(&self, item: M::Item) -> Node<M> {
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

// impl<M: LiftingMonoid> std::fmt::Display for Node<M> where M::Item: std::fmt::Display {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Node::Node2(node_data) => write!(f, "([{}] L:{} R:{})", node_data.items[0], node_data.children[0], node_data.last_child),
//             Node::Node3(node_data) => write!(f, "([{} {}] L:{} M:{} R:{})", node_data.items[0], node_data.items[1], node_data.children[0], node_data.children[1], node_data.last_child),
//             Node::Nil(_) => write!(f, "nil"),
//         }
//     }
// }

#[cfg(test)]
mod test {
    use crate::{LiftingMonoid, Node, SumMonoid};

    #[test]
    fn example_node2() {
        let root = Node::<SumMonoid>::Nil(SumMonoid::lift(&0));

        println!("{:#?}", root.insert(30).insert(60).insert(50));
    }

    #[test]
    fn example_node3() {
        let mut root = Node::<SumMonoid>::Nil(SumMonoid::lift(&0));

        root = root.insert(1);
        root = root.insert(2);
        root = root.insert(4);
        root = root.insert(16);
        root = root.insert(8);

        println!("{root}");
    }
}
