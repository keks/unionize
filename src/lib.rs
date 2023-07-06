use monoid::Peano;
use query::Accumulator;
use range::Range;

pub mod proto;
pub mod range;

pub mod hash_item;
pub mod monoid;
pub mod query;
pub mod tree;

pub trait Node<'a, M>: std::fmt::Debug + Clone
where
    M: monoid::Monoid + 'a,
    Self: 'a,
{
    type ChildIter: Iterator<Item = (&'a Self, &'a M::Item)>;

    fn monoid(&self) -> &M;
    fn is_nil(&self) -> bool;

    fn min_item(&self) -> Option<&M::Item>;
    fn max_item(&self) -> Option<&M::Item>;

    fn children(&'a self) -> Option<Self::ChildIter>;
    fn last_child(&self) -> Option<&Self>;

    fn query<A: Accumulator<M>>(&'a self, query_range: &Range<M::Item>, state: &mut A) {
        if self.is_nil() {
            return;
        }

        let min = self
            .min_item()
            .expect("only nil nodes return None here, and we know it's not nil");
        let max = self
            .max_item()
            .expect("only nil nodes return None here, and we know it's not nil");

        let has_overlap = query_range.has_overlap(self);
        let is_subrange = query_range.fully_contains(self);

        if !(has_overlap) {
            return;
        }

        if query_range.from() == query_range.to() {
            // querying full range, node is completely in range,
            // but start at the boundary item, wrap around, and then end at the boundary item.

            // first add items and children after the boundary
            if let Some(new_query_range) = query_range.cap_right(min.clone()) {
                self.query(&new_query_range, state);
            }

            // then add items and children before the boundary
            if let Some(new_query_range) = query_range.cap_left(min.clone()) {
                self.query(&new_query_range, state);
            }

            return;
        } else if query_range.from() < query_range.to() {
            if is_subrange {
                state.add_xnode(self);
                return;
            }
            // this is a non-wrapping query
            for (child, item) in self.children().unwrap() {
                child.query(query_range, state);
                if query_range.contains(item) {
                    state.add_item(&item);
                }
            }

            self.last_child().unwrap().query(query_range, state);
        } else {
            if is_subrange {
                state.add_xnode(self);
                return;
            }
            // we have a wrapping query

            for (child, item) in self.children().unwrap() {
                if query_range.from() <= item {
                    if let Some(next_query_range) = query_range.cap_right(item.clone()) {
                        child.query(&next_query_range, state);
                    }

                    // so it's >= query_range.from,
                    // so it's in query_range.
                    state.add_item(&item);
                }
            }

            // query the last subtree for elements before the wrap.
            // and we have to restrict it to not include stuff from after the wrap
            {
                let last_child = self.last_child().unwrap();
                if query_range.from() <= &max {
                    let next_query_range = query_range
                        .cap_right(max.next())
                        .expect("guaranteed since max.next() > query_range.from()");
                    last_child.query(&next_query_range, state);
                }
            }

            for (child, item) in self.children().unwrap() {
                if let Some(child_min) = child.min_item() {
                    if child_min <= query_range.to() {
                        if let Some(next_query_range) = query_range.cap_left(child_min.clone()) {
                            child.query(&next_query_range, state);
                        }
                    }
                }

                if item < query_range.to() {
                    state.add_item(&item);
                }
            }

            // The last child may also contain nodes from after wrapping, but that is only the
            // case if last_child.range.from < query_range.to().
            let last_child = self.last_child().unwrap();
            if let Some(last_child_min) = last_child.min_item() {
                if last_child_min < query_range.to() {
                    let next_query_range = query_range
                        .cap_left(last_child_min.clone())
                        .expect("guaranteed since min_item < query_range.to()");
                    last_child.query(&next_query_range, state);
                }
            }
        }
    }
}
