use crate::{
    monoid::{Monoid, Peano},
    range::Range,
    XNode,
};

use super::Accumulator;

pub fn query_range_generic<'a, M: Monoid + 'a, N: XNode<'a, M>, A: Accumulator<M>>(
    node: &'a N,
    query_range: &Range<M::Item>,
    state: &mut A,
) {
    println!("query node@{node:?}");
    if node.is_nil() {
        return;
    }

    let min = node.min_item().unwrap();
    let max = node.max_item().unwrap();

    let has_overlap = min <= query_range.to() || max >= query_range.from();
    let is_subrange = if query_range.is_full() {
        true
    } else if query_range.is_wrapping() {
        max < query_range.to() || min >= query_range.from()
    } else {
        query_range.from() <= min && max < query_range.to()
    };

    println!("min:{min:?} max:{max:?} has_overlap:{has_overlap} is_subrange:{is_subrange}");

    if !(has_overlap) {
        return;
    }

    if query_range.from() == query_range.to() {
        // querying full range, node is completely in range,
        // but start at the boundary item, wrap around, and then end at the boundary item.

        // first add items and children after the boundary
        if let Some(new_query_range) = query_range.cap_right(min.clone()) {
            query_range_generic(node, &new_query_range, state);
        }

        // then add items and children before the boundary
        if let Some(new_query_range) = query_range.cap_left(min.clone()) {
            query_range_generic(node, &new_query_range, state);
        }

        return;
    } else if query_range.from() < query_range.to() {
        if is_subrange {
            state.add_xnode(node);
            return;
        }
        // this is a non-wrapping query
        for (child, item) in node.children().unwrap() {
            println!("child: {child:?}");
            query_range_generic(child, query_range, state);
            if query_range.contains(item) {
                state.add_item(&item);
            }
        }

        let child = node.last_child().unwrap();
        println!("child: {child:?} --- last");
        query_range_generic(child, query_range, state);
    } else {
        if is_subrange {
            state.add_xnode(node);
            return;
        }
        // we have a wrapping query

        for (child, item) in node.children().unwrap() {
            if query_range.from() <= item {
                if let Some(next_query_range) = query_range.cap_right(item.clone()) {
                    query_range_generic(child, &next_query_range, state);
                }

                // so it's >= query_range.from,
                // so it's in query_range.
                state.add_item(&item);
            }
        }

        // query the last subtree for elements before the wrap.
        // and we have to restrict it to not include stuff from after the wrap
        {
            let last_child = node.last_child().unwrap();
            if query_range.from() <= &max {
                let next_query_range = query_range
                    .cap_right(max.next())
                    .expect("guaranteed since max.next() > query_range.from()");
                query_range_generic(last_child, &next_query_range, state);
            }
        }

        for (child, item) in node.children().unwrap() {
            let min_item = child.min_item().unwrap();
            if min_item <= query_range.to() {
                if let Some(next_query_range) = query_range.cap_left(min_item.clone()) {
                    query_range_generic(child, &next_query_range, state);
                }

                if item < query_range.to() {
                    state.add_item(&item);
                }
            }
        }

        // The last child may also contain nodes from after wrapping, but that is only the
        // case if last_child.range.from < query_range.to().
        let last_child = node.last_child().unwrap();
        let min_item = last_child.min_item().unwrap();
        if min_item < query_range.to() {
            let next_query_range = query_range
                .cap_left(min_item.clone())
                .expect("guaranteed since min_item < query_range.to()");
            query_range_generic(last_child, &next_query_range, state);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::monoid::Monoid;
    use crate::proto::ProtocolMonoid;
    use crate::query::split::SplitAccumulator;
    use crate::query::{simple::SimpleAccumulator, test::TestMonoid};
    use crate::ranged_node::RangedNode;
    use crate::tree::Node;
    use proptest::{prelude::*, prop_assert_eq, prop_assume, proptest};
    use std::collections::HashSet;

    proptest! {
        #[test]
        fn generic_split_correctness(items in prop::collection::vec(1..1000u64, 3..5usize), from in 0..1000u64, to in 0..1000u64) {
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

            let min = root.min_item().unwrap();
            let max = root.max_item().unwrap();

            let ranged_root = RangedNode::new(&root, Range(*min, max.next()));
            let query_result = ranged_root.query_range(&query_range);

            // make sure we don't glitch on empty splits
            prop_assume!(query_result.count() > 1);
            prop_assert!(query_result.count() <= items.len());

            let item_count = query_result.count();
            let first_bucket_count = item_count/2;
            let second_bucket_count = item_count - first_bucket_count;

            let split_sizes = &[first_bucket_count, second_bucket_count];
            let mut acc = SplitAccumulator::new(&query_range, split_sizes);
            query_range_generic(&root, &query_range, &mut acc);

            // assert that we don't get ranges of the form x..x due to cutting down the range.
            // x..x means full range, not empty range, and this would be a problem.
            prop_assert!(acc.ranges()[0].from() != acc.ranges()[0].to());
            prop_assert!(acc.ranges()[1].from() != acc.ranges()[1].to());

            let mut simple1 = SimpleAccumulator::new();
            let mut simple2 = SimpleAccumulator::new();

            ranged_root.query_range_generic(&acc.ranges()[0], &mut simple1);
            ranged_root.query_range_generic(&acc.ranges()[1], &mut simple2);

            println!("query range: {query_range}");
            println!("split counts: {split_sizes:?}");

            println!("splits: {:?}", acc.ranges());
            println!("results --- simple:{:?} split:{:?}", (simple1.result(),simple2.result()), (&acc.results()[0], &acc.results()[1]));

            prop_assert_eq!((simple1.result(),simple2.result()), (&acc.results()[0], &acc.results()[1]));
        }
    }
}
