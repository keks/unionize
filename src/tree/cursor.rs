use std::rc::Rc;

use crate::{
    range::Rangable,
    ranged_node::{RangedNode, RangedRcNode},
    tree::ChildId,
    LiftingMonoid,
};

#[derive(Clone, Debug)]
struct InnerCursor<'a, M>
where
    M: LiftingMonoid,
    M::Item: Rangable,
{
    node: RangedRcNode<'a, M>,
    parent: Option<(Cursor<'a, M>, ChildId)>,
}

#[derive(Clone, Debug)]
pub struct Cursor<'a, M>(Rc<InnerCursor<'a, M>>)
where
    M: LiftingMonoid,
    M::Item: Rangable;

impl<'a, M> Cursor<'a, M>
where
    M: LiftingMonoid,
    M::Item: Rangable,
{
    pub fn new(node: RangedRcNode<M>) -> Cursor<M> {
        Cursor(Rc::new(InnerCursor { node, parent: None }))
    }

    fn inner(&self) -> &InnerCursor<'a, M> {
        &self.0
    }

    fn inner_rc(&self) -> &Rc<InnerCursor<'a, M>> {
        &self.0
    }

    pub fn push(self, child_id: ChildId) -> Option<Self> {
        let child = self.inner().node.ranged_node().get_child(child_id);
        child.map(|child| {
            Cursor(Rc::new(InnerCursor {
                node: child,
                parent: Some((self, child_id)),
            }))
        })
    }

    pub fn pop(self) -> Option<(Cursor<'a, M>, ChildId)> {
        self.inner().parent.clone()
    }

    pub fn current(&self) -> RangedNode<M> {
        self.inner().node.ranged_node().clone()
    }

    pub fn current_rc(&self) -> RangedRcNode<M> {
        self.inner().node.clone()
    }

    pub fn find<'b>(self, item: &M::Item) -> Cursor<'a, M>
    where
        'b: 'a,
    {
        let mut cur = self;

        loop {
            let (current_carries_item, current_is_leaf, child_id) = {
                let current = cur.current();
                let current_node = current.node();
                let (child_id, _) = current_node.find_child(item);
                (
                    current_node.carries_item(item),
                    current_node.is_leaf(),
                    child_id,
                )
            };

            if current_carries_item || current_is_leaf {
                return cur;
            }

            cur = cur.push(child_id).unwrap().find(item);
        }
    }

    pub fn find_first(self) -> Cursor<'a, M> {
        let mut cur_cursor: Cursor<M> = self.clone();
        loop {
            if cur_cursor.current().node().is_leaf() {
                return cur_cursor;
            }

            cur_cursor = cur_cursor
                .push(ChildId::Normal(0))
                .expect("there should be a child since this is not a leaf");
        }
    }

    pub fn goto_next_child(self, child_id: ChildId) -> Option<Self> {
        match child_id {
            ChildId::Last => None,
            ChildId::Normal(idx) => {
                let next_id = ChildId::Normal(idx + 1);
                if self.current().get_child(next_id).is_some() {
                    self.push(next_id)
                } else {
                    self.push(ChildId::Last)
                }
            }
        }
    }
}
