use crate::{monoid::Monoid, ranged_node::RangedNode};

use super::Accumulator;

#[derive(Debug, Clone)]
pub struct ItemsAccumulator<M: Monoid> {
    items: Vec<M::Item>,
}

impl<M: Monoid> ItemsAccumulator<M> {
    pub fn new() -> Self {
        Self { items: vec![] }
    }

    pub fn results(&self) -> &[M::Item] {
        &self.items
    }

    pub fn into_results(self) -> Vec<M::Item> {
        self.items
    }
}

impl<M> Accumulator<M> for ItemsAccumulator<M>
where
    M: Monoid,
{
    fn add_node(&mut self, node: &RangedNode<M>) {
        let is_leaf = node.node().is_leaf();
        for (child, item) in node.children() {
            if !is_leaf {
                self.add_node(&child);
            }
            self.items.push(item);
        }

        if !is_leaf {
            self.add_node(&node.last_child())
        }
    }

    fn add_item(&mut self, item: &<M as Monoid>::Item) {
        self.items.push(item.clone())
    }
}
