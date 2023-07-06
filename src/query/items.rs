use crate::{monoid::Monoid, Node};

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
    fn add_xnode<'a, N: Node<'a, M>>(&mut self, node: &'a N)
    where
        M: 'a,
    {
        if node.is_nil() {
            return;
        }

        for (child, item) in node.children().unwrap() {
            self.add_xnode(child);
            self.items.push(item.clone());
        }

        self.add_xnode(node.last_child().unwrap())
    }

    fn add_item(&mut self, item: &<M as Monoid>::Item) {
        self.items.push(item.clone())
    }
}
