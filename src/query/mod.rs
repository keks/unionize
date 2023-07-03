pub mod generic;
pub mod items;
pub mod simple;
pub mod split;

use crate::{
    monoid::{Item, Monoid},
    proto::ProtocolMonoid,
    range::Range,
    ranged_node::RangedNode,
    Node, XNode,
};
use split::SplitAccumulator;

impl<'a, M> RangedNode<'a, M>
where
    M: Monoid,
    M::Item: Item,
{
    pub fn query_range(&self, query_range: &Range<M::Item>) -> M {
        let mut acc = simple::SimpleAccumulator::new();
        self.query_range_generic(query_range, &mut acc);
        acc.into_result()
    }
}

impl<'a, M> RangedNode<'a, M>
where
    M: ProtocolMonoid,
    M::Item: Item,
{
    pub fn query_range_split(&self, query_range: &Range<M::Item>, split_sizes: &[usize]) -> Vec<M> {
        let mut state = SplitAccumulator::new(query_range, split_sizes);
        self.query_range_generic(query_range, &mut state);

        state.results
    }
}

pub trait Accumulator<M>: std::fmt::Debug
where
    M: Monoid,
    M::Item: Item,
{
    fn add_node(&mut self, node: &RangedNode<M>);
    fn add_xnode<'a, N: XNode<'a, M>>(&mut self, node: &'a N)
    where
        M: 'a;
    fn add_item(&mut self, item: &M::Item);
}

impl<'a, M> RangedNode<'a, M>
where
    M: Monoid,
    M::Item: Item,
{
    pub fn query_range_generic<A: Accumulator<M>>(
        &self,
        query_range: &Range<M::Item>,
        state: &mut A,
    ) {
        println!(
            "y query:{query_range:?} node:{:?} node_range:{:?}",
            self.node(),
            self.range()
        );
        if !self.range().has_overlap(query_range) {
            return;
        }

        let node: &Node<M> = self.node();
        if node.is_nil() {
            return;
        }

        if query_range.from() == query_range.to() {
            // querying full range, node is completely in range,
            // but start at the boundary item, wrap around, and then end at the boundary item.

            // // but if the boundary is the start of the node, just add the node
            // if query_range.from() == self.range().from() {
            //     state.add_node(&self);
            //     return;
            // }

            // first add items and children after the boundary
            if let Some(new_query_range) = query_range.cap_right(self.range().from().clone()) {
                self.query_range_generic(&new_query_range, state);
            }

            // then add items and children before the boundary
            if let Some(new_query_range) = query_range.cap_left(self.range().from().clone()) {
                self.query_range_generic(&new_query_range, state);
            }

            return;
        } else if query_range.from() < query_range.to() {
            if self.range().is_subrange_of(query_range) {
                state.add_node(&self);
                return;
            }
            // this is a non-wrapping query
            for (child, ref item) in self.children() {
                child.query_range_generic(query_range, state);
                if query_range.contains(item) {
                    println!("y adding item {item:?}");
                    state.add_item(&item);
                    println!("{state:?}");
                }
            }

            self.last_child().query_range_generic(query_range, state);
        } else {
            if self.range().is_subrange_of(query_range) {
                state.add_node(&self);
                return;
            }
            // we have a wrapping query

            for (child, ref item) in self.children() {
                if child.range().to() >= query_range.from() {
                    let next_upper_bound = child.range().to();
                    if let Some(next_query_range) = query_range.cap_right(next_upper_bound.clone())
                    {
                        child.query_range_generic(&next_query_range, state);
                    }

                    // item is >= child.range.to,
                    // so it's >= query_range.from,
                    // so it's in query_range.
                    state.add_item(&item);
                }
            }

            // query the last subtree for elements before the wrap.
            // and we have to restrict it to not include stuff from after the wrap
            {
                let last_child = self.last_child();
                if last_child.range().to() > query_range.from() {
                    let next_upper_bound = last_child.range().to();
                    let next_query_range = query_range
                        .cap_right(next_upper_bound.clone())
                        .expect("guaranteed since next_lower_bound > query_range.from()");
                    last_child.query_range_generic(&next_query_range, state);
                }
            }

            for (child, ref item) in self.children() {
                if child.range().from() <= query_range.to() {
                    let next_lower_bound = child.range().from();
                    if let Some(next_query_range) = query_range.cap_left(next_lower_bound.clone()) {
                        child.query_range_generic(&next_query_range, state);
                    }

                    if item < query_range.to() {
                        state.add_item(&item);
                    }
                }
            }

            // The last child may also contain nodes from after wrapping, but that is only the
            // case if last_child.range.from < query_range.to().
            let last_child = self.last_child();
            let next_lower_bound = last_child.range().from();
            if next_lower_bound < query_range.to() {
                let next_query_range = query_range
                    .cap_left(next_lower_bound.clone())
                    .expect("guaranteed since next_lower_bound < query_range.to()");
                last_child.query_range_generic(&next_query_range, state);
            }
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::{
        monoid::sum::{SumItem, SumMonoid},
        monoid::{count::CountingMonoid, Monoid},
        proto::ProtocolMonoid,
        query::{items::ItemsAccumulator, SplitAccumulator},
        range::Range,
        ranged_node::RangedNode,
        Node,
    };
    use proptest::{prelude::prop, proptest};
    use std::collections::HashSet;

    pub type TestMonoid<T> = CountingMonoid<SumMonoid<T>>;

    impl SumItem for u64 {
        fn zero() -> u64 {
            0
        }
    }

    proptest! {
        #[test]
        fn query_generic_items(items in prop::collection::vec(1..1000u64, 1..100usize), from in 0..1000u64, to in 0..1000u64) {
            println!();
            let item_set: HashSet<u64> = HashSet::from_iter(items.iter().cloned());
            let query_range = Range(from, to);
            println!("items used in test: {:?}", item_set);
            println!("query range: {:?}", query_range);

            let mut root = Node::<TestMonoid<u64>>::Nil(TestMonoid::lift(&0));

            let mut items_sorted = vec![];
            for item in &item_set {
                items_sorted.push(item.clone());
                root = root.insert(*item);
            }
            println!("in tree form: {:}", root);

            items_sorted.sort();

            let min = items.iter().fold(10000, |acc, x| u64::min(*x, acc));
            let max = items.iter().fold(0, |acc, x| u64::max(*x, acc));
            let ranged_root = RangedNode::new(&root, Range(min, max+1));

            let mut acc = ItemsAccumulator::new();
            ranged_root.query_range_generic(&query_range, &mut acc);

            let generic_query_result = acc.results().to_vec();

            let expected: Vec<_> = if query_range.is_full() {
                let boundary = query_range.from();
                let mut lesser_items : Vec<_> = items_sorted.iter().filter(|item| *item < boundary).cloned().collect();
                let greater_items: Vec<_> = items_sorted.iter().filter(|item| *item >= boundary).cloned().collect();

                let mut items = greater_items;
                items.append(&mut lesser_items);
                items
            } else if query_range.is_wrapping() {
                items_sorted.iter().filter(|item|*item >= query_range.from()).chain(items_sorted.iter().filter(|item| *item < query_range.to())).cloned().collect()
            } else {
                items_sorted.iter().filter(|item|query_range.contains(item)).cloned().collect()
            };

            assert_eq!(generic_query_result, expected);
        }
    }

    proptest! {
        #[test]
        fn query_generic_split_distinct_ranges(items in prop::collection::vec(1..1000u64, 1..100usize), from in 0..1000u64, to in 0..1000u64) {
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

            let item_count = query_result.count();
            let first_bucket_count = item_count/2;
            let second_bucket_count = item_count - first_bucket_count;

            let split_sizes = &[first_bucket_count, second_bucket_count];
            println!("split sizes: {split_sizes:?}");
            let mut acc = SplitAccumulator::new(&query_range, split_sizes);
            ranged_root.query_range_generic(&query_range, &mut acc);
            let ranges = acc.ranges().to_vec();
            let ranges_set: HashSet<Range<_>> = HashSet::from_iter(ranges.iter().cloned());

            if !split_sizes.contains(&0) {
                assert_eq!(ranges.len(), ranges_set.len(), "{ranges:?} != {ranges_set:?}")
            }
        }
    }

    proptest! {
        #[test]
        fn query_generic_split_equivalence(items in prop::collection::vec(1..1000u64, 1..100usize), from in 0..1000u64, to in 0..1000u64) {
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

            let item_count = query_result.count();
            let first_bucket_count = item_count/2;
            let second_bucket_count = item_count - first_bucket_count;

            let split_sizes = &[first_bucket_count, second_bucket_count];
            let split_query_result = ranged_root.query_range_split(&query_range, split_sizes);

            let mut acc = SplitAccumulator::new(&query_range, split_sizes);
            ranged_root.query_range_generic(&query_range, &mut acc);
            let generic_query_result = acc.results().to_vec();

            assert_eq!(split_query_result, generic_query_result);
        }
    }

    proptest! {
        #[test]
        fn new_split_range_query_correctness_two_buckets(items in prop::collection::vec(1..1000u64, 1..100usize), from in 0..1000u64, to in 0..1000u64) {
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

            let item_count = query_result.count();
            let first_bucket_count = item_count/2;
            let second_bucket_count = item_count - first_bucket_count;

            let query_result = ranged_root.query_range_split(&query_range, &[first_bucket_count, second_bucket_count]);

            let mut items_sorted: Vec<_> = item_set.iter().cloned().collect();
            items_sorted.sort();

            // if this is a wrapping query we need to reorder such that the ones before the wrap
            // come first
            if from > to {
                if let Some(pos) = items_sorted.iter().position(|item| from <= *item) {
                    let mut new_items_sorted = vec![];
                    new_items_sorted.extend(&items_sorted[pos..]);
                    new_items_sorted.extend(&items_sorted[..pos]);
                    items_sorted = new_items_sorted;
                }
            }

            println!("sorted and possibly reordered items (from={from}):\n{items_sorted:?}");

            let matching_items: Vec<_> = if from < to {
                items_sorted.iter().cloned().filter(|item| from <= *item && *item < to).collect()
            } else if from > to {
                items_sorted.iter().cloned().filter(|item| from <= *item || *item < to).collect()
            } else {
                let mut items = Vec::with_capacity(items_sorted.len());
                items.extend(items_sorted.iter().cloned().filter(|item| *item >= from));
                items.extend(items_sorted.iter().cloned().filter(|item| *item < from));
                items
            };

            assert_eq!(item_count, matching_items.len());

            let first_bucket_items = &matching_items[..first_bucket_count];
            let second_bucket_items = &matching_items[first_bucket_count..];

            println!("sizes: [{first_bucket_count}, {second_bucket_count}]");
            println!("first bucket items: {first_bucket_items:?}");
            println!("second bucket items: {second_bucket_items:?}");

            let first_bucket: TestMonoid<u64> = first_bucket_items.iter().fold(TestMonoid::neutral(), |acc, el| acc.combine(&TestMonoid::lift(&el)));
            let second_bucket: TestMonoid<u64> = second_bucket_items.iter().fold(TestMonoid::neutral(), |acc, el| acc.combine(&TestMonoid::lift(&el)));

            assert_eq!(vec![first_bucket, second_bucket], query_result);
        }
    }

    proptest! {
        #[test]
        fn new_split_range_query_correctness_single_bucket(items in prop::collection::vec(1..1000u64, 1..100usize), from in 0..1000u64, to in 0..1000u64) {
            println!();
            let item_set: HashSet<u64> = HashSet::from_iter(items.iter().cloned());
            println!("items used in test: {:?}", item_set);

            let mut root = Node::<TestMonoid<u64>>::Nil(TestMonoid::lift(&0));

            for item in &item_set {
                root = root.insert(*item);
            }
            println!("in tree form: {:}", root);


            let min = items.iter().fold(10000, |acc, x| u64::min(*x, acc));
            let max = items.iter().fold(0, |acc, x| u64::max(*x, acc));

            let ranged_root = RangedNode::new(&root, Range(min, max+1));
            let query_result = ranged_root.query_range_split(&Range(from, to), &[item_set.len()]);

            let expected = if from < to {
                item_set.iter().filter(|item| from <= **item && **item < to).fold(TestMonoid::neutral(), |acc, item| acc.combine(&TestMonoid::lift(item)))
            } else if from > to {
                item_set.iter().filter(|item| from <= **item || **item < to).fold(TestMonoid::neutral(), |acc, item| acc.combine(&TestMonoid::lift(item)))
            } else {
                item_set.iter().fold(TestMonoid::neutral(), |acc, item| acc.combine(&TestMonoid::lift(item)))
            };

            assert_eq!(expected, query_result[0]);
        }
    }

    proptest! {
        fn range_query_simple_correctness(items in prop::collection::vec(1..1000u64, 1..100usize), from in 0..1000u64, to in 0..1000u64) {
            println!();
            let item_set: HashSet<u64> = HashSet::from_iter(items.iter().cloned());
            println!("items used in test: {:?}", item_set);

            let mut root = Node::<TestMonoid<u64>>::Nil(TestMonoid::lift(&0));

            for item in &item_set {
                root = root.insert(*item);
            }
            println!("in tree form: {:}", root);


            let min = items.iter().fold(10000, |acc, x| u64::min(*x, acc));
            let max = items.iter().fold(0, |acc, x| u64::max(*x, acc));

            let query_range = Range(from, to);
            let ranged_root = RangedNode::new(&root, Range(min, max+1));
            let mut acc = super::simple::SimpleAccumulator::new();
            ranged_root.query_range_generic(&query_range, &mut acc);

            let expected = if from < to {
                item_set.iter().filter(|item| from <= **item && **item < to).fold(TestMonoid::neutral(), |acc, item| acc.combine(&TestMonoid::lift(item)))
            } else if from > to {
                item_set.iter().filter(|item| from <= **item || **item < to).fold(TestMonoid::neutral(), |acc, item| acc.combine(&TestMonoid::lift(item)))
            } else {
                item_set.iter().fold(TestMonoid::neutral(), |acc, item| acc.combine(&TestMonoid::lift(item)))
            };

            assert_eq!(expected, acc.result().clone());
        }
    }

    proptest! {
        #[test]
        fn new_range_query_correctness(items in prop::collection::vec(1..1000u64, 1..100usize), from in 0..1000u64, to in 0..1000u64) {
            println!();
            let item_set: HashSet<u64> = HashSet::from_iter(items.iter().cloned());
            println!("items used in test: {:?}", item_set);

            let mut root = Node::<SumMonoid<u64>>::Nil(SumMonoid::lift(&0));

            for item in &item_set {
                root = root.insert(*item);
            }
            println!("in tree form: {:}", root);


            let min = items.iter().fold(10000, |acc, x| u64::min(*x, acc));
            let max = items.iter().fold(0, |acc, x| u64::max(*x, acc));

            let ranged_root = RangedNode::new(&root, Range(min, max+1));
            let query_result = ranged_root.query_range(&Range(from, to));

            let expected = if from < to {
                item_set.iter().filter(|item| from <= **item && **item < to).fold(SumMonoid(0), |acc, item| acc.combine(&SumMonoid::lift(item)))
            } else if from > to {
                item_set.iter().filter(|item| from <= **item || **item < to).fold(SumMonoid(0), |acc, item| acc.combine(&SumMonoid::lift(item)))
            } else {
                item_set.iter().fold(SumMonoid(0), |acc, item| acc.combine(&SumMonoid::lift(item)))
            };

            assert_eq!(expected, query_result);
        }
    }

    #[test]
    fn repro_3() {
        let items = vec![13, 30, 395, 899];
        let from = 904;
        let to = 442;
        let query_range = Range(from, to);
        println!("items used in test: {:?}", items);
        println!("query range: {:?}", query_range);

        let mut root = Node::<TestMonoid<u64>>::Nil(TestMonoid::lift(&0));

        for item in &items {
            root = root.insert(*item);
        }
        println!("in tree form: {:}", root);

        let min = items.iter().fold(10000, |acc, x| u64::min(*x, acc));
        let max = items.iter().fold(0, |acc, x| u64::max(*x, acc));

        let ranged_root = RangedNode::new(&root, Range(min, max + 1));
        let query_result = ranged_root.query_range(&query_range);

        println!("------------------------------");

        let item_count = query_result.count();
        let first_bucket_count = item_count / 2;
        let second_bucket_count = item_count - first_bucket_count;

        let query_result =
            ranged_root.query_range_split(&query_range, &[first_bucket_count, second_bucket_count]);

        let mut items_sorted: Vec<_> = items.iter().cloned().collect();
        items_sorted.sort();

        // if this is a wrapping query we need to reorder such that the ones before the wrap
        // come first
        if from > to {
            if let Some(pos) = items_sorted.iter().position(|item| from <= *item) {
                let mut new_items_sorted = vec![];
                new_items_sorted.extend(&items_sorted[pos..]);
                new_items_sorted.extend(&items_sorted[..pos]);
                items_sorted = new_items_sorted;
            }
        }

        println!("sorted and possibly reordered items (from={from}):\n{items_sorted:?}");

        let matching_items: Vec<_> = if from < to {
            items_sorted
                .iter()
                .cloned()
                .filter(|item| from <= *item && *item < to)
                .collect()
        } else if from > to {
            items_sorted
                .iter()
                .cloned()
                .filter(|item| from <= *item || *item < to)
                .collect()
        } else {
            items_sorted.clone()
        };

        assert_eq!(item_count, matching_items.len());

        let first_bucket_items = &matching_items[..first_bucket_count];
        let second_bucket_items = &matching_items[first_bucket_count..];

        println!("first bucket items: {first_bucket_items:?}");
        println!("second bucket items: {second_bucket_items:?}");

        let first_bucket: TestMonoid<u64> = first_bucket_items
            .iter()
            .fold(TestMonoid::neutral(), |acc, el| {
                acc.combine(&TestMonoid::lift(&el))
            });
        let second_bucket: TestMonoid<u64> = second_bucket_items
            .iter()
            .fold(TestMonoid::neutral(), |acc, el| {
                acc.combine(&TestMonoid::lift(&el))
            });

        assert_eq!(vec![first_bucket, second_bucket], query_result);
    }

    #[test]
    fn repro_2() {
        let items = vec![804, 826, 219, 900, 343, 721, 916];
        let from = 695;
        let to = 227;
        let query_range = Range(from, to);
        println!("items used in test: {:?}", items);
        println!("query range: {:?}", query_range);

        let mut root = Node::<TestMonoid<u64>>::Nil(TestMonoid::lift(&0));

        for item in &items {
            root = root.insert(*item);
        }
        println!("in tree form: {:}", root);

        let min = items.iter().fold(10000, |acc, x| u64::min(*x, acc));
        let max = items.iter().fold(0, |acc, x| u64::max(*x, acc));

        let ranged_root = RangedNode::new(&root, Range(min, max + 1));
        let query_result = ranged_root.query_range(&query_range);

        println!("------------------------------");

        let item_count = query_result.count();
        let first_bucket_count = item_count / 2;
        let second_bucket_count = item_count - first_bucket_count;

        let query_result =
            ranged_root.query_range_split(&query_range, &[first_bucket_count, second_bucket_count]);

        let mut items_sorted: Vec<_> = items.iter().cloned().collect();
        items_sorted.sort();

        // if this is a wrapping query we need to reorder such that the ones before the wrap
        // come first
        if from > to {
            if let Some(pos) = items_sorted.iter().position(|item| from <= *item) {
                let mut new_items_sorted = vec![];
                new_items_sorted.extend(&items_sorted[pos..]);
                new_items_sorted.extend(&items_sorted[..pos]);
                items_sorted = new_items_sorted;
            }
        }

        println!("sorted and possibly reordered items (from={from}):\n{items_sorted:?}");

        let matching_items: Vec<_> = if from < to {
            items_sorted
                .iter()
                .cloned()
                .filter(|item| from <= *item && *item < to)
                .collect()
        } else if from > to {
            items_sorted
                .iter()
                .cloned()
                .filter(|item| from <= *item || *item < to)
                .collect()
        } else {
            items_sorted.clone()
        };

        assert_eq!(item_count, matching_items.len());

        let first_bucket_items = &matching_items[..first_bucket_count];
        let second_bucket_items = &matching_items[first_bucket_count..];

        println!("sizes: [{first_bucket_count}, {second_bucket_count}]");
        println!("first bucket items: {first_bucket_items:?}");
        println!("second bucket items: {second_bucket_items:?}");

        let first_bucket: TestMonoid<u64> = first_bucket_items
            .iter()
            .fold(TestMonoid::neutral(), |acc, el| {
                acc.combine(&TestMonoid::lift(&el))
            });
        let second_bucket: TestMonoid<u64> = second_bucket_items
            .iter()
            .fold(TestMonoid::neutral(), |acc, el| {
                acc.combine(&TestMonoid::lift(&el))
            });

        assert_eq!(vec![first_bucket, second_bucket], query_result);
    }

    #[test]
    fn repro_1() {
        let items = vec![196, 197, 198];
        let from = 196;
        let to = 195;

        let mut root = Node::<SumMonoid<u64>>::Nil(SumMonoid::lift(&0));
        for item in &items {
            root = root.insert(*item);
        }

        println!("items used in test: {:?}", items);
        println!("from:{from} to:{to}");
        println!("in tree form: {:}", root);

        let min = items.iter().fold(10000, |acc, x| u64::min(*x, acc));
        let max = items.iter().fold(0, |acc, x| u64::max(*x, acc));

        let ranged_root = RangedNode::new(&root, Range(min - 1, max + 1));
        let query_result = ranged_root.query_range(&Range(from, to));

        let expected = if from < to {
            items
                .iter()
                .filter(|item| from <= **item && **item < to)
                .fold(SumMonoid(0), |acc, item| {
                    acc.combine(&SumMonoid::lift(item))
                })
        } else if from > to {
            items
                .iter()
                .filter(|item| from <= **item || **item < to)
                .fold(SumMonoid(0), |acc, item| {
                    acc.combine(&SumMonoid::lift(item))
                })
        } else {
            items.iter().fold(SumMonoid(0), |acc, item| {
                acc.combine(&SumMonoid::lift(item))
            })
        };

        assert_eq!(expected, query_result);
    }

    #[test]
    fn repro_0() {
        let items = vec![42, 754, 572, 1];
        let from = 533;
        let to = 442;

        let mut root = Node::<SumMonoid<u64>>::Nil(SumMonoid::lift(&0));
        for item in &items {
            root = root.insert(*item);
        }

        println!("items used in test: {:?}", items);
        println!("from:{from} to:{to}");
        println!("in tree form: {:}", root);

        let min = items.iter().fold(10000, |acc, x| u64::min(*x, acc));
        let max = items.iter().fold(0, |acc, x| u64::max(*x, acc));

        let ranged_root = RangedNode::new(&root, Range(min - 1, max + 1));
        let query_result = ranged_root.query_range(&Range(from, to));

        let expected = if from < to {
            items
                .iter()
                .filter(|item| from <= **item && **item < to)
                .fold(SumMonoid(0), |acc, item| {
                    acc.combine(&SumMonoid::lift(item))
                })
        } else if from > to {
            items
                .iter()
                .filter(|item| from <= **item || **item < to)
                .fold(SumMonoid(0), |acc, item| {
                    acc.combine(&SumMonoid::lift(item))
                })
        } else {
            items.iter().fold(SumMonoid(0), |acc, item| {
                acc.combine(&SumMonoid::lift(item))
            })
        };

        assert_eq!(expected, query_result);
    }
}
