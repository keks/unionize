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
#[cfg(test)]

mod test {
    use super::*;
    use crate::query::test::TestMonoid;
    use crate::tree::Node;
    use crate::{monoid::Monoid, range::Range};
    use proptest::{prelude::*, prop_assert_eq, proptest};
    use std::collections::HashSet;

    proptest! {
        #[test]
        fn simple_correctness(items in prop::collection::vec(1..1000u64, 1..100usize), from in 0..1000u64, to in 0..1000u64) {
            println!();
            let item_set: HashSet<u64> = HashSet::from_iter(items.iter().cloned());
            let query_range = Range(from, to);
            println!("items used in test: {:?}", item_set);
            println!("query range: {:?}", query_range);

            let mut root = Node::<TestMonoid<u64>>::Nil(TestMonoid::lift(&0));

            for item in &item_set {
                root = root.insert(*item);
            }
            println!("in tree form: {:}", root);

            let min = items.iter().fold(10000, |acc, x| u64::min(*x, acc));
            let max = items.iter().fold(0, |acc, x| u64::max(*x, acc));

            let ranged_root = RangedNode::new(&root, Range(min, max+1));
            let query_result = ranged_root.query_range(&query_range);
            let mut acc = SimpleAccumulator::new();
            ranged_root.query_range_generic(&query_range, &mut acc);

            prop_assert_eq!(&query_result, acc.result());
        }
    }
}
