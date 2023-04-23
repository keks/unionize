use crate::{
    proto::ProtocolMonoid,
    range::{NewRange, Rangable, Range, RangeCompare},
    ranged_node::{RangedNode, RangedRcNode},
    tree::ChildId,
    LiftingMonoid, Node,
};
use std::fmt::Debug;

// This algorithm seems to work, hence I definitely want to keep it.
// However, in practice we need something that computes the range fingerprint and splits at the same time.
// I will therefore copy this code and make modifications into the block below.
impl<M: LiftingMonoid> Node<M>
where
    M::Item: Debug,
{
    pub(crate) fn query_range_monoid(&self, query_range: &Range<M::Item>) -> M {
        self.query_range_monoid_inner(query_range, &Range::Full)
    }

    fn query_range_monoid_inner(
        &self,
        query_range: &Range<M::Item>,
        node_range: &Range<M::Item>,
    ) -> M {
        //println!("nd:{node}");
        //println!("qr:{query_range:?}");
        //println!("nr:{node_range:?}");
        //println!("");

        if node_range.is_subrange_of(query_range) {
            return self.monoid().clone();
        }

        assert!(node_range.has_overlap(query_range));
        assert!(query_range.is_valid());

        match self {
            Node::Nil(_) => M::neutral(),
            Node::Node2(node_data) => {
                let item = &node_data.get_item(0).unwrap();
                let left_child = &node_data.child_by_child_id(ChildId::Normal(0)).unwrap();
                let right_child = &node_data.child_by_child_id(ChildId::Last).unwrap();
                match query_range.cmp(item) {
                    // format: L i R
                    // parens denote the range
                    // capital is subtree
                    // lower-case is item
                    // range can not only contain item

                    // this only comes up for NewRange comparisons, so we're good
                    RangeCompare::InBetween => unreachable!(),

                    // [L) i R
                    RangeCompare::GreaterThan => left_child
                        .query_range_monoid_inner(query_range, &node_range.with_end(item.clone())),

                    // [L i) R
                    RangeCompare::IsUpperBound => {
                        let left_monoid = left_child.query_range_monoid_inner(
                            query_range,
                            &node_range.with_end(item.clone()),
                        );

                        left_monoid
                    }

                    // [L i R)
                    RangeCompare::Included => {
                        let first = left_child.query_range_monoid_inner(
                            query_range,
                            &node_range.with_end(item.clone()),
                        );
                        let second = M::lift(item);
                        let third = right_child.query_range_monoid_inner(
                            query_range,
                            &node_range.with_start(item.clone()),
                        );

                        first.combine(&second).combine(&third)
                    }

                    // L [i R)
                    RangeCompare::IsLowerBound => {
                        let item_monoid = M::lift(item);
                        let right_monoid = right_child.query_range_monoid_inner(
                            query_range,
                            &node_range.with_start(item.clone()),
                        );

                        item_monoid.combine(&right_monoid)
                    }

                    // L i [R)
                    RangeCompare::LessThan => right_child.query_range_monoid_inner(
                        query_range,
                        &node_range.with_start(item.clone()),
                    ),
                }
            }
            Node::Node3(node_data) => {
                let item1 = &node_data.get_item(0).unwrap();
                let item2 = &node_data.get_item(1).unwrap();
                let left_child = &node_data.child_by_child_id(ChildId::Normal(0)).unwrap();
                let middle_child = &node_data.child_by_child_id(ChildId::Normal(1)).unwrap();
                let right_child = &node_data.child_by_child_id(ChildId::Last).unwrap();

                let cmp1 = query_range.cmp(item1);
                let cmp2 = query_range.cmp(item2);

                //println!("cmp 3 | {cmp1:?} {cmp2:?}");

                match (cmp1, cmp2) {
                    // format: L i1 M i2 R
                    // parens denote the range
                    // capital is subtree
                    // lower-case is item
                    // range can not only contain item

                    // this only comes up for NewRange comparisons, so we're good
                    (RangeCompare::InBetween, _) => unreachable!(),
                    (_, RangeCompare::InBetween) => unreachable!(),

                    // (L) i1 M i2 R
                    (RangeCompare::GreaterThan, RangeCompare::GreaterThan) => left_child
                        .query_range_monoid_inner(query_range, &node_range.with_end(item1.clone())),

                    // (L i1) M i2 R
                    (RangeCompare::IsUpperBound, RangeCompare::GreaterThan) => {
                        let left_monoid = left_child.query_range_monoid_inner(
                            query_range,
                            &node_range.with_end(item1.clone()),
                        );

                        left_monoid
                    }

                    // (L i1 M) i2 R
                    (RangeCompare::Included, RangeCompare::GreaterThan) => {
                        let left_monoid = left_child.query_range_monoid_inner(
                            query_range,
                            &node_range.with_end(item1.clone()),
                        );
                        let item_monoid = M::lift(item1);
                        let middle_monoid = middle_child.query_range_monoid_inner(
                            query_range,
                            &node_range.with_end(item2.clone()).with_start(item1.clone()),
                        );

                        left_monoid.combine(&item_monoid).combine(&middle_monoid)
                    }

                    // (L i1 M i2) R
                    (RangeCompare::Included, RangeCompare::IsUpperBound) => {
                        let left_monoid = left_child.query_range_monoid_inner(
                            query_range,
                            &node_range.with_end(item1.clone()),
                        );
                        let item1_monoid = M::lift(item1);
                        let middle_monoid = middle_child.query_range_monoid_inner(
                            query_range,
                            &node_range.with_end(item2.clone()).with_start(item1.clone()),
                        );

                        left_monoid.combine(&item1_monoid).combine(&middle_monoid)
                    }

                    // (L i1 M i2 R)
                    (RangeCompare::Included, RangeCompare::Included) => {
                        let left_monoid = left_child.query_range_monoid_inner(
                            query_range,
                            &node_range.with_end(item1.clone()),
                        );
                        let item1_monoid = M::lift(item1);
                        let middle_monoid = middle_child.query_range_monoid_inner(
                            query_range,
                            &node_range.with_end(item2.clone()).with_start(item1.clone()),
                        );
                        let item2_monoid = M::lift(item2);
                        let right_monoid = right_child.query_range_monoid_inner(
                            query_range,
                            &node_range.with_start(item2.clone()),
                        );

                        left_monoid
                            .combine(&item1_monoid)
                            .combine(&middle_monoid)
                            .combine(&item2_monoid)
                            .combine(&right_monoid)
                    }

                    // L (i1 M) i2 R
                    (RangeCompare::IsLowerBound, RangeCompare::GreaterThan) => {
                        let item1_monoid = M::lift(item1);
                        let middle_monoid = middle_child.query_range_monoid_inner(
                            query_range,
                            &node_range.with_end(item2.clone()).with_start(item1.clone()),
                        );

                        item1_monoid.combine(&middle_monoid)
                    }

                    // L (i1 M i2) R
                    (RangeCompare::IsLowerBound, RangeCompare::IsUpperBound) => {
                        let item1_monoid = M::lift(item1);
                        let middle_monoid = middle_child.query_range_monoid_inner(
                            query_range,
                            &node_range.with_end(item2.clone()).with_start(item1.clone()),
                        );

                        item1_monoid.combine(&middle_monoid)
                    }

                    // L (i1 M i2 R)
                    (RangeCompare::IsLowerBound, RangeCompare::Included) => {
                        let item1_monoid = M::lift(item1);
                        let middle_monoid = middle_child.query_range_monoid_inner(
                            query_range,
                            &node_range.with_end(item2.clone()).with_start(item1.clone()),
                        );
                        let item2_monoid = M::lift(item2);
                        let right_monoid = right_child.query_range_monoid_inner(
                            query_range,
                            &node_range.with_start(item2.clone()),
                        );

                        item1_monoid
                            .combine(&middle_monoid)
                            .combine(&item2_monoid)
                            .combine(&right_monoid)
                    }
                    // L i1 (M) i2 R
                    (RangeCompare::LessThan, RangeCompare::GreaterThan) => middle_child
                        .query_range_monoid_inner(
                            query_range,
                            &node_range.with_end(item2.clone()).with_start(item1.clone()),
                        ),
                    // L i1 (M i2) R
                    (RangeCompare::LessThan, RangeCompare::IsUpperBound) => {
                        let middle_monoid = middle_child.query_range_monoid_inner(
                            query_range,
                            &node_range.with_end(item2.clone()).with_start(item1.clone()),
                        );

                        middle_monoid
                    }
                    // L i1 (M i2 R)
                    (RangeCompare::LessThan, RangeCompare::Included) => {
                        let middle_monoid = middle_child.query_range_monoid_inner(
                            query_range,
                            &node_range.with_end(item2.clone()).with_start(item1.clone()),
                        );
                        let item2_monoid = M::lift(item2);
                        let right_monoid = right_child.query_range_monoid_inner(
                            query_range,
                            &node_range.with_start(item2.clone()),
                        );

                        middle_monoid.combine(&item2_monoid).combine(&right_monoid)
                    }
                    // L i1 M (i2 R)
                    (RangeCompare::LessThan, RangeCompare::IsLowerBound) => {
                        let item2_monoid = M::lift(item2);
                        let right_monoid = right_child.query_range_monoid_inner(
                            query_range,
                            &node_range.with_start(item2.clone()),
                        );

                        item2_monoid.combine(&right_monoid)
                    }
                    // L i1 M i2 (R)
                    (RangeCompare::LessThan, RangeCompare::LessThan) => right_child
                        .query_range_monoid_inner(
                            query_range,
                            &node_range.with_start(item2.clone()),
                        ),

                    // the rest doen't make sense logically. i think.
                    _ => unreachable!(),
                }
            }
        }
    }
}

// This is the same as above, but with the new ranges
impl<'a, M: LiftingMonoid> RangedNode<'a, M>
where
    M::Item: Debug + Rangable,
{
    pub(crate) fn query_range_monoid_new(&self, query_range: &NewRange<M::Item>) -> M {
        if !self.range().has_overlap(query_range) {
            println!("returning zero because root and query don't overlap");
            return M::neutral();
        }

        self.query_range_inner_next_attempt(query_range)
    }

    fn query_range_inner_next_attempt(&self, query_range: &NewRange<M::Item>) -> M {
        // the only time the node range can be wrapping is when it's full.
        // in other words, from <= to (where from == to means full range)
        println!("enter:  N:{:?}/Q:{:?}", self.range(), query_range);

        //assert!(!self.range().is_wrapping() || self.range().is_full());
        assert!(self.range().has_overlap(query_range));

        let node: &Node<M> = self.node();

        if node.is_nil() {
            println!(
                "return: N:{:?}/Q:{:?} - zero because node is nil",
                self.range(),
                query_range
            );
            return M::neutral();
        }

        if self.range().is_subrange_of(query_range) {
            return self.node().monoid().clone();
        }
        // todo: add an is_contained_in shortcut so we can just return the total

        let NewRange(query_from, query_to) = query_range;

        if query_from == query_to {
            println!(
                "return: N:{:?}/Q:{:?} - query is full, so returning monoid",
                self.range(),
                query_range
            );
            return node.monoid().clone();
        } else {
            if query_from < query_to {
                // this is a non-wrapping query

                let mut result = M::neutral();

                for (child, ref item) in self.children() {
                    println!(
                        "(child:{:?}:{:?}) - item:{:?}..",
                        child.range(),
                        child.node().monoid(),
                        item
                    );
                    if query_from < item && item < query_to {
                        println!("case a - both subtree (at least partly) and item in range");
                        let child: RangedNode<'a, M> = child.into();
                        let child_result = child.query_range_inner_next_attempt(query_range);
                        result = result.combine(&child_result);
                        result = result.combine(&M::lift(item));
                    } else if query_from == item && item < query_to {
                        println!("case b - item in range");
                        result = result.combine(&M::lift(item));
                    } else if child.range().has_overlap(query_range) {
                        //} && item <= query_to {
                        println!("case c - subtree may (at least partly) in range");
                        let child: RangedNode<'a, M> = child.into();
                        let child_result = child.query_range_inner_next_attempt(query_range);
                        result = result.combine(&child_result);
                    } else {
                        println!("case d - not in range");
                    }
                }

                if query_range.has_overlap(self.last_child().range()) {
                    let child: RangedNode<M> = self.last_child().into();
                    let child_result = child.query_range_inner_next_attempt(query_range);
                    result = result.combine(&child_result);
                }

                println!("returning (non-wrapping) {result:?}");

                result
            } else {
                // we have a wrapping query

                let mut before_wrap = M::neutral();
                let mut after_wrap = M::neutral();

                for (child, ref item) in self.children() {
                    let child: RangedNode<M> = child.into();
                    println!("iter:   N:{:?}/Q:{:?} - child:", self.range(), query_range);
                    println!("          node:{:?}", child.node());
                    println!("          range:{:?}", child.range());
                    println!("          total:{:?}", child.node().monoid());
                    println!("          item:{item:?}");
                    println!("          before:{before_wrap:?} after:{after_wrap:?}");

                    // q1: do we need to add the subtree to before wrap?
                    // q2: do we need to add the item to before wrap?
                    if child.range().to() >= query_from || child.range().from() <= query_to {
                        // TODO replace the below
                    }

                    if child.range().to() >= query_from {
                        println!("  in the before_wrap branch");
                        if item > query_from {
                            println!("  recursing...");
                            let next_query_range =
                                NewRange(query_from.clone(), child.range().to().clone());
                            let child_result =
                                child.query_range_inner_next_attempt(&next_query_range);
                            before_wrap = before_wrap.combine(&child_result);
                        }

                        // item is >= child.range.to, so it's defintely >= query_from so it is in
                        // query_range.
                        before_wrap = before_wrap.combine(&M::lift(item));
                    }

                    // q3: do we need to add the subtree to after wrap?
                    // q4: do we need to add the item to after wrap?
                    if child.range().from() <= query_to {
                        println!("in the after_wrap branch");

                        // this is an ugly condition. Maybe there is a better way to deal with
                        // this, but I haven't found it.
                        // The problem is that we start with a query range and then keep cutting
                        // it down left and right. At some point we have start == end, which means
                        // it should be then empty query. But the semantics of the query make it
                        // mean it's the set of all items, which results in over-adding. To avoid
                        // this we check whether this is what would be happening here, and if it
                        // happens, we just skip it.
                        if child.range().from() != query_to {
                            let next_query_range =
                                NewRange(child.range().from().clone(), query_to.clone());
                            let child_result =
                                child.query_range_inner_next_attempt(&next_query_range);
                            after_wrap = after_wrap.combine(&child_result);
                        }

                        if item < query_to {
                            println!("  adding item...");
                            after_wrap = after_wrap.combine(&M::lift(item));
                        }
                    }
                }

                // this always has to be queried
                let last_child = self.last_child();
                let last_child: RangedNode<M> = last_child.into();
                let last_child_result = last_child.query_range_inner_next_attempt(query_range);
                before_wrap = before_wrap.combine(&last_child_result);

                let result = before_wrap.combine(&after_wrap);

                println!("returning (wrapping) {result:?}");

                result
            }
        }
    }
}

struct QueryResult<M: ProtocolMonoid>(Vec<(Range<M::Item>, M)>);

impl<M: ProtocolMonoid> QueryResult<M> {
    fn can_absorb(&self, next_monoid: &M) -> bool {
        let next_count = next_monoid.count();
        let fits_directly = match self.0.last() {
            Some((_, last)) => {
                let last_count = last.count();
                next_count.is_power_of_two() && next_count <= last_count
            }
            None => next_count.is_power_of_two(),
        };

        let mut sum_count = next_count;
        let mut fits_by_combining = false;

        for (_, cur) in self.0.iter().rev() {
            let cur_count = cur.count();
            sum_count += cur_count;
            if sum_count.is_power_of_two() && next_count <= cur_count {
                fits_by_combining = true;
                break;
            }
        }

        fits_directly || fits_by_combining
    }

    fn absorb(&mut self, next_range: &Range<M::Item>, next_monoid: &M) {
        assert!(self.can_absorb(next_monoid));

        self.0.push((next_range.clone(), next_monoid.clone()));
        self.organize();
    }

    fn organize(&mut self) {
        let QueryResult(elems) = self;

        if elems.len() < 2 {
            return;
        }

        if elems.len() < 3 {
            if elems[0].1.count() >= elems[1].1.count() {
                return;
            } else {
                // okay what I have to do here is to combine the two ranges
                // this is a terrible pain with the enum
                // Maybe this is where I port everything to the aljoscha-ranges
                // -> do that now!
            }
        }

        for (i, _) in self.0.iter().enumerate().skip(1).rev() {
            if i == 1 {
                // we want these to
            } else {
            }
            //if elems[i] > elems[i+1] {}
        }
    }
}

impl Rangable for u64 {}

#[cfg(test)]
mod test {
    use super::Range;
    use crate::{
        monoid::sum::SumMonoid, range::NewRange, ranged_node::RangedNode, LiftingMonoid, Node,
    };

    use std::collections::HashSet;

    use proptest::{prelude::prop, proptest};

    proptest! {
        #[test]
        fn new_range_query_correctness(items in prop::collection::vec(1..1000u64, 1..100usize), from in 0..1000u64, to in 0..1000u64) {
            println!();
            let item_set: HashSet<u64> = HashSet::from_iter(items.iter().cloned());
            println!("items used in test: {:?}", item_set);

            let mut root = Node::<SumMonoid>::Nil(SumMonoid::lift(&0));

            for item in &item_set {
                root = root.insert(*item);
            }
            println!("in tree form: {:}", root);


            let min = items.iter().fold(10000, |acc, x| u64::min(*x, acc));
            let max = items.iter().fold(0, |acc, x| u64::max(*x, acc));

            let ranged_root = RangedNode::new(&root, NewRange(min, max+1));
            let query_result = ranged_root.query_range_monoid_new(&NewRange(from, to));

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

    fn repro_1() {
        let items = vec![196, 197, 198];
        let from = 196;
        let to = 195;

        let mut root = Node::<SumMonoid>::Nil(SumMonoid::lift(&0));
        for item in &items {
            root = root.insert(*item);
        }

        println!("items used in test: {:?}", items);
        println!("from:{from} to:{to}");
        println!("in tree form: {:}", root);

        let min = items.iter().fold(10000, |acc, x| u64::min(*x, acc));
        let max = items.iter().fold(0, |acc, x| u64::max(*x, acc));

        let ranged_root = RangedNode::new(&root, NewRange(min - 1, max + 1));
        let query_result = ranged_root.query_range_monoid_new(&NewRange(from, to));

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

        let mut root = Node::<SumMonoid>::Nil(SumMonoid::lift(&0));
        for item in &items {
            root = root.insert(*item);
        }

        println!("items used in test: {:?}", items);
        println!("from:{from} to:{to}");
        println!("in tree form: {:}", root);

        let min = items.iter().fold(10000, |acc, x| u64::min(*x, acc));
        let max = items.iter().fold(0, |acc, x| u64::max(*x, acc));

        let ranged_root = RangedNode::new(&root, NewRange(min - 1, max + 1));
        let query_result = ranged_root.query_range_monoid_new(&NewRange(from, to));

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
    fn range_querys_are_correct_for_small_node2_tree() {
        let mut root = Node::<SumMonoid>::Nil(SumMonoid::lift(&0));
        root = root.insert(30);
        root = root.insert(60);
        root = root.insert(50);

        let SumMonoid(res) = root.query_range_monoid(&Range::Full);
        assert_eq!(res, 140);

        let item_set: HashSet<u64>;

        let SumMonoid(res) = root.query_range_monoid(&Range::StartingFrom(25));
        assert_eq!(res, 140);
        let SumMonoid(res) = root.query_range_monoid(&Range::StartingFrom(30));
        assert_eq!(res, 140);
        let SumMonoid(res) = root.query_range_monoid(&Range::StartingFrom(40));
        assert_eq!(res, 110);
        let SumMonoid(res) = root.query_range_monoid(&Range::StartingFrom(50));
        assert_eq!(res, 110);
        let SumMonoid(res) = root.query_range_monoid(&Range::StartingFrom(60));
        assert_eq!(res, 60);
        let SumMonoid(res) = root.query_range_monoid(&Range::StartingFrom(55));
        assert_eq!(res, 60);
        let SumMonoid(res) = root.query_range_monoid(&Range::StartingFrom(70));
        assert_eq!(res, 0);

        let SumMonoid(res) = root.query_range_monoid(&Range::UpTo(25));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::UpTo(30));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::UpTo(55));
        assert_eq!(res, 80);
        let SumMonoid(res) = root.query_range_monoid(&Range::UpTo(60));
        assert_eq!(res, 80);
        let SumMonoid(res) = root.query_range_monoid(&Range::UpTo(61));
        assert_eq!(res, 140);

        let SumMonoid(res) = root.query_range_monoid(&Range::Between(10, 20));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(10, 30));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(10, 40));
        assert_eq!(res, 30);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(10, 50));
        assert_eq!(res, 30);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(10, 55));
        assert_eq!(res, 80);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(10, 60));
        assert_eq!(res, 80);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(10, 61));
        assert_eq!(res, 140);

        let SumMonoid(res) = root.query_range_monoid(&Range::Between(30, 30));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(30, 40));
        assert_eq!(res, 30);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(30, 50));
        assert_eq!(res, 30);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(30, 55));
        assert_eq!(res, 80);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(30, 60));
        assert_eq!(res, 80);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(30, 61));
        assert_eq!(res, 140);

        let SumMonoid(res) = root.query_range_monoid(&Range::Between(40, 40));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(40, 50));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(40, 55));
        assert_eq!(res, 50);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(40, 60));
        assert_eq!(res, 50);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(40, 61));
        assert_eq!(res, 110);

        let SumMonoid(res) = root.query_range_monoid(&Range::Between(50, 50));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(50, 55));
        assert_eq!(res, 50);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(50, 60));
        assert_eq!(res, 50);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(50, 61));
        assert_eq!(res, 110);

        let SumMonoid(res) = root.query_range_monoid(&Range::Between(55, 55));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(55, 60));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(55, 61));
        assert_eq!(res, 60);

        let SumMonoid(res) = root.query_range_monoid(&Range::Between(60, 60));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(60, 61));
        assert_eq!(res, 60);

        let SumMonoid(res) = root.query_range_monoid(&Range::Between(61, 70));
        assert_eq!(res, 0);
    }

    #[test]
    fn range_querys_are_correct_for_small_node3_tree() {
        let mut root = Node::<SumMonoid>::Nil(SumMonoid::lift(&0));
        root = root.insert(2);
        root = root.insert(4);
        root = root.insert(8);
        root = root.insert(16);
        root = root.insert(32);

        let SumMonoid(res) = root.query_range_monoid(&Range::Full);
        assert_eq!(res, 62);

        let SumMonoid(res) = root.query_range_monoid(&Range::StartingFrom(0));
        assert_eq!(res, 62);
        let SumMonoid(res) = root.query_range_monoid(&Range::StartingFrom(2));
        assert_eq!(res, 62);
        let SumMonoid(res) = root.query_range_monoid(&Range::StartingFrom(3));
        assert_eq!(res, 60);
        let SumMonoid(res) = root.query_range_monoid(&Range::StartingFrom(4));
        assert_eq!(res, 60);
        let SumMonoid(res) = root.query_range_monoid(&Range::StartingFrom(6));
        assert_eq!(res, 56);
        let SumMonoid(res) = root.query_range_monoid(&Range::StartingFrom(8));
        assert_eq!(res, 56);
        let SumMonoid(res) = root.query_range_monoid(&Range::StartingFrom(12));
        assert_eq!(res, 48);
        let SumMonoid(res) = root.query_range_monoid(&Range::StartingFrom(16));
        assert_eq!(res, 48);
        let SumMonoid(res) = root.query_range_monoid(&Range::StartingFrom(24));
        assert_eq!(res, 32);
        let SumMonoid(res) = root.query_range_monoid(&Range::StartingFrom(32));
        assert_eq!(res, 32);
        let SumMonoid(res) = root.query_range_monoid(&Range::StartingFrom(33));
        assert_eq!(res, 0);

        let SumMonoid(res) = root.query_range_monoid(&Range::UpTo(0));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::UpTo(2));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::UpTo(3));
        assert_eq!(res, 2);
        let SumMonoid(res) = root.query_range_monoid(&Range::UpTo(4));
        assert_eq!(res, 2);
        let SumMonoid(res) = root.query_range_monoid(&Range::UpTo(6));
        assert_eq!(res, 6);
        let SumMonoid(res) = root.query_range_monoid(&Range::UpTo(8));
        assert_eq!(res, 6);
        let SumMonoid(res) = root.query_range_monoid(&Range::UpTo(12));
        assert_eq!(res, 14);
        let SumMonoid(res) = root.query_range_monoid(&Range::UpTo(16));
        assert_eq!(res, 14);
        let SumMonoid(res) = root.query_range_monoid(&Range::UpTo(24));
        assert_eq!(res, 30);
        let SumMonoid(res) = root.query_range_monoid(&Range::UpTo(32));
        assert_eq!(res, 30);
        let SumMonoid(res) = root.query_range_monoid(&Range::UpTo(33));
        assert_eq!(res, 62);

        let SumMonoid(res) = root.query_range_monoid(&Range::Between(0, 1));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(0, 2));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(0, 3));
        assert_eq!(res, 2);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(0, 4));
        assert_eq!(res, 2);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(0, 6));
        assert_eq!(res, 6);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(0, 8));
        assert_eq!(res, 6);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(0, 12));
        assert_eq!(res, 14);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(0, 16));
        assert_eq!(res, 14);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(0, 24));
        assert_eq!(res, 30);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(0, 32));
        assert_eq!(res, 30);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(0, 33));
        assert_eq!(res, 62);

        let SumMonoid(res) = root.query_range_monoid(&Range::Between(2, 2));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(2, 3));
        assert_eq!(res, 2);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(2, 4));
        assert_eq!(res, 2);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(2, 6));
        assert_eq!(res, 6);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(2, 8));
        assert_eq!(res, 6);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(2, 12));
        assert_eq!(res, 14);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(2, 16));
        assert_eq!(res, 14);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(2, 24));
        assert_eq!(res, 30);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(2, 32));
        assert_eq!(res, 30);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(2, 33));
        assert_eq!(res, 62);

        let SumMonoid(res) = root.query_range_monoid(&Range::Between(3, 4));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(3, 6));
        assert_eq!(res, 4);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(3, 8));
        assert_eq!(res, 4);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(3, 12));
        assert_eq!(res, 12);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(3, 16));
        assert_eq!(res, 12);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(3, 24));
        assert_eq!(res, 28);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(3, 32));
        assert_eq!(res, 28);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(3, 33));
        assert_eq!(res, 60);

        let SumMonoid(res) = root.query_range_monoid(&Range::Between(4, 4));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(4, 6));
        assert_eq!(res, 4);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(4, 8));
        assert_eq!(res, 4);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(4, 12));
        assert_eq!(res, 12);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(4, 16));
        assert_eq!(res, 12);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(4, 24));
        assert_eq!(res, 28);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(4, 32));
        assert_eq!(res, 28);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(4, 33));
        assert_eq!(res, 60);

        let SumMonoid(res) = root.query_range_monoid(&Range::Between(6, 8));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(6, 12));
        assert_eq!(res, 8);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(6, 16));
        assert_eq!(res, 8);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(6, 24));
        assert_eq!(res, 24);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(6, 32));
        assert_eq!(res, 24);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(6, 33));
        assert_eq!(res, 56);

        let SumMonoid(res) = root.query_range_monoid(&Range::Between(8, 8));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(8, 12));
        assert_eq!(res, 8);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(8, 16));
        assert_eq!(res, 8);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(8, 24));
        assert_eq!(res, 24);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(8, 32));
        assert_eq!(res, 24);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(8, 33));
        assert_eq!(res, 56);

        let SumMonoid(res) = root.query_range_monoid(&Range::Between(12, 16));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(12, 24));
        assert_eq!(res, 16);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(12, 32));
        assert_eq!(res, 16);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(12, 33));
        assert_eq!(res, 48);

        let SumMonoid(res) = root.query_range_monoid(&Range::Between(16, 16));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(16, 24));
        assert_eq!(res, 16);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(16, 32));
        assert_eq!(res, 16);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(16, 33));
        assert_eq!(res, 48);

        let SumMonoid(res) = root.query_range_monoid(&Range::Between(24, 32));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(&Range::Between(24, 33));
        assert_eq!(res, 32);

        let SumMonoid(res) = root.query_range_monoid(&Range::Between(33, 34));
        assert_eq!(res, 0);
    }
}
