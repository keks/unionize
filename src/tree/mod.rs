pub mod mem_rc;
pub mod mem_rc_bounds;

use crate::Accumulator;
use crate::Item;
use crate::Monoid;
use crate::Range;

pub trait Node<M>: core::fmt::Debug + Clone
where
    M: Monoid,
{
    type NonNilNodeRef<'a>: NonNilNodeRef<'a, M, Self>
    where
        Self: 'a;

    fn monoid(&self) -> &M;
    fn is_nil(&self) -> bool;

    fn node_contents<'a>(&'a self) -> Option<Self::NonNilNodeRef<'a>>;

    fn query<'a, A: Accumulator<M>>(&'a self, range: &Range<M::Item>, state: &mut A) {
        let node: Self::NonNilNodeRef<'a> = if let Some(non_nil_node) = self.node_contents() {
            non_nil_node
        } else {
            // in case of nil node
            return;
        };

        let (min, max) = node.bounds();

        if !range.partially_contains(min, max) {
            return;
        }

        if range.is_wrapping() {
            // this block achieves two things.
            // first, we make sure we process the items in query order, not item order.
            //   that means that with wrapping queries, we first add the items before the wrap, and
            //   then the items after it.
            // second, we make sure we don't have to deal with wrapping queries all over the place.
            //   they are annoying and doing it this way means we only need to take care of them once.

            if max >= range.from() {
                let high_range = Range(range.from().clone(), max.next());
                self.query(&high_range, state);
            }

            if min < range.to() {
                let low_range = Range(min.clone(), range.to().clone());
                self.query(&low_range, state);
            }

            return;
        }

        if range.fully_contains(min, max) {
            state.add_node(self);
            return;
        }

        for (child, item) in node.children() {
            child.query(range, state);
            if range.contains(item) {
                state.add_item(item);
            }
        }

        node.last_child().query(range, state);
    }
}

pub trait NonNilNodeRef<'a, M, N>: core::fmt::Debug + Clone
where
    M: Monoid,
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
