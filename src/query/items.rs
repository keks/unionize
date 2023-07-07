use crate::{monoid::Monoid, Node, NonNilNode};

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
    fn add_node<'a, N: Node<M>>(&mut self, node: &'a N) {
        let non_nil_node = if let Some(non_nil_node) = node.node_contents() {
            non_nil_node
        } else {
            return;
        };

        for (child, item) in non_nil_node.children() {
            self.add_node(child);
            self.items.push(item.clone());
        }

        self.add_node(node.last_child().unwrap())
    }

    fn add_item(&mut self, item: &<M as Monoid>::Item) {
        self.items.push(item.clone())
    }
}
