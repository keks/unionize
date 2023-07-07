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
        let node: Self::NonNilNode<'a> = if let Some(non_nil_node) = self.node_contents() {
            non_nil_node
        } else {
            // in case of nil node
            return;
        };

        let (min, max) = node.bounds();

        if !query_range.partially_contains(min, max) {
            return;
        }

        if query_range.is_wrapping() {
            // this block achieves two things.
            // first, we make sure we process the items in query order, not item order.
            //   that means that with wrapping queries, we first add the items before the wrap, and
            //   then the items after it.
            // second, we make sure we don't have to deal with wrapping queries all over the place.
            //   they are annoying and doing it this way means we only need to take care of them once.

            if max >= query_range.from() {
                let high_range = Range(query_range.from().clone(), max.next());
                // println!("h {high_range}");
                self.query(&high_range, state);
            }

            if min < query_range.to() {
                let low_range = Range(min.clone(), query_range.to().clone());
                // println!("l {low_range}");
                self.query(&low_range, state);
            }

            return;
        }

        if query_range.fully_contains(min, max) {
            state.add_node(self);
            return;
        }

        // println!("min:{min:?} max:{max:?} range:{query_range}");

        for (child, item) in node.children() {
            child.query(query_range, state);
            if query_range.contains(item) {
                // println!("i range:{query_range} min:{min:?} max:{max:?} i:{item:?}");
                state.add_item(item);
            }
        }

        node.last_child().query(query_range, state);
    }
}
