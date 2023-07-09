use crate::{monoid::Monoid, Node, NonNilNodeRef};

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
    fn add_node<'a, N: Node<M>>(&mut self, node: &'a N) {
        if let Some(non_nil_node) = node.node_contents() {
            for (child, item) in non_nil_node.children() {
                self.0 = self.0.combine(child.monoid());
                self.0 = self.0.combine(&M::lift(&item));
            }

            self.0 = self.0.combine(non_nil_node.last_child().monoid());
        }
    }

    fn add_item(&mut self, item: &M::Item) {
        self.0 = self.0.combine(&M::lift(&item));
    }
}
#[cfg(test)]

mod test {
    extern crate std;
    use std::{collections::HashSet, println};

    extern crate alloc;
    use alloc::format;

    use super::*;

    use crate::easy::tests::{TestMonoid, TestNode};
    use crate::Node;
    use crate::{monoid::Monoid, range::Range};

    use proptest::{prelude::*, prop_assert_eq, proptest};

    proptest! {
        #[test]
        fn simple_correctness(items in prop::collection::vec(1..1000u64, 1..100usize), from in 0..1000u64, to in 0..1000u64) {
            println!();
            let item_set: HashSet<u64> = HashSet::from_iter(items.iter().cloned());
            let query_range = Range(from, to);
            println!("items used in test: {:?}", item_set);
            println!("query range: {:?}", query_range);

            let mut root = TestNode::nil();

            for item in &item_set {
                root = root.insert(*item);
            }
            println!("in tree form: {:}", root);


            let mut acc = SimpleAccumulator::new();
            root.query(&query_range, &mut acc);

            let expected = item_set.iter().filter(|item|query_range.contains(item)).fold(TestMonoid::neutral(), |acc, item|acc.combine(&TestMonoid::lift(item)));
            prop_assert_eq!(&expected, acc.result());
        }
    }
}
