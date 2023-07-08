extern crate alloc;
use alloc::{vec, vec::Vec};

use super::Accumulator;
use crate::{item::Item, protocol::ProtocolMonoid, range::Range, Node, NonNilNodeRef};

#[derive(Debug, Clone)]
pub struct SplitAccumulator<'a, M>
where
    M: ProtocolMonoid,
    M::Item: Item,
{
    split_sizes: &'a [usize],
    pub(crate) results: Vec<M>,
    pub(crate) ranges: Vec<Range<M::Item>>,
    current_offset: usize,
    update_ranges: bool,
}

impl<'a, M> SplitAccumulator<'a, M>
where
    M: ProtocolMonoid,
    M::Item: Item,
{
    pub fn new(query_range: &Range<M::Item>, split_sizes: &'a [usize]) -> Self {
        let mut state = SplitAccumulator {
            split_sizes,
            results: vec![M::neutral(); split_sizes.len()],
            ranges: vec![query_range.clone(); split_sizes.len()],
            current_offset: 0,
            update_ranges: false,
        };

        state.advance_bucket();

        state
    }

    fn advance_bucket(&mut self) {
        while !self.is_done()
            && self.split_sizes[self.current_offset] <= self.results[self.current_offset].count()
        {
            // the actual result split should never exceed the target split size
            assert_eq!(
                self.split_sizes[self.current_offset],
                self.results[self.current_offset].count()
            );
            self.current_offset += 1;
            self.update_ranges = true;
        }
    }

    fn is_done(&self) -> bool {
        self.current_offset >= self.split_sizes.len()
    }

    fn current_split_size(&self) -> usize {
        self.split_sizes[self.current_offset]
    }

    fn current_result(&mut self) -> &mut M {
        &mut self.results[self.current_offset]
    }

    pub fn results(&self) -> &[M] {
        &self.results
    }

    pub fn ranges(&self) -> &[Range<M::Item>] {
        &self.ranges
    }

    pub fn into_results(self) -> Vec<M> {
        self.results
    }
}
impl<'a, M> Accumulator<M> for SplitAccumulator<'a, M>
where
    M: ProtocolMonoid,
    M::Item: Item,
{
    fn add_node<'b, N: Node<M>>(&mut self, node: &'b N) {
        let non_nil_node = if let Some(non_nil_node) = node.node_contents() {
            non_nil_node
        } else {
            return;
        };

        // I think this assertion should always hold if the caller doesn't mess up.
        assert!(
            !self.is_done(),
            "current state: {self:#?}\n  node to be added: {node:#?}",
        );

        if self.update_ranges {
            let next_item = non_nil_node.min();
            self.ranges[self.current_offset - 1].1 = next_item.clone();
            self.ranges[self.current_offset].0 = next_item.clone();
            self.update_ranges = false;
        }

        let current_split_size = self.current_split_size();
        let current_result = self.current_result();
        let space_left = current_split_size - current_result.count();

        let node_monoid = node.monoid();
        if node_monoid.count() < space_left {
            *current_result = current_result.combine(&node_monoid);
        } else if node_monoid.count() == space_left {
            *current_result = current_result.combine(&node_monoid);
            self.advance_bucket();
        } else {
            for (child, item) in non_nil_node.children() {
                self.add_node(child);
                self.add_item(&item);
            }

            self.add_node(non_nil_node.last_child());
        }
    }

    fn add_item(&mut self, item: &M::Item) {
        assert!(
            !self.is_done(),
            "current state: {self:#?}\nitem to be added: {item:#?}"
        );

        if self.update_ranges {
            self.ranges[self.current_offset - 1].1 = item.clone();
            self.ranges[self.current_offset].0 = item.clone();
            self.update_ranges = false;
        }

        let current_result = self.current_result();
        *current_result = current_result.combine(&M::lift(item));

        self.advance_bucket();
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use std::{collections::HashSet, println};

    extern crate alloc;
    use alloc::format;

    use super::*;

    use crate::monoid::Monoid;
    use crate::query::{simple::SimpleAccumulator, test::TestMonoid};
    use crate::tree::mem_rc_bounds::Node;
    use crate::Node as NodeTrait;

    use proptest::{prelude::*, prop_assert_eq, prop_assume, proptest};

    proptest! {
        #[test]
        fn split_correctness(items in prop::collection::vec(1..1000u64, 3..5usize), from in 0..1000u64, to in 0..1000u64) {
            println!();
            let item_set: HashSet<u64> = HashSet::from_iter(items.iter().cloned());
            let query_range = Range(from, to);
            println!("items used in test: {:?}", item_set);
            println!("query range: {:?}", query_range);

            // items are unique
            prop_assume!(item_set.len() == items.len());

            let mut root = Node::<TestMonoid<u64>>::Nil(TestMonoid::lift(&0));

            for item in &item_set {
                root = root.insert(*item);
            }
            println!("in tree form: {:}", root);

            let mut simple_acc = SimpleAccumulator::new();
            root.query(&query_range, &mut simple_acc);
            let query_result = simple_acc.into_result();

            // make sure we don't glitch on empty splits
            prop_assume!(query_result.count() > 1);

            let item_count = query_result.count();
            let first_bucket_count = item_count/2;
            let second_bucket_count = item_count - first_bucket_count;

            let split_sizes = &[first_bucket_count, second_bucket_count];
            let mut acc = SplitAccumulator::new(&query_range, split_sizes);
            root.query(&query_range, &mut acc);

            // assuming we the splits are >0 (as per the count>1 assumption above), assert that we
            // don't get ranges of the form x..x due to cutting down the range. x..x means full
            // range, not empty range, and this would be a problem.
            prop_assert!(acc.ranges()[0].from() != acc.ranges()[0].to());
            prop_assert!(acc.ranges()[1].from() != acc.ranges()[1].to());

            let mut simple1 = SimpleAccumulator::new();
            let mut simple2 = SimpleAccumulator::new();

            root.query(&acc.ranges()[0], &mut simple1);
            root.query(&acc.ranges()[1], &mut simple2);

            println!("query range: {query_range}");
            println!("split counts: {split_sizes:?}");
            println!("splits: {:?}", acc.ranges());

            prop_assert_eq!((simple1.result(),simple2.result()), (&acc.results()[0], &acc.results()[1]));
        }
    }
}
