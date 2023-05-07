use crate::{
    proto::ProtocolMonoid,
    range::{Rangable, Range},
    ranged_node::RangedNode,
    LiftingMonoid, Node,
};
use std::fmt::Debug;

impl<'a, M> RangedNode<'a, M>
where
    M: LiftingMonoid,
    M::Item: Debug + Rangable,
{
    pub fn query_range(&self, query_range: &Range<M::Item>) -> M {
        // println!("enter:  N:{:?}/Q:{:?}", self.range(), query_range);

        if !self.range().has_overlap(query_range) {
            // println!(
            //     "return: N:{:?}/Q:{:?} - zero because no overlap",
            //     self.range(),
            //     query_range
            // );
            return M::neutral();
        }

        let node: &Node<M> = self.node();

        if node.is_nil() {
            // println!(
            //     "return: N:{:?}/Q:{:?} - zero because node is nil",
            //     self.range(),
            //     query_range
            // );
            return M::neutral();
        }

        if self.range().is_subrange_of(query_range) {
            return self.node().monoid().clone();
        }

        let Range(query_from, query_to) = query_range;

        if query_from == query_to {
            // println!(
            //     "return: N:{:?}/Q:{:?} - query is full, so returning monoid",
            //     self.range(),
            //     query_range
            // );
            return node.monoid().clone();
        } else if query_from < query_to {
            // this is a non-wrapping query

            let mut result = M::neutral();

            for (child, ref item) in self.children() {
                // println!(
                //     "(child:{:?}:{:?}) - item:{:?}..",
                //     child.range(),
                //     child.node().monoid(),
                //     item
                // );
                if query_from < item && item < query_to {
                    // println!("case a - both subtree (at least partly) and item in range");
                    let child: RangedNode<'a, M> = child.into();
                    let child_result = child.query_range(query_range);
                    result = result.combine(&child_result);
                    result = result.combine(&M::lift(item));
                } else if query_from == item && item < query_to {
                    //println!("case b - item in range");
                    result = result.combine(&M::lift(item));
                } else if child.range().has_overlap(query_range) {
                    //} && item <= query_to {
                    // println!("case c - subtree may (at least partly) in range");
                    let child: RangedNode<'a, M> = child.into();
                    let child_result = child.query_range(query_range);
                    result = result.combine(&child_result);
                } else {
                    //println!("case d - not in range");
                }
            }

            if query_range.has_overlap(self.last_child().range()) {
                let child: RangedNode<M> = self.last_child().into();
                let child_result = child.query_range(query_range);
                result = result.combine(&child_result);
            }

            // println!("returning (non-wrapping) {result:?}");

            result
        } else {
            // we have a wrapping query

            let mut before_wrap = M::neutral();
            let mut after_wrap = M::neutral();

            for (child, ref item) in self.children() {
                let child: RangedNode<M> = child.into();
                // println!("iter:   N:{:?}/Q:{:?} - child:", self.range(), query_range);
                // println!("          node:{:?}", child.node());
                // println!("          range:{:?}", child.range());
                // println!("          total:{:?}", child.node().monoid());
                // println!("          item:{item:?}");
                // println!("          before:{before_wrap:?} after:{after_wrap:?}");

                // q1: do we need to add the subtree to before wrap?
                // q2: do we need to add the item to before wrap?
                if child.range().to() >= query_from || child.range().from() <= query_to {
                    // TODO replace the below
                }

                if child.range().to() >= query_from {
                    // println!("  in the before_wrap branch");
                    if item > query_from {
                        // println!("  recursing...");
                        let next_query_range =
                            Range(query_from.clone(), child.range().to().clone());
                        let child_result = child.query_range(&next_query_range);
                        before_wrap = before_wrap.combine(&child_result);
                    }

                    // item is >= child.range.to, so it's defintely >= query_from so it is in
                    // query_range.
                    before_wrap = before_wrap.combine(&M::lift(item));
                }

                // q3: do we need to add the subtree to after wrap?
                // q4: do we need to add the item to after wrap?
                if child.range().from() <= query_to {
                    // println!("in the after_wrap branch");

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
                            Range(child.range().from().clone(), query_to.clone());
                        let child_result = child.query_range(&next_query_range);
                        after_wrap = after_wrap.combine(&child_result);
                    }

                    if item < query_to {
                        // println!("  adding item...");
                        after_wrap = after_wrap.combine(&M::lift(item));
                    }
                }
            }

            // this always has to be queried
            let last_child = self.last_child();
            let last_child: RangedNode<M> = last_child.into();
            let last_child_result = last_child.query_range(query_range);
            before_wrap = before_wrap.combine(&last_child_result);

            let result = before_wrap.combine(&after_wrap);

            // println!("returning (wrapping) {result:?}");

            result
        }
    }
}

#[derive(Debug, Clone)]
pub struct SplitAccumulator<'a, M>
where
    M: ProtocolMonoid,
    M::Item: Rangable,
{
    split_sizes: &'a [usize],
    results: Vec<M>,
    ranges: Vec<Range<M::Item>>,
    current_offset: usize,
    update_ranges: bool,
}

impl<'a, M> SplitAccumulator<'a, M>
where
    M: ProtocolMonoid,
    M::Item: Rangable,
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
    M::Item: Rangable,
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

#[derive(Debug, Clone)]
pub struct ItemsAccumulator<M: LiftingMonoid> {
    items: Vec<M::Item>,
}

impl<M: LiftingMonoid> ItemsAccumulator<M> {
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
    M: LiftingMonoid + std::fmt::Debug,
    M::Item: Rangable,
{
    fn add_node(&mut self, node: &RangedNode<M>) {
        let is_leaf = node.node().is_leaf();
        for (child, item) in node.children() {
            if !is_leaf {
                self.add_node(&child.ranged_node());
            }
            self.items.push(item);
        }

        if !is_leaf {
            self.add_node(&node.last_child().ranged_node())
        }
    }

    fn add_item(&mut self, item: &<M as LiftingMonoid>::Item) {
        self.items.push(item.clone())
    }
}

impl<'a, M> RangedNode<'a, M>
where
    M: ProtocolMonoid,
    M::Item: Debug + Rangable,
{
    pub fn query_range_split(&self, query_range: &Range<M::Item>, split_sizes: &[usize]) -> Vec<M> {
        let mut state = SplitAccumulator::new(query_range, split_sizes);
        self.query_range_split_inner(query_range, &mut state);

        state.results
    }

    fn query_range_split_inner(
        &self,
        query_range: &Range<M::Item>,
        state: &mut SplitAccumulator<'a, M>,
    ) {
        println!("enter:  N:{:?}/Q:{:?}", self.range(), query_range);

        if !self.range().has_overlap(query_range) {
            println!(
                "return: N:{:?}/Q:{:?} - zero because no overlap",
                self.range(),
                query_range
            );
            return;
        }

        let node: &Node<M> = self.node();

        if node.is_nil() {
            println!(
                "return: N:{:?}/Q:{:?} - zero because node is nil",
                self.range(),
                query_range
            );
            return;
        }

        if self.range().is_subrange_of(query_range) {
            state.add_node(&self);
            return;
        }

        let Range(query_from, query_to) = query_range;

        if query_from == query_to {
            println!(
                "return: N:{:?}/Q:{:?} - query is full, so returning monoid",
                self.range(),
                query_range
            );
            state.add_node(&self);
            return;
        } else if query_from < query_to {
            // this is a non-wrapping query

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
                    child.query_range_split_inner(query_range, state);
                    state.add_item(&item);
                } else if query_from == item && item < query_to {
                    println!("case b - item in range");
                    state.add_item(&item);
                } else if child.range().has_overlap(query_range) {
                    println!("case c - subtree may (at least partly) in range");
                    let child: RangedNode<'a, M> = child.into();
                    child.query_range_split_inner(query_range, state);
                } else {
                    println!("case d - not in range");
                }
            }

            if query_range.has_overlap(self.last_child().range()) {
                let child: RangedNode<M> = self.last_child().into();
                child.query_range_split_inner(query_range, state);
            }
        } else {
            // we have a wrapping query

            for (child, ref item) in self.children() {
                let child: RangedNode<M> = child.into();
                println!(
                    "iter:   N:{:?}/Q:{:?} - before wrap. child:",
                    self.range(),
                    query_range
                );
                println!("          node:{:?}", child.node());
                println!("          range:{:?}", child.range());
                println!("          total:{:?}", child.node().monoid());
                println!("          item:{item:?}");
                println!("          state:{state:#?}");

                // q1: do we need to add the subtree to before wrap?
                // q2: do we need to add the item to before wrap?
                if child.range().to() >= query_from {
                    if item > query_from {
                        println!("  recursing...");
                        let next_query_range =
                            Range(query_from.clone(), child.range().to().clone());
                        child.query_range_split_inner(&next_query_range, state);
                    }

                    // item is >= child.range.to, so it's defintely >= query_from so it is in
                    // query_range.
                    println!("  adding...");
                    state.add_item(&item);
                }
            }

            // query the last subtree for elements before the wrap.
            // and we have to restrict it to not include stuff from after the wrap
            {
                let last_child = self.last_child();
                let last_child: RangedNode<M> = last_child.into();
                if last_child.range().to() > query_from {
                    let next_query_range =
                        Range(query_from.clone(), last_child.range().to().clone());
                    last_child.query_range_split_inner(&next_query_range, state);
                }
            }

            for (child, ref item) in self.children() {
                let child: RangedNode<M> = child.into();
                println!(
                    "iter:   N:{:?}/Q:{:?} - after wrap. child:",
                    self.range(),
                    query_range
                );
                println!("          node:{:?}", child.node());
                println!("          range:{:?}", child.range());
                println!("          total:{:?}", child.node().monoid());
                println!("          item:{item:?}");
                println!("          state:{state:#?}");

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
                        println!("  recursing...");
                        let next_query_range =
                            Range(child.range().from().clone(), query_to.clone());
                        child.query_range_split_inner(&next_query_range, state);
                    }

                    if item < query_to {
                        println!("  adding...");
                        state.add_item(&item);
                    }
                }
            }

            // the last subtree has to be queried unconditionally,
            // and we have to restrict it to not include stuff from before the wrap
            // The last child may also contain nodes from after wrapping, but that is only the
            // case if last_child.range.from < query_to.
            //
            let last_child = self.last_child();
            let last_child: RangedNode<M> = last_child.into();
            if last_child.range().from() < query_to {
                {
                    let next_query_range =
                        Range(last_child.range().from().clone(), query_to.clone());
                    last_child.query_range_split_inner(&next_query_range, state);
                }
            }
        }
    }
}

impl Rangable for u64 {}

pub trait Accumulator<M>: std::fmt::Debug
where
    M: LiftingMonoid,
    M::Item: Rangable,
{
    fn add_node(&mut self, node: &RangedNode<M>);
    fn add_item(&mut self, node: &M::Item);
}

impl<'a, M> RangedNode<'a, M>
where
    M: ProtocolMonoid,
    M::Item: Debug + Rangable,
{
    pub fn query_range_generic<A: Accumulator<M>>(
        &self,
        query_range: &Range<M::Item>,
        state: &mut A,
    ) {
        //println!("enter:  N:{:?}/Q:{:?}", self.range(), query_range);

        if !self.range().has_overlap(query_range) {
            // println!(
            //     "return: N:{:?}/Q:{:?} - zero because no overlap",
            //     self.range(),
            //     query_range
            // );
            return;
        }

        let node: &Node<M> = self.node();

        if node.is_nil() {
            // println!(
            //     "return: N:{:?}/Q:{:?} - zero because node is nil",
            //     self.range(),
            //     query_range
            // );
            return;
        }

        if self.range().is_subrange_of(query_range) {
            state.add_node(&self);
            return;
        }

        let Range(query_from, query_to) = query_range;

        if query_from == query_to {
            // println!(
            //     "return: N:{:?}/Q:{:?} - query is full, so returning monoid",
            //     self.range(),
            //     query_range
            // );
            state.add_node(&self);
            return;
        } else if query_from < query_to {
            // this is a non-wrapping query

            for (child, ref item) in self.children() {
                // println!(
                //     "(child:{:?}:{:?}) - item:{:?}..",
                //     child.range(),
                //     child.node().monoid(),
                //     item
                // );

                if query_from < item && item < query_to {
                    // println!("case a - both subtree (at least partly) and item in range");
                    let child: RangedNode<'a, M> = child.into();
                    child.query_range_generic(query_range, state);
                    state.add_item(&item);
                } else if query_from == item && item < query_to {
                    //println!("case b - item in range");
                    state.add_item(&item);
                } else if child.range().has_overlap(query_range) {
                    //println!("case c - subtree may (at least partly) in range");
                    let child: RangedNode<'a, M> = child.into();
                    child.query_range_generic(query_range, state);
                } else {
                    //println!("case d - not in range");
                }
            }

            if query_range.has_overlap(self.last_child().range()) {
                let child: RangedNode<M> = self.last_child().into();
                child.query_range_generic(query_range, state);
            }
        } else {
            // we have a wrapping query

            for (child, ref item) in self.children() {
                let child: RangedNode<M> = child.into();
                // println!(
                //     "iter:   N:{:?}/Q:{:?} - before wrap. child:",
                //     self.range(),
                //     query_range
                // );
                // println!("          node:{:?}", child.node());
                // println!("          range:{:?}", child.range());
                // println!("          total:{:?}", child.node().monoid());
                // println!("          item:{item:?}");
                // println!("          state:{state:#?}");

                // q1: do we need to add the subtree to before wrap?
                // q2: do we need to add the item to before wrap?
                if child.range().to() >= query_from {
                    if item > query_from {
                        //println!("  recursing...");
                        let next_query_range =
                            Range(query_from.clone(), child.range().to().clone());
                        child.query_range_generic(&next_query_range, state);
                    }

                    // item is >= child.range.to, so it's defintely >= query_from so it is in
                    // query_range.
                    //println!("  adding...");
                    state.add_item(&item);
                }
            }

            // query the last subtree for elements before the wrap.
            // and we have to restrict it to not include stuff from after the wrap
            {
                let last_child = self.last_child();
                let last_child: RangedNode<M> = last_child.into();
                if last_child.range().to() > query_from {
                    let next_query_range =
                        Range(query_from.clone(), last_child.range().to().clone());
                    last_child.query_range_generic(&next_query_range, state);
                }
            }

            for (child, ref item) in self.children() {
                let child: RangedNode<M> = child.into();
                // println!(
                //     "iter:   N:{:?}/Q:{:?} - after wrap. child:",
                //     self.range(),
                //     query_range
                // );
                // println!("          node:{:?}", child.node());
                // println!("          range:{:?}", child.range());
                // println!("          total:{:?}", child.node().monoid());
                // println!("          item:{item:?}");
                // println!("          state:{state:#?}");

                // q3: do we need to add the subtree to after wrap?
                // q4: do we need to add the item to after wrap?
                if child.range().from() <= query_to {
                    // println!("in the after_wrap branch");

                    // this is an ugly condition. Maybe there is a better way to deal with
                    // this, but I haven't found it.
                    // The problem is that we start with a query range and then keep cutting
                    // it down left and right. At some point we have start == end, which means
                    // it should be then empty query. But the semantics of the query make it
                    // mean it's the set of all items, which results in over-adding. To avoid
                    // this we check whether this is what would be happening here, and if it
                    // happens, we just skip it.
                    if child.range().from() != query_to {
                        //println!("  recursing...");
                        let next_query_range =
                            Range(child.range().from().clone(), query_to.clone());
                        child.query_range_generic(&next_query_range, state);
                    }

                    if item < query_to {
                        //println!("  adding...");
                        state.add_item(&item);
                    }
                }
            }

            // the last subtree has to be queried unconditionally,
            // and we have to restrict it to not include stuff from before the wrap
            // The last child may also contain nodes from after wrapping, but that is only the
            // case if last_child.range.from < query_to.
            //
            let last_child = self.last_child();
            let last_child: RangedNode<M> = last_child.into();
            if last_child.range().from() < query_to {
                {
                    let next_query_range =
                        Range(last_child.range().from().clone(), query_to.clone());
                    last_child.query_range_generic(&next_query_range, state);
                }
            }
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::{
        monoid::count::CountingMonoid,
        monoid::{
            sum::{SumItem, SumMonoid},
            FormattingMonoid,
        },
        proto::ProtocolMonoid,
        query::{ItemsAccumulator, SplitAccumulator},
        range::{Rangable, Range},
        ranged_node::RangedNode,
        LiftingMonoid, Node,
    };
    use proptest::{prelude::prop, proptest};
    use std::collections::HashSet;

    #[derive(Clone, PartialEq, Eq)]
    pub struct TestMonoid<T>(CountingMonoid<T>, SumMonoid<T>)
    where
        T: Rangable + Clone + SumItem;

    impl<T> TestMonoid<T>
    where
        T: Rangable + Clone + SumItem + std::fmt::Debug,
    {
        fn sum(&self) -> &T {
            let TestMonoid(_, sum_monoid) = self;
            sum_monoid.sum()
        }
    }

    impl<T> std::fmt::Debug for TestMonoid<T>
    where
        T: Rangable + Clone + std::fmt::Display + SumItem,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let count = self.count();
            let sum = self.sum();
            write!(f, "T(C:{count} S:{sum})")
        }
    }

    impl<T> LiftingMonoid for TestMonoid<T>
    where
        T: Rangable + Clone + std::fmt::Display + SumItem,
    {
        type Item = T;

        fn neutral() -> Self {
            Self(CountingMonoid::neutral(), SumMonoid::neutral())
        }

        fn lift(item: &Self::Item) -> Self {
            Self(CountingMonoid::lift(item), SumMonoid::lift(item))
        }

        fn combine(&self, other: &Self) -> Self {
            let Self(other_count, other_sum) = other;
            let Self(self_count, self_sum) = self;
            Self(self_count.combine(other_count), self_sum.combine(other_sum))
        }
    }

    impl<T> FormattingMonoid for TestMonoid<T>
    where
        T: Rangable + Clone + std::fmt::Display + SumItem,
    {
        fn item_to_string(item: &Self::Item) -> String {
            format!("{item}")
        }
    }

    impl<T> ProtocolMonoid for TestMonoid<T>
    where
        T: Rangable + Clone + std::fmt::Display + SumItem,
    {
        fn count(&self) -> usize {
            let TestMonoid(counting_monoid, _) = self;
            counting_monoid.count()
        }
    }

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
                items_sorted
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
                items_sorted.clone()
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
