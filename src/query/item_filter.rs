extern crate alloc;
extern crate std;

use alloc::{vec, vec::Vec};

use crate::{monoid::Monoid, Item, Node, NonNilNodeRef, Range};

use super::Accumulator;

#[derive(Debug, Clone)]
pub struct ItemFilterAccumulator<'a, M: Monoid> {
    items: &'a [M::Item],
    bits_is_new: Vec<bool>, // TODO replace with bitvec
    cur: usize,
}

impl<'a, M: Monoid> ItemFilterAccumulator<'a, M> {
    pub fn new(items: &'a [M::Item]) -> Self {
        Self {
            items,
            bits_is_new: vec![false; items.len()],
            cur: 0,
        }
    }

    pub fn query_range(&self) -> Option<Range<M::Item>> {
        let min = self.items.first()?.clone();
        let max = self.items.last()?;

        Some(Range(min, max.next()))
    }

    pub fn bits_is_new(&self) -> &[bool] {
        &self.bits_is_new
    }

    pub fn into_bits_is_new(self) -> Vec<bool> {
        self.bits_is_new
    }

    pub fn result(&self) -> impl Iterator<Item = &M::Item> {
        self.items
            .iter()
            .zip(&self.bits_is_new)
            .filter_map(|(item, is_new)| if *is_new { Some(item) } else { None })
    }

    fn cur_item(&self) -> Option<&M::Item> {
        self.items.get(self.cur)
    }
}

impl<'a, M> Accumulator<M> for ItemFilterAccumulator<'a, M>
where
    M: Monoid,
{
    fn add_node<N: Node<M>>(&mut self, node: &N) {
        let non_nil_node = if let Some(non_nil_node) = node.node_contents() {
            non_nil_node
        } else {
            return;
        };

        for (child, item) in non_nil_node.children() {
            self.add_node(child);
            self.add_item(item);
        }

        self.add_node(non_nil_node.last_child())
    }

    fn add_item(&mut self, item: &<M as Monoid>::Item) {
        while matches!(self.cur_item(), Some(cur_item) if item > cur_item) {
            self.bits_is_new[self.cur] = true;
            self.cur += 1;
        }

        if Some(item) == self.cur_item() {
            self.cur += 1;
        }
    }

    fn finalize(&mut self) {
        while self.cur < self.bits_is_new.len() {
            self.bits_is_new[self.cur] = true;
            self.cur += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    use crate::{
        easy::{tests::TestNode, uniform::Node as UniformNode},
        item::le_byte_array::LEByteArray,
        Node as NodeTrait,
    };

    #[test]
    fn base_test() {
        let mut node = UniformNode::nil();
        let item1 = LEByteArray([1; 30]);
        let item2 = LEByteArray([2; 30]);
        node = node.insert(item1);

        let items = vec![item1, item2];

        let mut dedup_acc = ItemFilterAccumulator::new(&items);
        let range: Range<LEByteArray<30>> = dedup_acc.query_range().unwrap();

        assert_eq!(range, Range(item1.clone(), item2.next()));
        node.query(&range, &mut dedup_acc);

        let out: Vec<_> = dedup_acc.result().cloned().collect();
        assert_eq!(out, vec![item2]);
    }

    #[test]
    fn repro_atttempt() {
        let node = TestNode::nil().insert(1).insert(443);
        let items = [443];

        let mut acc = ItemFilterAccumulator::new(&items);
        let range = acc.query_range().unwrap();
        node.query(&range, &mut acc);

        let new: Vec<_> = acc.result().collect();

        assert!(new.is_empty())
    }
}
