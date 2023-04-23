use std::rc::Rc;

use crate::{tree::cursor::Cursor, range::Range, tree::ChildId, LiftingMonoid, Node};

#[derive(Debug, Clone)]
pub(crate) struct Items<M: LiftingMonoid> {
    cursor: Cursor<M>,
    range: Range<M::Item>,
    prev: Option<M::Item>,
    is_done: bool,
}

impl<M: LiftingMonoid> Node<M> {
    pub(crate) fn items(root: Rc<Node<M>>, range: Range<M::Item>) -> Items<M> {
        let mut cursor = Cursor::new(root);

        match &range {
            Range::Full | Range::UpTo(_) => cursor.find_first(),

            Range::StartingFrom(x) | Range::Between(x, _) => cursor.find(&x),
        }

        Items {
            cursor,
            range,
            prev: None,
            is_done: false,
        }
    }
}

impl<M: LiftingMonoid> Iterator for Items<M> where M::Item: std::fmt::Debug {
    type Item = M::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done {
            return None;
        }

        let current = self.cursor.current();
        let result = match &self.prev {
            // in this case we are called for the first time
            None => {
                let item = match &self.range {
                    Range::Full | Range::UpTo(_) => current.get_item(0),

                    Range::StartingFrom(start) | Range::Between(start, _) => {
                        let result = current.find_item(|item| &start <= item);

                        match &result {
                            Some(_) => result,
                            None => {
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
                                    match self.cursor.pop() {
                                        Some((ChildId::Last, _)) => continue 'L,
                                        Some((ChildId::Normal(idx), _)) => {
                                            let result = self.cursor.current().get_item(idx);
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
                };

                item
            }

            Some(prev_item) => {
                if current.is_leaf() {
                    if let Some(item) = current.find_item(|item| &prev_item < item) {
                        // if the current node carries the next item, update prev and return it
                        self.prev = Some(item.clone());
                        Some(item)
                    } else {
                        // otherwise move up the tree.
                        let mut up_from_child_id: Option<ChildId>;
                        'L: loop {
                            up_from_child_id = self.cursor.pop().map(|(id, _)| id);
                            match up_from_child_id {
                                Some(ChildId::Last) => {
                                    continue 'L;
                                }
                                Some(ChildId::Normal(idx)) => {
                                    let result = self.cursor.current().get_item(idx);
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
                    self.cursor.goto_next_child(&ChildId::Normal(pos));
                    self.cursor.find_first();
                    self.cursor.current().get_item(0)
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
            crate::range::RangeCompare::GreaterThan | crate::range::RangeCompare::IsUpperBound => {
                self.is_done = true;
                return None;
            }
            crate::range::RangeCompare::InBetween => unreachable!(), // can only occur for NewRange
        }

        self.prev = Some(item.clone());
        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{range::Range, LiftingMonoid, Node, monoid::sum::SumMonoid};

    #[test]
    fn full_works() {
        let mut root = Node::<SumMonoid>::Nil(SumMonoid::lift(&0));

        root = root.insert(1);
        root = root.insert(2);
        root = root.insert(4);
        root = root.insert(16);
        root = root.insert(8);

        let items = Node::<SumMonoid>::items(Rc::new(root.clone()), Range::Full);

        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2, 4, 8, 16], collected, "{root}");
    }

    #[test]
    fn starting_from_works() {
        let mut root = Node::<SumMonoid>::Nil(SumMonoid::lift(&0));

        root = root.insert(1);
        root = root.insert(2);
        root = root.insert(4);
        root = root.insert(16);
        root = root.insert(8);

        let rc_root = Rc::new(root.clone());

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::StartingFrom(0));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2, 4, 8, 16], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::StartingFrom(1));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2, 4, 8, 16], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::StartingFrom(2));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![2, 4, 8, 16], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::StartingFrom(3));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![4, 8, 16], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::StartingFrom(4));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![4, 8, 16], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::StartingFrom(6));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![8, 16], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::StartingFrom(8));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![8, 16], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::StartingFrom(12));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![16], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::StartingFrom(16));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![16], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::StartingFrom(24));
        let collected: Vec<_> = items.collect();
        assert_eq!(Vec::<u64>::new(), collected, "{rc_root}");
    }

    #[test]
    fn up_to_works() {
        let mut root = Node::<SumMonoid>::Nil(SumMonoid::lift(&0));

        root = root.insert(1);
        root = root.insert(2);
        root = root.insert(4);
        root = root.insert(16);
        root = root.insert(8);

        let rc_root = Rc::new(root.clone());

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::UpTo(0));
        let collected: Vec<_> = items.collect();
        assert_eq!(Vec::<u64>::new(), collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::UpTo(1));
        let collected: Vec<_> = items.collect();
        assert_eq!(Vec::<u64>::new(), collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::UpTo(2));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::UpTo(3));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::UpTo(4));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::UpTo(6));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2, 4], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::UpTo(8));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2, 4], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::UpTo(12));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2, 4, 8], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::UpTo(16));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2, 4, 8], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::UpTo(24));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2, 4, 8, 16], collected, "{rc_root}");
    }

    #[test]
    fn between_works() {
        let mut root = Node::<SumMonoid>::Nil(SumMonoid::lift(&0));

        root = root.insert(1);
        root = root.insert(2);
        root = root.insert(4);
        root = root.insert(16);
        root = root.insert(8);

        let rc_root = Rc::new(root.clone());

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(0, 0));
        let collected: Vec<_> = items.collect();
        assert_eq!(Vec::<u64>::new(), collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(0, 1));
        let collected: Vec<_> = items.collect();
        assert_eq!(Vec::<u64>::new(), collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(0, 2));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(0, 3));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(0, 4));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(0, 6));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2, 4], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(0, 8));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2, 4], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(0, 12));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2, 4, 8], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(0, 16));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2, 4, 8], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(0, 24));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2, 4, 8, 16], collected, "{rc_root}");

        ////

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(1, 1));
        let collected: Vec<_> = items.collect();
        assert_eq!(Vec::<u64>::new(), collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(1, 2));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(1, 3));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(1, 4));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(1, 6));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2, 4], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(1, 8));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2, 4], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(1, 12));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2, 4, 8], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(1, 16));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2, 4, 8], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(1, 24));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![1, 2, 4, 8, 16], collected, "{rc_root}");

        ////

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(2, 2));
        let collected: Vec<_> = items.collect();
        assert_eq!(Vec::<u64>::new(), collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(2, 3));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![2], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(2, 4));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![2], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(2, 6));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![2, 4], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(2, 8));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![2, 4], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(2, 12));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![2, 4, 8], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(2, 16));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![2, 4, 8], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(2, 24));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![2, 4, 8, 16], collected, "{rc_root}");

        ////

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(3, 3));
        let collected: Vec<_> = items.collect();
        assert_eq!(Vec::<u64>::new(), collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(3, 4));
        let collected: Vec<_> = items.collect();
        assert_eq!(Vec::<u64>::new(), collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(3, 6));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![4], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(3, 8));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![4], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(3, 12));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![4, 8], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(3, 16));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![4, 8], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(3, 24));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![4, 8, 16], collected, "{rc_root}");

        ////

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(4, 4));
        let collected: Vec<_> = items.collect();
        assert_eq!(Vec::<u64>::new(), collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(4, 6));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![4], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(4, 8));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![4], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(4, 12));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![4, 8], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(4, 16));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![4, 8], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(4, 24));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![4, 8, 16], collected, "{rc_root}");

        ////

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(6, 6));
        let collected: Vec<_> = items.collect();
        assert_eq!(Vec::<u64>::new(), collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(6, 8));
        let collected: Vec<_> = items.collect();
        assert_eq!(Vec::<u64>::new(), collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(6, 12));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![8], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(6, 16));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![8], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(6, 24));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![8, 16], collected, "{rc_root}");

        ////

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(8, 8));
        let collected: Vec<_> = items.collect();
        assert_eq!(Vec::<u64>::new(), collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(8, 12));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![8], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(8, 16));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![8], collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(8, 24));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![8, 16], collected, "{rc_root}");

        ////

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(12, 12));
        let collected: Vec<_> = items.collect();
        assert_eq!(Vec::<u64>::new(), collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(12, 16));
        let collected: Vec<_> = items.collect();
        assert_eq!(Vec::<u64>::new(), collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(12, 24));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![16], collected, "{rc_root}");

        ////

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(16, 16));
        let collected: Vec<_> = items.collect();
        assert_eq!(Vec::<u64>::new(), collected, "{rc_root}");

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(16, 24));
        let collected: Vec<_> = items.collect();
        assert_eq!(vec![16], collected, "{rc_root}");

        ////

        let items = Node::<SumMonoid>::items(rc_root.clone(), Range::Between(18, 24));
        let collected: Vec<_> = items.collect();
        assert_eq!(Vec::<u64>::new(), collected, "{rc_root}");
    }
}
