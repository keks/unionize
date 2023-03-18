use crate::{sexpr::{SiseMonoid}};

use super::Node;

// I know I'm reinventing ranges but they are not enum.
// Maybe I can make this implement From<Range*>
#[derive(Debug, Clone)]
pub(crate) enum Range<T: std::fmt::Debug + Ord + Clone> {
    Full,
    UpTo(T),
    StartingFrom(T),
    Between(T, T),
}


impl<T: std::fmt::Debug + Ord + Clone + ?std::fmt::Display> std::fmt::Display for Range<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Range::Full => write!(f, ".."),
            Range::UpTo(x) => write!(f, "..{x:?}"),
            Range::StartingFrom(x) => write!(f, "{x:?}.."),
            Range::Between(x, y) => write!(f, "{x:?}..{y:?}"),
        }
    }
}

#[derive(Debug)]
pub(crate) enum RangeCompare {
    LessThan,
    IsLowerBound,
    Included,
    GreaterThan,
    IsUpperBound,
}

impl<T: std::fmt::Debug + Ord + Clone> Range<T> {
    fn contains(&self, item: &T) -> bool {
        match self {
            Self::Full => true,
            Self::UpTo(x) => item < x,
            Self::StartingFrom(x) => item >= x,
            Self::Between(x, y) => item >= x && item < y,
        }
    }

    pub(crate) fn cmp(&self, item: &T) -> RangeCompare {
        match self {
            Self::Full => RangeCompare::Included,
            Self::UpTo(x) if item < x => RangeCompare::Included,
            Self::UpTo(x) if item == x => RangeCompare::IsUpperBound,
            Self::UpTo(_) => RangeCompare::GreaterThan,
            Self::StartingFrom(x) if item > x => RangeCompare::Included,
            Self::StartingFrom(x) if item == x => RangeCompare::IsLowerBound,
            Self::StartingFrom(_) => RangeCompare::LessThan,
            Self::Between(x, _) if item < x => RangeCompare::LessThan,
            Self::Between(_, y) if item == y => RangeCompare::IsUpperBound, // this needs to be before the ==x check
            Self::Between(x, _) if item == x => RangeCompare::IsLowerBound,
            Self::Between(_, y) if item > y => RangeCompare::GreaterThan,
            Self::Between(_, _) => RangeCompare::Included,
        }
    }

    fn with_end(&self, item: T) -> Self {
        match self {
            Self::Full => Self::UpTo(item),
            Self::StartingFrom(x) => {
                assert!(x < &item, "making a range with start < end");
                Self::Between(x.clone(), item)
            }
            Self::UpTo(x) => {
                assert!(x > &item, "new end {item:?} is larger than old end {x:?}");
                Self::UpTo(item)
            }
            Self::Between(x, y) => {
                assert!(x < &item, "making a range with start < end");
                assert!(y > &item, "new end is larger than old end");
                Self::Between(x.clone(), item)
            }
        }
    }

    fn with_start(&self, item: T) -> Self {
        match self {
            Self::Full => Self::StartingFrom(item),
            Self::StartingFrom(x) => {
                assert!(&item > x, "new start is less than old start");
                Self::StartingFrom(item)
            }
            Self::UpTo(x) => {
                assert!(&item < x, "making a range with start < end");
                Self::Between(item, x.clone())
            }
            Self::Between(x, y) => {
                assert!(&item < y, "making a range with start < end");
                assert!(&item > x, "new start is less than old start");
                Self::Between(item, y.clone())
            }
        }
    }

    fn intersect(&self, other: &Self) -> Self {
        match (self, other) {
            (Range::UpTo(x), Range::UpTo(y)) => Range::UpTo(T::min(x.clone(), y.clone())),
            (Range::StartingFrom(x), Range::StartingFrom(y)) => {
                Range::StartingFrom(T::max(x.clone(), y.clone()))
            }
            (Range::Between(x1, y1), Range::Between(x2, y2)) => {
                let x = T::max(x1.clone(), x2.clone());
                let y = T::min(y1.clone(), y2.clone());
                assert!(x <= y);
                Range::Between(x, y)
            }

            (Range::Full, x) | (x, Range::Full) => x.clone(),

            (Range::StartingFrom(x), Range::UpTo(y)) | (Range::UpTo(y), Range::StartingFrom(x)) => {
                assert!(x <= y);
                Range::Between(x.clone(), y.clone())
            }

            (Range::Between(x, y1), Range::UpTo(y2)) | (Range::UpTo(y2), Range::Between(x, y1)) => {
                let y = T::min(y1.clone(), y2.clone());
                assert!(x <= &y);
                Range::Between(x.clone(), y)
            }

            (Range::Between(x1, y), Range::StartingFrom(x2))
            | (Range::StartingFrom(x2), Range::Between(x1, y)) => {
                let x = T::max(x1.clone(), x2.clone());
                assert!(&x <= y);
                Range::Between(x, y.clone())
            }
        }
    }

    fn is_subrange(&self, other: &Self) -> bool {
        match (self, other) {
            (_, Range::Full) => true,
            (Range::Full, _) => false,

            (Range::UpTo(x), Range::UpTo(y)) => x <= y,
            (Range::StartingFrom(x), Range::StartingFrom(y)) => x >= y,
            (Range::Between(x1, y1), Range::Between(x2, y2)) => x1 >= x2 && y1 <= y2,

            _ => false,
        }
    }

    fn has_overlap(&self, other: &Self) -> bool {
        match (self, other) {
            (Range::Full, _) | (_, Range::Full) => true,


            // ?-----
            //   ?---
            (Range::UpTo(_), Range::UpTo(_)) |
            // ---?
            // -----?
            (Range::StartingFrom(_), Range::StartingFrom(_)) => true,

            // x---..
            // ..---y
            (Range::StartingFrom(x), Range::UpTo(y)) |
            (Range::UpTo(y), Range::StartingFrom(x)) |
            
            // x---?
            // x-----?
            // ..---y
            (Range::UpTo(y), Range::Between(x, _)) |
            (Range::Between(x, _), Range::UpTo(y)) |

            // ?-----y
            //   ?---y
            //  x---..
            (Range::StartingFrom(x), Range::Between(_, y)) |
            (Range::Between(_, y), Range::StartingFrom(x)) => x <= y,

            //  x1---y1
            //    x2---?
            // ?---y2
            (Range::Between(x1, y1), Range::Between(x2, y2)) => (x1 >= x2 && x1 <= y2) || ( y2 >= x1 && y2 <= y2 ),
        }
    }

    fn is_valid(&self) -> bool {
        match self {
            Self::Between(x, y) => x <= y,
            _ => true,
        }
    }
}

impl<M: SiseMonoid> Node<M> {
    pub(crate) fn query_range_monoid(&self, query_range: Range<M::Item>) -> M {
        self.query_range_monoid_inner(query_range, Range::Full)
    }

    fn query_range_monoid_inner(
        &self,
        query_range: Range<M::Item>,
        node_range: Range<M::Item>,
    ) -> M {
        //println!("nd:{node}");
        //println!("qr:{query_range:?}");
        //println!("nr:{node_range:?}");
        //println!("");

        if node_range.is_subrange(&query_range) {
            return self.monoid().clone();
        }

        assert!(node_range.has_overlap(&query_range));
        assert!(query_range.is_valid());

        match self {
            Node::Nil(_) => M::neutral(),
            Node::Node2(node_data) => {
                let item = &node_data.items[0];
                let left_child = &node_data.children[0];
                let right_child = &node_data.last_child;
                match query_range.cmp(&node_data.items[0]) {
                    // format: L i R
                    // parens denote the range
                    // capital is subtree
                    // lower-case is item
                    // range can not only contain item

                    // [L) i R
                    RangeCompare::GreaterThan => left_child.query_range_monoid_inner(
                            query_range.clone(),
                            node_range.with_end(item.clone()),
                    ),

                    // [L i) R
                    RangeCompare::IsUpperBound => {
                        let left_monoid = left_child.query_range_monoid_inner(
                            query_range.clone(),
                            node_range.with_end(item.clone()),
                        );

                        left_monoid
                    }

                    // [L i R)
                    RangeCompare::Included => {
                        let first = left_child.query_range_monoid_inner(
                            
                            query_range.clone(),
                            node_range.with_end(item.clone()),
                        );
                        let second = M::lift(item);
                        let third = right_child.query_range_monoid_inner(
                            
                            query_range,
                            node_range.with_start(item.clone()),
                        );

                        first.combine(&second).combine(&third)
                    }

                    // L [i R)
                    RangeCompare::IsLowerBound => {
                        let item_monoid = M::lift(item);
                        let right_monoid = right_child.query_range_monoid_inner(
                            
                            query_range,
                            node_range.with_start(item.clone()),
                        );

                        item_monoid.combine(&right_monoid)
                    }

                    // L i [R)
                    RangeCompare::LessThan => right_child.query_range_monoid_inner(
                        query_range,
                        node_range.with_start(item.clone()),
                    ),
                }
            }
            Node::Node3(node_data) => {
                let item1 = &node_data.items[0];
                let item2 = &node_data.items[1];
                let left_child = &node_data.children[0];
                let middle_child = &node_data.children[1];
                let right_child = &node_data.last_child;

                let cmp1 = query_range.cmp(item1);
                let cmp2 = query_range.cmp(item2);

                //println!("cmp 3 | {cmp1:?} {cmp2:?}");

                match (cmp1, cmp2) {
                    // format: L i1 M i2 R
                    // parens denote the range
                    // capital is subtree
                    // lower-case is item
                    // range can not only contain item

                    // (L) i1 M i2 R
                    (RangeCompare::GreaterThan, RangeCompare::GreaterThan) => left_child.query_range_monoid_inner(
                        query_range,
                        node_range.with_end(item1.clone()),
                    ),

                    // (L i1) M i2 R
                    (RangeCompare::IsUpperBound, RangeCompare::GreaterThan) => {
                        let left_monoid = left_child.query_range_monoid_inner(
                            query_range,
                            node_range.with_end(item1.clone()),
                        );

                        left_monoid
                    }

                    // (L i1 M) i2 R
                    (RangeCompare::Included, RangeCompare::GreaterThan) => {
                        let left_monoid = left_child.query_range_monoid_inner(
                            query_range.clone(),
                            node_range.with_end(item1.clone()),
                        );
                        let item_monoid = M::lift(item1);
                        let middle_monoid = middle_child.query_range_monoid_inner(
                            query_range,
                            node_range.with_end(item2.clone()).with_start(item1.clone()),
                        );

                        left_monoid.combine(&item_monoid).combine(&middle_monoid)
                    }

                    // (L i1 M i2) R
                    (RangeCompare::Included, RangeCompare::IsUpperBound) => {
                        let left_monoid = left_child.query_range_monoid_inner(
                            query_range.clone(),
                            node_range.with_end(item1.clone()),
                        );
                        let item1_monoid = M::lift(item1);
                        let middle_monoid = middle_child.query_range_monoid_inner(
                            query_range,
                            node_range.with_end(item2.clone()).with_start(item1.clone()),
                        );

                        left_monoid
                            .combine(&item1_monoid)
                            .combine(&middle_monoid)
                    }

                    // (L i1 M i2 R)
                    (RangeCompare::Included, RangeCompare::Included) => {
                        let left_monoid = left_child.query_range_monoid_inner(
                            query_range.clone(),
                            node_range.with_end(item1.clone()),
                        );
                        let item1_monoid = M::lift(item1);
                        let middle_monoid = middle_child.query_range_monoid_inner(
                            query_range.clone(),
                            node_range.with_end(item2.clone()).with_start(item1.clone()),
                        );
                        let item2_monoid = M::lift(item2);
                        let right_monoid = right_child.query_range_monoid_inner(
                            query_range,
                            node_range.with_start(item2.clone()),
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
                            node_range.with_end(item2.clone()).with_start(item1.clone()),
                        );

                        item1_monoid.combine(&middle_monoid)
                    }

                    // L (i1 M i2) R
                    (RangeCompare::IsLowerBound, RangeCompare::IsUpperBound) => {
                        let item1_monoid = M::lift(item1);
                        let middle_monoid = middle_child.query_range_monoid_inner(
                            query_range,
                            node_range.with_end(item2.clone()).with_start(item1.clone()),
                        );

                        item1_monoid.combine(&middle_monoid)
                    }

                    // L (i1 M i2 R)
                    (RangeCompare::IsLowerBound, RangeCompare::Included) => {
                        let item1_monoid = M::lift(item1);
                        let middle_monoid = middle_child.query_range_monoid_inner(
                            query_range.clone(),
                            node_range.with_end(item2.clone()).with_start(item1.clone()),
                        );
                        let item2_monoid = M::lift(item2);
                        let right_monoid = right_child.query_range_monoid_inner(
                            query_range,
                            node_range.with_start(item2.clone()),
                        );

                        item1_monoid
                            .combine(&middle_monoid)
                            .combine(&item2_monoid)
                            .combine(&right_monoid)
                    }
                    // L i1 (M) i2 R
                    (RangeCompare::LessThan, RangeCompare::GreaterThan) => middle_child.query_range_monoid_inner(
                        query_range,
                        node_range.with_end(item2.clone()).with_start(item1.clone()),
                    ),
                    // L i1 (M i2) R
                    (RangeCompare::LessThan, RangeCompare::IsUpperBound) => {
                        let middle_monoid = middle_child.query_range_monoid_inner(
                            query_range,
                            node_range.with_end(item2.clone()).with_start(item1.clone()),
                        );

                        middle_monoid
                    }
                    // L i1 (M i2 R)
                    (RangeCompare::LessThan, RangeCompare::Included) => {
                        let middle_monoid = middle_child.query_range_monoid_inner(
                            query_range.clone(),
                            node_range.with_end(item2.clone()).with_start(item1.clone()),
                        );
                        let item2_monoid = M::lift(item2);
                        let right_monoid = right_child.query_range_monoid_inner(
                            query_range,
                            node_range.with_start(item2.clone()),
                        );

                        middle_monoid.combine(&item2_monoid).combine(&right_monoid)
                    }
                    // L i1 M (i2 R)
                    (RangeCompare::LessThan, RangeCompare::IsLowerBound) => {
                        let item2_monoid = M::lift(item2);
                        let right_monoid = right_child.query_range_monoid_inner(
                            query_range,
                            node_range.with_start(item2.clone()),
                        );

                        item2_monoid.combine(&right_monoid)
                    }
                    // L i1 M i2 (R)
                    (RangeCompare::LessThan, RangeCompare::LessThan) => right_child.query_range_monoid_inner(
                        query_range,
                        node_range.with_start(item2.clone()),
                    ),

                    // the rest doen't make sense logically. i think.
                    _ => unreachable!(),
                }
            }
        }
    }

}


#[cfg(test)]
mod test {
    use crate::{LiftingMonoid, Node, SumMonoid};

    use super::Range;

    #[test]
    fn range_querys_are_correct_for_small_node2_tree() {

        let mut root = Node::<SumMonoid>::Nil(SumMonoid::lift(&0));
        root = root.insert(30);
        root = root.insert(60);
        root = root.insert(50);

        let SumMonoid(res) = root.query_range_monoid(Range::Full);
        assert_eq!(res, 140);

        let SumMonoid(res) = root.query_range_monoid(Range::StartingFrom(25));
        assert_eq!(res, 140);
        let SumMonoid(res) = root.query_range_monoid(Range::StartingFrom(30));
        assert_eq!(res, 140);
        let SumMonoid(res) = root.query_range_monoid(Range::StartingFrom(40));
        assert_eq!(res, 110);
        let SumMonoid(res) = root.query_range_monoid(Range::StartingFrom(50));
        assert_eq!(res, 110);
        let SumMonoid(res) = root.query_range_monoid(Range::StartingFrom(60));
        assert_eq!(res, 60);
        let SumMonoid(res) = root.query_range_monoid(Range::StartingFrom(55));
        assert_eq!(res, 60);
        let SumMonoid(res) = root.query_range_monoid(Range::StartingFrom(70));
        assert_eq!(res, 0);

        let SumMonoid(res) = root.query_range_monoid(Range::UpTo(25));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::UpTo(30));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::UpTo(55));
        assert_eq!(res, 80);
        let SumMonoid(res) = root.query_range_monoid(Range::UpTo(60));
        assert_eq!(res, 80);
        let SumMonoid(res) = root.query_range_monoid(Range::UpTo(61));
        assert_eq!(res, 140);


        let SumMonoid(res) = root.query_range_monoid(Range::Between(10, 20));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(10, 30));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(10, 40));
        assert_eq!(res, 30);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(10, 50));
        assert_eq!(res, 30);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(10, 55));
        assert_eq!(res, 80);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(10, 60));
        assert_eq!(res, 80);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(10, 61));
        assert_eq!(res, 140);

        let SumMonoid(res) = root.query_range_monoid(Range::Between(30, 30));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(30, 40));
        assert_eq!(res, 30);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(30, 50));
        assert_eq!(res, 30);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(30, 55));
        assert_eq!(res, 80);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(30, 60));
        assert_eq!(res, 80);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(30, 61));
        assert_eq!(res, 140);

        let SumMonoid(res) = root.query_range_monoid(Range::Between(40, 40));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(40, 50));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(40, 55));
        assert_eq!(res, 50);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(40, 60));
        assert_eq!(res, 50);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(40, 61));
        assert_eq!(res, 110);

        let SumMonoid(res) = root.query_range_monoid(Range::Between(50, 50));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(50, 55));
        assert_eq!(res, 50);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(50, 60));
        assert_eq!(res, 50);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(50, 61));
        assert_eq!(res, 110);

        let SumMonoid(res) = root.query_range_monoid(Range::Between(55, 55));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(55, 60));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(55, 61));
        assert_eq!(res, 60);

        let SumMonoid(res) = root.query_range_monoid(Range::Between(60, 60));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(60, 61));
        assert_eq!(res, 60);

        let SumMonoid(res) = root.query_range_monoid(Range::Between(61, 70));
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

        let SumMonoid(res) = root.query_range_monoid(Range::Full);
        assert_eq!(res, 62);

        let SumMonoid(res) = root.query_range_monoid(Range::StartingFrom(0));
        assert_eq!(res, 62);
        let SumMonoid(res) = root.query_range_monoid(Range::StartingFrom(2));
        assert_eq!(res, 62);
        let SumMonoid(res) = root.query_range_monoid(Range::StartingFrom(3));
        assert_eq!(res, 60);
        let SumMonoid(res) = root.query_range_monoid(Range::StartingFrom(4));
        assert_eq!(res, 60);
        let SumMonoid(res) = root.query_range_monoid(Range::StartingFrom(6));
        assert_eq!(res, 56);
        let SumMonoid(res) = root.query_range_monoid(Range::StartingFrom(8));
        assert_eq!(res, 56);
        let SumMonoid(res) = root.query_range_monoid(Range::StartingFrom(12));
        assert_eq!(res, 48);
        let SumMonoid(res) = root.query_range_monoid(Range::StartingFrom(16));
        assert_eq!(res, 48);
        let SumMonoid(res) = root.query_range_monoid(Range::StartingFrom(24));
        assert_eq!(res, 32);
        let SumMonoid(res) = root.query_range_monoid(Range::StartingFrom(32));
        assert_eq!(res, 32);
        let SumMonoid(res) = root.query_range_monoid(Range::StartingFrom(33));
        assert_eq!(res, 0);

        let SumMonoid(res) = root.query_range_monoid(Range::UpTo(0));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::UpTo(2));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::UpTo(3));
        assert_eq!(res, 2);
        let SumMonoid(res) = root.query_range_monoid(Range::UpTo(4));
        assert_eq!(res, 2);
        let SumMonoid(res) = root.query_range_monoid(Range::UpTo(6));
        assert_eq!(res, 6);
        let SumMonoid(res) = root.query_range_monoid(Range::UpTo(8));
        assert_eq!(res, 6);
        let SumMonoid(res) = root.query_range_monoid(Range::UpTo(12));
        assert_eq!(res, 14);
        let SumMonoid(res) = root.query_range_monoid(Range::UpTo(16));
        assert_eq!(res, 14);
        let SumMonoid(res) = root.query_range_monoid(Range::UpTo(24));
        assert_eq!(res, 30);
        let SumMonoid(res) = root.query_range_monoid(Range::UpTo(32));
        assert_eq!(res, 30);
        let SumMonoid(res) = root.query_range_monoid(Range::UpTo(33));
        assert_eq!(res, 62);

        let SumMonoid(res) = root.query_range_monoid(Range::Between(0, 1));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(0, 2));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(0, 3));
        assert_eq!(res, 2);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(0, 4));
        assert_eq!(res, 2);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(0, 6));
        assert_eq!(res, 6);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(0, 8));
        assert_eq!(res, 6);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(0, 12));
        assert_eq!(res, 14);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(0, 16));
        assert_eq!(res, 14);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(0, 24));
        assert_eq!(res, 30);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(0, 32));
        assert_eq!(res, 30);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(0, 33));
        assert_eq!(res, 62);

        let SumMonoid(res) = root.query_range_monoid(Range::Between(2, 2));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(2, 3));
        assert_eq!(res, 2);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(2, 4));
        assert_eq!(res, 2);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(2, 6));
        assert_eq!(res, 6);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(2, 8));
        assert_eq!(res, 6);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(2, 12));
        assert_eq!(res, 14);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(2, 16));
        assert_eq!(res, 14);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(2, 24));
        assert_eq!(res, 30);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(2, 32));
        assert_eq!(res, 30);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(2, 33));
        assert_eq!(res, 62);

        let SumMonoid(res) = root.query_range_monoid(Range::Between(3, 4));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(3, 6));
        assert_eq!(res, 4);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(3, 8));
        assert_eq!(res, 4);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(3, 12));
        assert_eq!(res, 12);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(3, 16));
        assert_eq!(res, 12);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(3, 24));
        assert_eq!(res, 28);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(3, 32));
        assert_eq!(res, 28);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(3, 33));
        assert_eq!(res, 60);

        let SumMonoid(res) = root.query_range_monoid(Range::Between(4, 4));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(4, 6));
        assert_eq!(res, 4);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(4, 8));
        assert_eq!(res, 4);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(4, 12));
        assert_eq!(res, 12);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(4, 16));
        assert_eq!(res, 12);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(4, 24));
        assert_eq!(res, 28);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(4, 32));
        assert_eq!(res, 28);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(4, 33));
        assert_eq!(res, 60);

        let SumMonoid(res) = root.query_range_monoid(Range::Between(6, 8));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(6, 12));
        assert_eq!(res, 8);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(6, 16));
        assert_eq!(res, 8);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(6, 24));
        assert_eq!(res, 24);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(6, 32));
        assert_eq!(res, 24);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(6, 33));
        assert_eq!(res, 56);

        let SumMonoid(res) = root.query_range_monoid(Range::Between(8, 8));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(8, 12));
        assert_eq!(res, 8);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(8, 16));
        assert_eq!(res, 8);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(8, 24));
        assert_eq!(res, 24);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(8, 32));
        assert_eq!(res, 24);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(8, 33));
        assert_eq!(res, 56);

        let SumMonoid(res) = root.query_range_monoid(Range::Between(12, 16));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(12, 24));
        assert_eq!(res, 16);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(12, 32));
        assert_eq!(res, 16);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(12, 33));
        assert_eq!(res, 48);

        let SumMonoid(res) = root.query_range_monoid(Range::Between(16, 16));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(16, 24));
        assert_eq!(res, 16);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(16, 32));
        assert_eq!(res, 16);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(16, 33));
        assert_eq!(res, 48);

        let SumMonoid(res) = root.query_range_monoid(Range::Between(24, 32));
        assert_eq!(res, 0);
        let SumMonoid(res) = root.query_range_monoid(Range::Between(24, 33));
        assert_eq!(res, 32);

        let SumMonoid(res) = root.query_range_monoid(Range::Between(33, 34));
        assert_eq!(res, 0);
    }
}

