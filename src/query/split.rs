use crate::{monoid::Item, proto::ProtocolMonoid, range::Range, ranged_node::RangedNode};

use super::Accumulator;

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
    fn add_node(&mut self, node: &RangedNode<M>) {
        if node.node().monoid().count() == 0 {
            return;
        }

        // I think this assertion should always hold if the caller doesn't mess up.
        assert!(
            !self.is_done(),
            "current state: {self:#?}\nnode to be added: {node:#?}"
        );

        if self.update_ranges {
            let next_item = node.range().from();
            self.ranges[self.current_offset - 1].1 = next_item.clone();
            self.ranges[self.current_offset].0 = next_item.clone();
            self.update_ranges = false;
        }

        let current_split_size = self.current_split_size();
        let current_result = self.current_result();
        let space_left = current_split_size - current_result.count();

        let node_monoid = node.node().monoid();
        if node_monoid.count() < space_left {
            *current_result = current_result.combine(&node_monoid);
        } else if node_monoid.count() == space_left {
            *current_result = current_result.combine(&node_monoid);
            self.advance_bucket();
        } else {
            for (child, item) in node.children() {
                self.add_node(&child.into());
                self.add_item(&item);
            }

            self.add_node(&node.last_child().into());
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
        // let current_count = current_result.count();

        // println!(
        //     "current_offset:{:} current_count:{current_count}",
        //     self.current_offset
        // );

        self.advance_bucket();
    }
}
