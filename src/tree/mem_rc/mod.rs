use crate::monoid::Monoid;
use std::rc::Rc;

mod fmt;
mod insert;
mod node_impl;

#[derive(Clone)]
pub enum Node<M: Monoid> {
    Node2(NodeData<M, 1>),
    Node3(NodeData<M, 2>),
    Nil(M),
}

impl<M: Monoid> Node<M> {
    pub fn nil() -> Self {
        Self::Nil(M::neutral())
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, Self::Nil(..))
    }

    pub fn monoid(&self) -> &M {
        match self {
            Node::Node2(node_data) => &node_data.total,
            Node::Node3(node_data) => &node_data.total,
            Node::Nil(m) => m,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NodeData<M: Monoid, const N: usize> {
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

impl<M: Monoid, const N: usize> NodeData<M, N> {
    pub const N: usize = N;

    pub fn new(items: [M::Item; N], children: [Rc<Node<M>>; N], last_child: Rc<Node<M>>) -> Self {
        let total = Self::compute_total(&items, &children, &last_child);

        NodeData {
            items,
            children,
            last_child,
            total,
        }
    }

    pub fn items(&self) -> &[M::Item; N] {
        &self.items
    }

    pub fn children(&self) -> (&[Rc<Node<M>>; N], &Rc<Node<M>>) {
        (&self.children, &self.last_child)
    }

    pub fn last_child(&self) -> &Rc<Node<M>> {
        &self.last_child
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

    pub fn grow<const N_PLUS_1: usize>(
        &self,
        item: M::Item,
        child: Rc<Node<M>>,
    ) -> NodeData<M, N_PLUS_1> {
        assert_eq!(N + 1, N_PLUS_1);

        let found = self.items.iter().position(|x| &item < x);
        let (before_range, after_range) = match found {
            Some(pos) => (0..pos, pos..N),
            None => (0..N, N..N),
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

    pub fn next_item(&self, item: &M::Item) -> Option<M::Item> {
        self.items.iter().find(|cur_item| item < cur_item).cloned()
    }

    pub fn get_item(&self, idx: usize) -> Option<M::Item> {
        self.items.get(idx).cloned()
    }

    pub(crate) fn min_item(&self) -> &M::Item {
        match &self.children[0].as_ref() {
            Node::Node2(node_data) => node_data.min_item(),
            Node::Node3(node_data) => node_data.min_item(),
            Node::Nil(_) => &self.items[0],
        }
    }

    pub(crate) fn max_item(&self) -> &M::Item {
        match &self.last_child.as_ref() {
            Node::Node2(node_data) => node_data.max_item(),
            Node::Node3(node_data) => node_data.max_item(),
            Node::Nil(_) => &self.items[N - 1],
        }
    }

    pub(crate) fn bounds(&self) -> (&M::Item, &M::Item) {
        (self.min_item(), self.max_item())
    }
}

impl<M: Monoid> NodeData<M, 1> {
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

impl<M: Monoid> NodeData<M, 2> {
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

impl<M: Monoid> NodeData<M, 3> {
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

impl<M: Monoid> Node<M> {
    impl_NodeData_on_Node!(find_child . item: &M::Item => (ChildId, Rc<Node<M>>));
    impl_NodeData_on_Node!(is_leaf . => bool);
}
