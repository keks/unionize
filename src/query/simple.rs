use crate::{monoid::Monoid, Node};

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
    fn add_xnode<'a, N: Node<'a, M>>(&mut self, node: &'a N)
    where
        M: 'a,
    {
        if let Some(children) = node.children() {
            for (child, item) in children {
                self.0 = self.0.combine(child.monoid());
                self.0 = self.0.combine(&M::lift(&item));
            }

            self.0 = self.0.combine(node.last_child().unwrap().monoid());
        }
    }

    fn add_item(&mut self, item: &M::Item) {
        self.0 = self.0.combine(&M::lift(&item));
    }
}
#[cfg(test)]

mod test {
    use super::*;
    use crate::query::generic::query_range_generic;
    use crate::query::test::TestMonoid;
    use crate::tree::mem_rc_bounds::Node;
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


            let mut acc = SimpleAccumulator::new();
            query_range_generic(&root, &query_range, &mut acc);

            let expected = item_set.iter().filter(|item|query_range.contains(item)).fold(TestMonoid::neutral(), |acc, item|acc.combine(&TestMonoid::lift(item)));
            prop_assert_eq!(&expected, acc.result());
        }
    }
}
