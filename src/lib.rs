use monoid::Peano;
use query::Accumulator;
use range::Range;

pub mod proto;
pub mod range;

pub mod hash_item;
pub mod monoid;
pub mod query;
pub mod tree;

pub trait NonNilNode<'a, M, N>: std::fmt::Debug + Clone
where
    M: monoid::Monoid,
    N: Node<M>,
{
    type ChildIter<'b>: Iterator<Item = (&'b N, &'b M::Item)>
    where
        N: 'b,
        M::Item: 'b,
        Self: 'b;

    fn bounds(&self) -> (&M::Item, &M::Item);
    fn min(&self) -> &M::Item;
    fn max(&self) -> &M::Item;
    fn children<'b>(&'b self) -> Self::ChildIter<'b>;
    fn last_child<'b>(&'b self) -> &'b N;
}

pub trait Node<M>: std::fmt::Debug + Clone
where
    M: monoid::Monoid,
{
    // type ChildIter: Iterator<Item = (&'a Self, &'a M::Item)>;
    type NonNilNode<'a>: NonNilNode<'a, M, Self>
    where
        Self: 'a;

    fn monoid(&self) -> &M;
    fn is_nil(&self) -> bool;

    fn bounds(&self) -> Option<(&M::Item, &M::Item)>;
    fn min_item(&self) -> Option<&M::Item>;
    fn max_item(&self) -> Option<&M::Item>;

    // fn children(&'a self) -> Option<Self::ChildIter>;
    fn last_child(&self) -> Option<&Self>;

    fn node_contents<'a>(&'a self) -> Option<Self::NonNilNode<'a>>;

    fn query<'a, A: Accumulator<M>>(&'a self, query_range: &Range<M::Item>, state: &mut A) {
        let non_nil_node: Self::NonNilNode<'a> = if let Some(non_nil_node) = self.node_contents() {
            non_nil_node
        } else {
            // in case of nil node
            // println!("nil");
            return;
        };

        let (min, max) = non_nil_node.bounds();
        let children = non_nil_node.children();
        let last_child = non_nil_node.last_child();

        let partially_contains = query_range.partially_contains(min, max);
        // println!(
        //     "min:{min:?} max:{max:?} range:{query_range} partially_contains:{partially_contains}"
        // );

        if !partially_contains {
            return;
        }

        if query_range.is_wrapping() {
            if max >= query_range.from() {
                let from = min.max(query_range.from());
                let to = max.next();
                let high_range = Range(from.clone(), to);
                // println!("h {high_range}");
                self.query(&high_range, state);
            }

            if min < query_range.to() {
                let from = min;
                let to = max.next().min(query_range.to().clone());
                let low_range = Range(from.clone(), to);
                // println!("l {low_range}");
                self.query(&low_range, state);
            }
            // // first process items and children after the boundary
            // if let Some(new_query_range) = query_range.cap_right(max.next()) {
            //     self.query(&new_query_range, state);
            // }
            //
            // // then process items and children before the boundary
            // if let Some(new_query_range) = query_range.cap_left(M::Item::zero()) {
            //     self.query(&new_query_range, state);
            // }

            return;
        }

        if query_range.fully_contains(min, max) {
            state.add_node(self);
            return;
        }

        for (child, item) in children {
            let child2: Self = child.clone();
            child2.query(query_range, state);
            if query_range.contains(item) {
                // println!("i range:{query_range} min:{min:?} max:{max:?} i:{item:?}");
                state.add_item(item);
            }
        }

        last_child.query(query_range, state);

        drop(last_child);
        drop(non_nil_node);
    }
}
