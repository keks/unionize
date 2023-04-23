use crate::{range::Range, tree::ChildId, LiftingMonoid, Node};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub(crate) struct Cursor<M: LiftingMonoid> {
    root: Rc<Node<M>>,
    path: Vec<(ChildId, Range<M::Item>, Rc<Node<M>>)>,
}

impl<M: LiftingMonoid> Cursor<M> {
    pub(crate) fn new(root: Rc<Node<M>>) -> Self {
        let path = vec![];
        Cursor { root, path }
    }

    pub(crate) fn current(&self) -> &Node<M> {
        match self.path.last() {
            Some((_child_id, _range, node)) => node,
            None => &self.root,
        }
    }

    pub fn current_rc(&self) -> Rc<Node<M>> {
        match self.path.last() {
            Some((_child_id, _range, node)) => node,
            None => &self.root,
        }
        .clone()
    }

    pub(crate) fn current_range(&self) -> Range<M::Item> {
        match self.path.last() {
            Some((_, range, _)) => range.clone(),
            None => Range::Full,
        }
    }

    pub(crate) fn push(&mut self, id: ChildId) {
        let current = self.current();
        let current_range = self.current_range();

        let next = current
            .child_by_child_id(id)
            .expect("cursor can't push: current node doesn't have a child with given child id");

        let next_range = match (current_range, id) {
            (Range::Full, ChildId::Normal(0)) => Range::UpTo(current.get_item(0).unwrap()),

            (Range::StartingFrom(start), ChildId::Normal(0))
            | (Range::Between(start, _), ChildId::Normal(0)) => {
                let end = current.get_item(0).unwrap();
                Range::Between(start, end)
            }

            (_, ChildId::Normal(i)) => {
                let start = current.get_item(i - 1).unwrap();
                let end = current.get_item(i).unwrap();
                Range::Between(start, end)
            }

            (Range::StartingFrom(_), ChildId::Last) | (Range::Full, ChildId::Last) => match current
            {
                Node::Node2(node_data) => {
                    Range::StartingFrom(node_data.get_item(0).unwrap().clone())
                }
                Node::Node3(node_data) => {
                    Range::StartingFrom(node_data.get_item(1).unwrap().clone())
                }
                Node::Nil(_) => panic!(),
            },
            (Range::UpTo(end), ChildId::Last) | (Range::Between(_, end), ChildId::Last) => {
                match current {
                    Node::Node2(node_data) => {
                        Range::Between(node_data.get_item(0).unwrap().clone(), end)
                    }
                    Node::Node3(node_data) => {
                        Range::Between(node_data.get_item(1).unwrap().clone(), end)
                    }
                    Node::Nil(_) => panic!(),
                }
            }
        };

        self.path.push((id, next_range, next));
    }

    pub(crate) fn pop(&mut self) -> Option<(ChildId, Rc<Node<M>>)> {
        let Cursor { root: _root, path } = self;
        path.pop().map(|(child_id, _, node)| (child_id, node))
    }

    pub(crate) fn find(&mut self, item: &M::Item) {
        loop {
            let current = self.current();

            if current.carries_item(item) || current.is_leaf() {
                break;
            }

            let (child_id, _) = current.find_child(item);
            self.push(child_id);
        }
    }

    pub(crate) fn find_first(&mut self) {
        loop {
            let current = self.current();

            if current.is_leaf() {
                break;
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
