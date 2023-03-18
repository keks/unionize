use std::rc::Rc;

use crate::{LiftingMonoid, Node, ChildId, range::Range};

#[derive(Clone, Debug)]
enum PathElemType<M: LiftingMonoid> {
    Root,
    Child(ChildId, Range<M::Item>),
}

#[derive(Clone, Debug)]
pub(crate) struct Cursor<M: LiftingMonoid> (Vec<(PathElemType<M>, Rc<Node<M>>)>);

impl<M: LiftingMonoid> Cursor<M> {
    pub(crate) fn new(node: Rc<Node<M>>) -> Self {
        Cursor(vec![(PathElemType::Root, node)])
    }

    pub(crate) fn current(&self) -> &Node<M> {
        let Cursor(path) = self;

        assert!(path.len() > 0);

        let (_, node) = &path.last().unwrap();

        node
    }

    pub(crate) fn current_range(&self) -> Range<M::Item> {
        let Cursor(path) = self;

        assert!(path.len() > 0);

        let (path_elem, _) = &path.last().unwrap();
        match path_elem {
            PathElemType::Root => Range::Full,
            PathElemType::Child(_, range) => range.clone(),
        }
    }

    pub(crate) fn push(&mut self, id: ChildId) {
        let current = self.current();
        let current_range = self.current_range();

        let next = current.child_by_child_id(id).expect("cursor can't push: current node doesn't have a child with given child id");

        let next_range = match (current_range, id) {
            (Range::Full, ChildId::Normal(0)) => Range::UpTo(current.get_item(0).unwrap()),

            (Range::StartingFrom(start), ChildId::Normal(0)) |
            (Range::Between(start, _), ChildId::Normal(0)) => {
                let end = current.get_item(0).unwrap();
                Range::Between(start, end)
            },

            (_, ChildId::Normal(i)) => {
                let start = current.get_item(i-1).unwrap();
                let end = current.get_item(i).unwrap();
                Range::Between(start, end)
            },

            (Range::StartingFrom(_), ChildId::Last)  |
            (Range::Full, ChildId::Last) => match current {
                Node::Node2(node_data) => Range::StartingFrom(node_data.items[0].clone()),
                Node::Node3(node_data) => Range::StartingFrom(node_data.items[1].clone()),
                Node::Nil(_) => panic!(),
            },
            (Range::UpTo(end), ChildId::Last) |
            (Range::Between(_, end), ChildId::Last) => match current {
                Node::Node2(node_data) => Range::Between(node_data.items[0].clone(), end),
                Node::Node3(node_data) => Range::Between(node_data.items[1].clone(), end),
                Node::Nil(_) => panic!(),
            },
        };


        self.0.push((PathElemType::Child(id, next_range), next));
    }

    pub(crate) fn pop(&mut self) -> Option<(ChildId, Rc<Node<M>>)> {
        let Cursor(path) = self;
        assert!(path.len() > 0);

        if self.0.len() == 1 {
            return None
        }

        let (elem, node) = self.0.pop().unwrap();

        if let PathElemType::Child(child_id, _) = elem {
            Some((child_id, node))
        } else {
            unreachable!()
        }
    }

    pub(crate) fn find(&mut self, item: &M::Item) {
        loop {
            let current = self.current();

            if current.carries_item(item) || current.is_leaf() {
                break
            }

            let (child_id, _) = current.find_child(item);
            self.push(child_id);
        }
    }

    pub(crate) fn find_first(&mut self) {
        loop {
            let current = self.current();

            if current.is_leaf() {
                break
            }

            let child_id = ChildId::Normal(0);
            self.push(child_id);
        }
    }

    pub(crate) fn goto_next_child(&mut self, id: &ChildId) {
        match id {
            ChildId::Last => panic!(),
            ChildId::Normal(i) => {
                let next_id = ChildId::Normal(*i + 1);
                if self.current().child_by_child_id(next_id).is_some() {
                    self.push(next_id);
                } else {
                    self.push(ChildId::Last);
                }
            }
        }
    }
}