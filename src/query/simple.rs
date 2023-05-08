use crate::{monoid::Monoid, ranged_node::RangedNode};

use super::Accumulator;

#[derive(Debug, Clone)]
pub struct SimpleAccumulator<M: Monoid>(M);

impl<M: Monoid> SimpleAccumulator<M> {
    pub fn new() -> Self {
        SimpleAccumulator(M::neutral())
    }

    pub fn result(&self) -> &M {
        &self.0
    }

    pub fn into_result(self) -> M {
        self.0
    }
}

impl<M: Monoid> Accumulator<M> for SimpleAccumulator<M> {
    fn add_node(&mut self, node: &RangedNode<M>) {
        for (child, item) in node.children() {
            self.0 = self.0.combine(child.node().monoid());
            self.0 = self.0.combine(&M::lift(&item));
        }

        self.0 = self.0.combine(node.node().last_child().monoid());
    }

    fn add_item(&mut self, item: &M::Item) {
        self.0 = self.0.combine(&M::lift(&item));
    }
}
