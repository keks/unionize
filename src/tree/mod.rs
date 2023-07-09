pub mod mem_rc;
pub mod mem_rc_bounds;

use crate::Accumulator;
use crate::Item;
use crate::Monoid;
use crate::Range;

/// Represents a 2-3-tree, which is just a narrow BTree. The items are the keys, and each node
/// holds a monoid that combines all the items in it.
pub trait Node<M>: core::fmt::Debug + Clone
where
    M: Monoid,
{
    /// Often times, it's annoying to deal with nil nodes, so we have a type that can reference
    /// inner nodes only. This moves nil checks to one place and hopefully also speeds up the code.
    type NonNilNodeRef<'a>: NonNilNodeRef<'a, M, Self>
    where
        Self: 'a;

    /// The fingerprint monoid representing all items in this node's subtree.
    fn monoid(&self) -> &M;

    /// Whether this is a nil node.
    fn is_nil(&self) -> bool;

    /// Returns a NonNilNodeRef if this is not a nil node, else returns None.
    fn node_contents<'a>(&'a self) -> Option<Self::NonNilNodeRef<'a>>;

    /// Query the tree. This will add the monoids covering all items in range to the accumulator.
    /// The query result an be read from the accumulator.
    fn query<'a, A: Accumulator<M>>(&'a self, range: &Range<M::Item>, state: &mut A) {
        query(self, range, state);
        state.finalize();
    }
}

fn query<'a, M, N, A>(root: &'a N, range: &Range<M::Item>, state: &mut A)
where
    M: Monoid,
    N: Node<M>,
    A: Accumulator<M>,
{
    let node: N::NonNilNodeRef<'a> = if let Some(non_nil_node) = root.node_contents() {
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
            query(root, &high_range, state);
        }

        if min < range.to() {
            let low_range = Range(min.clone(), range.to().clone());
            query(root, &low_range, state);
        }

        return;
    }

    if range.fully_contains(min, max) {
        state.add_node(root);
        return;
    }

    for (child, item) in node.children() {
        query(child, range, state);
        if range.contains(item) {
            state.add_item(item);
        }
    }

    query(node.last_child(), range, state);
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
