use crate::{
    range::{Rangable, Range},
    ranged_node::RangedRcNode,
    tree::cursor::Cursor,
    tree::ChildId,
    LiftingMonoid,
};

#[derive(Debug, Clone)]
pub(crate) struct Items<'a, M: LiftingMonoid>
where
    M::Item: Rangable,
{
    cursor: Cursor<'a, M>,
    range: Range<M::Item>,
    prev: Option<M::Item>,
    is_done: bool,
}

impl<'a, M> RangedRcNode<'a, M>
where
    M: LiftingMonoid,
    M::Item: Rangable,
{
    pub(crate) fn into_items(self, range: Range<M::Item>) -> Items<'a, M> {
        let mut cursor = Cursor::new(self);

        if range.is_full() {
            cursor = cursor.find_first()
        } else {
            cursor = cursor.find(range.from())
        }

        Items {
            cursor,
            range,
            prev: None,
            is_done: false,
        }
    }
}

impl<'a, M: LiftingMonoid> Iterator for Items<'a, M>
where
    M::Item: std::fmt::Debug + Rangable,
{
    type Item = M::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done {
            return None;
        }

        let result = match &self.prev {
            // in this case we are called for the first time
            None => {
                let current = self.cursor.current().clone();
                let current = current.node();
                if self.range.is_full() {
                    current.get_item(0)
                } else {
                    if let Some(result_item) = current.find_item(|item| self.range.from() <= item) {
                        Some(result_item)
                    } else {
                        // cursor.find() brought us here, so we should be in a leaf.
                        // but the leaf doesn't have this value, or a value that is larger!
                        // Maybe we can first print the cursor's current range?
                        // 2..8. (we are looking for 6). This actually makes sense.
                        // so we need to go and take the next item!
                        // but this only works of we are not in the last child's subtree.
                        // if we are, we just have to keep popping!

                        // actually, I think we are already doing that below...yup!
                        // might make sense to make this a helper function on cursor maybe...

                        'L: loop {
                            let next_cursor: Cursor<'a, M> = self.cursor.clone();
                            match next_cursor.pop() {
                                Some((next, ChildId::Last)) => {
                                    self.cursor = next;
                                    continue 'L;
                                }
                                Some((next, ChildId::Normal(idx))) => {
                                    self.cursor = next;
                                    let result = self.cursor.current().node().get_item(idx);
                                    assert!(result.is_some());

                                    break 'L result;
                                }
                                // we don't break but return here so
                                // everything stays the same and we'll always return the right value.
                                None => return None,
                            }
                        }
                    }
                }
            }

            Some(prev_item) => {
                let current = self.cursor.current().clone();
                let current = current.node();
                if current.is_leaf() {
                    if let Some(item) = current.find_item(|item| &prev_item < item) {
                        // if the current node carries the next item, update prev and return it
                        self.prev = Some(item.clone());
                        Some(item)
                    } else {
                        // otherwise move up the tree.
                        let mut up_from_child_id: Option<ChildId>;
                        'L: loop {
                            up_from_child_id = self.cursor.clone().pop().map(|(_, id)| id);
                            match up_from_child_id {
                                Some(ChildId::Last) => {
                                    continue 'L;
                                }
                                Some(ChildId::Normal(idx)) => {
                                    let result = self.cursor.current().node().get_item(idx);
                                    break 'L result;
                                }
                                None => {
                                    // if cursor.pop() returns none we are at the root.
                                    // this means we ran past the last element of the tree
                                    // and ran up the tree to find more data in a branch further right.

                                    // we don't break here so we don't update prev.
                                    // if we did, we'd start from new
                                    return None;
                                }
                            }
                        }
                    }
                } else {
                    // so we are not in a leaf.
                    // first we find the position of the previous item.
                    // the subtree at the posiiton after that contains the items that come after
                    // inside that subtree we have to descend to the first leaf and return the first item in that leaf.
                    // this is guaranteed to be successful.

                    let pos = current.item_position(|item| prev_item == item).unwrap();

                    let mut new_cursor = self.cursor.clone();
                    new_cursor = new_cursor.goto_next_child(ChildId::Normal(pos)).unwrap();
                    new_cursor = new_cursor.find_first();
                    self.cursor = new_cursor;

                    self.cursor.current().node().get_item(0)
                }
            }
        };

        assert!(result.is_some());

        let item = result.unwrap();
        match self.range.cmp(&item) {
            // this should never happen as we seek past that
            crate::range::RangeCompare::LessThan => unreachable!(),
            // this is the happy path
            crate::range::RangeCompare::IsLowerBound | crate::range::RangeCompare::Included => {}
            // this is when we found the end. just return None here
            crate::range::RangeCompare::GreaterThan
            | crate::range::RangeCompare::InBetween
            | crate::range::RangeCompare::IsUpperBound => {
                self.is_done = true;
                return None;
            }
        }

        self.prev = Some(item.clone());
        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use proptest::{prelude::prop, proptest};
    use std::{collections::HashSet, rc::Rc};

    use crate::{
        monoid::sum::SumMonoid, range::Range, ranged_node::RangedRcNode, LiftingMonoid, Node,
    };

    proptest! {
        fn items_correctness(items in prop::collection::vec(1..1000u64, 1..100usize), from in 0..1000u64, to in 0..1000u64) {
            println!();
            let item_set: HashSet<u64> = HashSet::from_iter(items.iter().cloned());
            let query_range = Range(from, to);
            println!("items used in test: {:?}", item_set);
            println!("query range: {:?}", query_range);

            let mut root = Node::<SumMonoid<u64>>::Nil(SumMonoid::lift(&0));

            for item in &item_set {
                root = root.insert(*item);
            }
            println!("in tree form: {:}", root);

            let min = items.iter().fold(10000, |acc, x| u64::min(*x, acc));
            let max = items.iter().fold(0, |acc, x| u64::max(*x, acc));

            let rc_root = Rc::new(root);
            let ranged_root = RangedRcNode::new(&rc_root, Range(min, max+1));

            let got :Vec<_> = ranged_root.into_items(query_range).collect();
            let expected: Vec<_> = item_set.iter().cloned().filter(|item| query_range.contains(&item)).collect();
            assert_eq!(got, expected);
        }
    }
}
